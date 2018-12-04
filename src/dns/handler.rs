// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::RwLock;
use std::net::IpAddr;
use trust_dns::op::{Message, MessageType, OpCode, Query, ResponseCode};
use trust_dns::rr::{Name, Record, RecordType as TrustRecordType};
use trust_dns::rr::dnssec::SupportedAlgorithms;
use trust_dns_server::server::{Request, RequestHandler};
use trust_dns_server::authority::{AuthLookup, Authority};

use dns::zone::ZoneName;
use dns::record::{RecordName, RecordType};
use geo::locate::Locator;
use geo::region::RegionCode;
use store::store::StoreRecord;
use APP_CONF;
use APP_STORE;

pub struct DNSHandler {
    authorities: HashMap<Name, RwLock<Authority>>,
}

impl RequestHandler for DNSHandler {
    fn handle_request(&self, request: &Request) -> Message {
        let request_message = &request.message;

        trace!("request: {:?}", request_message);

        let response: Message = match request_message.message_type() {
            MessageType::Query => {
                match request_message.op_code() {
                    OpCode::Query => {
                        let response = self.lookup(request.src.ip(), &request_message);

                        trace!("query response: {:?}", response);

                        response
                    }
                    code @ _ => {
                        error!("unimplemented opcode: {:?}", code);

                        Message::error_msg(
                            request_message.id(),
                            request_message.op_code(),
                            ResponseCode::NotImp,
                        )
                    }
                }
            }
            MessageType::Response => {
                warn!(
                    "got a response as a request from id: {}",
                    request_message.id()
                );

                Message::error_msg(
                    request_message.id(),
                    request_message.op_code(),
                    ResponseCode::NotImp,
                )
            }
        };

        response
    }
}

impl DNSHandler {
    pub fn new() -> Self {
        DNSHandler { authorities: HashMap::new() }
    }

    pub fn upsert(&mut self, name: Name, authority: Authority) {
        self.authorities.insert(name, RwLock::new(authority));
    }

    pub fn lookup(&self, source: IpAddr, request: &Message) -> Message {
        let mut response: Message = Message::new();

        response.set_id(request.id());
        response.set_op_code(OpCode::Query);
        response.set_message_type(MessageType::Response);
        response.add_queries(request.queries().into_iter().cloned());

        for query in request.queries() {
            if let Some(ref_authority) = self.find_auth_recurse(query.name()) {
                let authority = &ref_authority.read().unwrap();

                info!(
                    "request: {} found authority: {}",
                    request.id(),
                    authority.origin()
                );

                let supported_algorithms = SupportedAlgorithms::new();

                // Attempt to resolve from local store
                let records_local = authority.search(query, false, supported_algorithms);

                if !records_local.is_empty() {
                    debug!("found records for query from local store: {}", query);

                    let records_local_vec = records_local
                        .iter()
                        .map(|record| record.to_owned())
                        .collect();

                    Self::serve_response_records(
                        &mut response,
                        records_local_vec,
                        &authority,
                        supported_algorithms,
                    );
                } else {
                    if let Some(records_remote) =
                        Self::records_from_store(authority, source, query)
                    {
                        debug!("found records for query from remote store: {}", query);

                        Self::serve_response_records(
                            &mut response,
                            records_remote,
                            &authority,
                            supported_algorithms,
                        );
                    } else {
                        debug!("did not find records for query: {}", query);

                        // Important: mark error response for found root domain as authoritative
                        response.set_authoritative(true);

                        match records_local {
                            AuthLookup::NoName => {
                                debug!("domain not found for query: {}", query);

                                response.set_response_code(ResponseCode::NXDomain)
                            }
                            AuthLookup::NameExists => {
                                debug!("domain found for query: {}", query);

                                response.set_response_code(ResponseCode::NoError)
                            }
                            AuthLookup::Records(..) => panic!("error, should return noerror"),
                        };

                        let soa_records = authority.soa_secure(false, supported_algorithms);

                        if soa_records.is_empty() {
                            warn!("no soa record for: {:?}", authority.origin());
                        } else {
                            response.add_name_servers(soa_records.iter().cloned());
                        }
                    }
                }
            } else {
                debug!("domain authority not found for query: {}", query);

                response.set_response_code(ResponseCode::Refused);
            }
        }

        response
    }

    fn find_auth_recurse(&self, name: &Name) -> Option<&RwLock<Authority>> {
        let authority = self.authorities.get(name);

        if authority.is_some() {
            return authority;
        } else {
            let name = name.base_name();

            if !name.is_root() {
                return self.find_auth_recurse(&name);
            }
        }

        None
    }

    fn records_from_store(
        authority: &Authority,
        source: IpAddr,
        query: &Query,
    ) -> Option<Vec<Record>> {
        let (query_name, query_type) = (query.name(), query.query_type());

        // Attempt with requested domain
        let mut records = Self::records_from_store_attempt(
            authority,
            source,
            &query_name,
            &query_name,
            &query_type,
        );

        // Attempt with wildcard domain
        if records.is_none() {
            if let Some(base_name) = query_name.to_string().splitn(2, ".").nth(1) {
                let wildcard_name_string = format!("*.{}", base_name);

                if let Ok(wildcard_name) = Name::parse(&wildcard_name_string, Some(&Name::new())) {
                    if &wildcard_name != query_name {
                        records = Self::records_from_store_attempt(
                            authority,
                            source,
                            &query_name,
                            &wildcard_name,
                            &query_type,
                        )
                    }
                }
            }
        }

        records
    }

    fn records_from_store_attempt(
        authority: &Authority,
        source: IpAddr,
        query_name_client: &Name,
        query_name_effective: &Name,
        query_type: &TrustRecordType,
    ) -> Option<Vec<Record>> {
        let zone_name = ZoneName::from_trust(&authority.origin());
        let record_name = RecordName::from_trust(&authority.origin(), query_name_effective);
        let record_type = RecordType::from_trust(query_type);

        debug!(
            "lookup record in store for query: {} {} on zone: {:?}, record: {:?}, and type: {:?}",
            query_name_effective,
            query_type,
            zone_name,
            record_name,
            record_type
        );

        match (zone_name, record_name, record_type) {
            (Some(zone_name), Some(record_name), Some(record_type)) => {
                let mut records = Vec::new();

                if let Ok(record) = APP_STORE.get(&zone_name, &record_name, &record_type) {
                    debug!(
                        "found record in store for query: {} {} with result: {:?}",
                        query_name_effective,
                        query_type,
                        record
                    );

                    // Append record direct results
                    Self::parse_from_records(query_name_client, source, &record, &mut records);
                }

                // Look for a CNAME result?
                if record_type != RecordType::CNAME {
                    if let Ok(record_cname) = APP_STORE.get(
                        &zone_name,
                        &record_name,
                        &RecordType::CNAME,
                    )
                    {
                        debug!(
                            "found cname hint record in store for query: {} {} with result: {:?}",
                            query_name_effective,
                            query_type,
                            record_cname
                        );

                        // Append CNAME hint results
                        Self::parse_from_records(
                            query_name_client,
                            source,
                            &record_cname,
                            &mut records,
                        );
                    }
                }

                // Records found? Return them immediately
                if !records.is_empty() {
                    return Some(records);
                }

                // No record found, exhaust all record types to check if name exists
                // Notice: a DNS server must return NOERROR if name exists, else NXDOMAIN
                if Self::check_name_exists(&zone_name, &record_name) == true {
                    // Name exists, return empty records (ie. NOERROR)
                    return Some(vec![]);
                }
            }
            _ => {}
        };

        None
    }

    fn parse_from_records(
        query_name_client: &Name,
        source: IpAddr,
        record: &StoreRecord,
        records: &mut Vec<Record>,
    ) {
        if let Ok(type_data) = record.kind.to_trust() {
            // Pick record value (either from Geo-DNS or global)
            let values = if let Some(ref regions) = record.regions {
                debug!(
                    "record has regions, looking up location for source ip: {}",
                    source
                );

                // Pick relevant region (from country)
                let region_wrap = match Locator::ip_to_country(source) {
                    Some(country) => {
                        let region = country.to_region_code();

                        Some(match region {
                            RegionCode::EU => (country, region, &regions.eu),
                            RegionCode::NAM => (country, region, &regions.nam),
                            RegionCode::SAM => (country, region, &regions.sam),
                            RegionCode::OC => (country, region, &regions.oc),
                            RegionCode::AF => (country, region, &regions.af),
                            RegionCode::AS => (country, region, &regions._as),
                        })
                    }
                    None => None,
                };

                if let Some(ref region_wrap_inner) = region_wrap {
                    debug!(
                        "source ip: {} located to country: {} and region: {}",
                        source,
                        region_wrap_inner.0.to_name(),
                        region_wrap_inner.1.to_name()
                    );

                    if let Some(region_values) = region_wrap_inner.2 {
                        debug!(
                            "source ip: {} region values found: {:?}",
                            source,
                            region_values
                        );

                        region_values
                    } else {
                        debug!(
                            "source ip: {} region values not found, using global values",
                            source
                        );

                        &record.values
                    }
                } else {
                    debug!(
                        "source ip: {} could not be located, using global values",
                        source
                    );

                    &record.values
                }
            } else {
                &record.values
            };

            for value in values.iter() {
                if let Ok(value_data) = value.to_trust(&record.kind) {
                    records.push(Record::from_rdata(
                        query_name_client.to_owned(),
                        record.ttl.unwrap_or(APP_CONF.dns.record_ttl),
                        type_data,
                        value_data,
                    ));
                } else {
                    warn!(
                        "could not convert to dns record type: {} with value: {:?}",
                        record.kind.to_str(),
                        value
                    );
                }
            }
        } else {
            warn!(
                "could not convert to dns record type: {}",
                record.kind.to_str()
            );
        }
    }

    fn serve_response_records(
        response: &mut Message,
        records: Vec<Record>,
        authority: &Authority,
        supported_algorithms: SupportedAlgorithms,
    ) {
        response.set_response_code(ResponseCode::NoError);
        response.set_authoritative(true);

        // Add records to response?
        if !records.is_empty() {
            response.add_answers(records);
        }

        // Add name servers to response (required)
        let ns_records = authority.ns(false, supported_algorithms);

        if ns_records.is_empty() {
            warn!("no ns records for: {:?}", authority.origin());
        } else {
            response.add_name_servers(ns_records.iter().cloned());
        }
    }

    fn check_name_exists(zone_name: &ZoneName, record_name: &RecordName) -> bool {
        // Exhaust all record types
        for record_type in RecordType::list_choices() {
            // A record exists for name and type?
            if APP_STORE
                .check(zone_name, record_name, &record_type)
                .is_ok() == true
            {
                return true;
            }
        }

        return false;
    }
}
