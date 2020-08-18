// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use trust_dns::op::{Message, MessageType, OpCode, Query, ResponseCode};
use trust_dns::rr::dnssec::SupportedAlgorithms;
use trust_dns::rr::{Name, Record, RecordType as TrustRecordType};
use trust_dns_server::authority::{AuthLookup, Authority};
use trust_dns_server::server::{Request, RequestHandler};

use super::code::CodeName;
use super::flatten::DNS_FLATTEN;
use super::health::{DNSHealth, DNSHealthStatus};
use super::metrics::{MetricsValue, METRICS_STORE};
use super::record::{RecordName, RecordType};
use super::zone::ZoneName;
use crate::geo::locate::Locator;
use crate::geo::region::RegionCode;
use crate::store::store::{StoreError, StoreRecord};
use crate::APP_CONF;
use crate::APP_STORE;

pub struct DNSHandler {
    authorities: HashMap<Name, RwLock<Authority>>,
}

impl RequestHandler for DNSHandler {
    fn handle_request(&self, request: &Request) -> Message {
        let request_message = &request.message;

        trace!("request: {:?}", request_message);

        let response: Message = match request_message.message_type() {
            MessageType::Query => match request_message.op_code() {
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
            },
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
        DNSHandler {
            authorities: HashMap::new(),
        }
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
                let zone_name = ZoneName::from_trust(&authority.origin());

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
                        request,
                        &mut response,
                        &zone_name,
                        records_local_vec,
                        &authority,
                        supported_algorithms,
                    );
                } else {
                    match Self::records_from_store(authority, &zone_name, source, query) {
                        Ok(records_remote) => {
                            if let Some(records_remote_inner) = records_remote {
                                debug!(
                                    "found {} records for query from remote store: {}",
                                    records_remote_inner.len(),
                                    query
                                );

                                Self::serve_response_records(
                                    request,
                                    &mut response,
                                    &zone_name,
                                    records_remote_inner,
                                    &authority,
                                    supported_algorithms,
                                );
                            } else {
                                debug!("did not find records for query: {}", query);

                                match records_local {
                                    AuthLookup::NoName => {
                                        debug!("domain not found for query: {}", query);

                                        Self::stamp_response(
                                            request,
                                            &mut response,
                                            authority,
                                            supported_algorithms,
                                            ResponseCode::NXDomain,
                                            &zone_name,
                                            false,
                                        );
                                    }
                                    AuthLookup::NameExists => {
                                        debug!("domain found for query: {}", query);

                                        Self::stamp_response(
                                            request,
                                            &mut response,
                                            authority,
                                            supported_algorithms,
                                            ResponseCode::NoError,
                                            &zone_name,
                                            false,
                                        );
                                    }
                                    AuthLookup::Records(..) => {
                                        panic!("error, should return noerror")
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            debug!("query refused for: {} because: {}", query, err);

                            Self::stamp_response(
                                request,
                                &mut response,
                                authority,
                                supported_algorithms,
                                err,
                                &zone_name,
                                false,
                            );
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
        zone_name: &Option<ZoneName>,
        source: IpAddr,
        query: &Query,
    ) -> Result<Option<Vec<Record>>, ResponseCode> {
        let (query_name, query_type) = (query.name(), query.query_type());
        let record_type = RecordType::from_trust(&query_type);

        // Stack query type to metrics?
        if let Some(ref zone_name) = zone_name {
            METRICS_STORE.stack(zone_name, MetricsValue::QueryType(&record_type));
        }

        // Attempt with requested domain
        let mut records = Self::records_from_store_attempt(
            authority,
            source,
            zone_name,
            &query_name,
            &query_name,
            &query_type,
            &record_type,
        )?;

        // Check if 'records' is empty
        let is_records_empty = if let Some(ref records_inner) = records {
            records_inner.is_empty()
        } else {
            records.is_none()
        };

        // Attempt with wildcard domain? (records empty)
        if is_records_empty == true {
            debug!(
                "got empty records from store, attempting wildcard for query: {}",
                query
            );

            if let Some(base_name) = query_name.to_string().splitn(2, ".").nth(1) {
                let wildcard_name_string = format!("*.{}", base_name);

                if let Ok(wildcard_name) = Name::parse(&wildcard_name_string, Some(&Name::new())) {
                    if &wildcard_name != query_name {
                        let records_wildcard = Self::records_from_store_attempt(
                            authority,
                            source,
                            &zone_name,
                            &query_name,
                            &wildcard_name,
                            &query_type,
                            &record_type,
                        )?;

                        // Assign non-none wildcard records? (retain any NOERROR from 'records')
                        if records_wildcard.is_none() == false {
                            records = records_wildcard
                        }
                    }
                }
            }
        }

        Ok(records)
    }

    fn records_from_store_attempt(
        authority: &Authority,
        source: IpAddr,
        zone_name: &Option<ZoneName>,
        query_name_client: &Name,
        query_name_effective: &Name,
        query_type: &TrustRecordType,
        record_type: &Option<RecordType>,
    ) -> Result<Option<Vec<Record>>, ResponseCode> {
        let record_name = RecordName::from_trust(&authority.origin(), query_name_effective);

        debug!(
            "lookup record in store for query: {} {} on zone: {:?}, record: {:?}, and type: {:?}",
            query_name_effective, query_type, zone_name, record_name, record_type
        );

        match (zone_name.as_ref(), record_name) {
            (Some(zone_name), Some(record_name)) => {
                let mut records = Vec::new();

                if let &Some(ref record_type_inner) = record_type {
                    match APP_STORE.get(&zone_name, &record_name, record_type_inner) {
                        Ok(record) => {
                            debug!(
                                "found record in store for query: {} {}; got: {:?}",
                                query_name_effective, query_type, record
                            );

                            // Append record direct results
                            Self::parse_from_records(
                                query_name_client,
                                query_type,
                                record_type_inner,
                                source,
                                &zone_name,
                                &record,
                                &mut records,
                            );
                        }
                        Err(StoreError::Disconnected) => {
                            // Store is down, consider it as a DNS server failure (this avoids \
                            //   polluting recursive DNS caches)
                            return Err(ResponseCode::ServFail);
                        }
                        _ => {}
                    }

                    // Look for a CNAME result? (if no records were acquired)
                    if record_type_inner != &RecordType::CNAME && records.is_empty() {
                        match APP_STORE.get(&zone_name, &record_name, &RecordType::CNAME) {
                            Ok(record_cname) => {
                                debug!(
                                    "found cname hint record in store for query: {} {}; got: {:?}",
                                    query_name_effective, query_type, record_cname
                                );

                                // Append CNAME hint results
                                Self::parse_from_records(
                                    query_name_client,
                                    query_type,
                                    record_type_inner,
                                    source,
                                    &zone_name,
                                    &record_cname,
                                    &mut records,
                                );
                            }
                            Err(StoreError::Disconnected) => {
                                // Store is down, consider it as a DNS server failure (this avoids \
                                //   polluting recursive DNS caches)
                                return Err(ResponseCode::ServFail);
                            }
                            _ => {}
                        }
                    }
                }

                // Records found? Return them immediately
                if !records.is_empty() {
                    return Ok(Some(records));
                }

                // No record found, exhaust all record types to check if name exists
                // Notice: a DNS server must return NOERROR if name exists, else NXDOMAIN
                if Self::check_name_exists(&zone_name, &record_name)? == true {
                    // Name exists, return empty records (ie. NOERROR)
                    return Ok(Some(vec![]));
                }
            }
            _ => {}
        };

        Ok(None)
    }

    fn parse_from_records(
        query_name_client: &Name,
        query_type: &TrustRecordType,
        record_type: &RecordType,
        source: IpAddr,
        zone_name: &ZoneName,
        record: &StoreRecord,
        records: &mut Vec<Record>,
    ) {
        if let Ok(type_data) = record.kind.to_trust() {
            // Check if should resolve IP to country?
            let ip_country =
                if record.blackhole.is_some() == true || record.regions.is_some() == true {
                    debug!(
                        "record is location-aware, looking up location for source ip: {}",
                        source
                    );

                    Locator::ip_to_country(source)
                } else {
                    None
                };

            // Stack query origin to metrics (country will be 'none' if not resolved)
            // Notice: it will not be resolved for metrics purposes only, so in that case the \
            //   country will be 'none' even if it could have been detected.
            METRICS_STORE.stack(zone_name, MetricsValue::QueryOrigin(&ip_country));

            // Check if country is blackholed
            let mut is_blackholed = false;

            if let Some(ref blackhole) = record.blackhole {
                debug!("record has blackhole");

                if let Some(ref ip_country) = ip_country {
                    if blackhole.has_country(ip_country) == true {
                        debug!(
                            "source ip: {} country: {:?} appears in blackhole",
                            source, ip_country
                        );

                        is_blackholed = true;
                    } else {
                        debug!(
                            "source ip: {} country: {:?} does not appear in blackhole",
                            source, ip_country
                        );
                    }
                }
            }

            // Pick record value (either from Geo-DNS or global)
            let values = if let Some(ref regions) = record.regions {
                debug!("record has regions");

                // Pick relevant region (from country)
                let region_wrap = match ip_country {
                    Some(country) => {
                        let region = country.to_region_code();

                        Some(match region {
                            RegionCode::NNAM => (country, region, &regions.nnam),
                            RegionCode::SNAM => (country, region, &regions.snam),
                            RegionCode::NSAM => (country, region, &regions.nsam),
                            RegionCode::SSAM => (country, region, &regions.ssam),
                            RegionCode::WEU => (country, region, &regions.weu),
                            RegionCode::CEU => (country, region, &regions.ceu),
                            RegionCode::EEU => (country, region, &regions.eeu),
                            RegionCode::RU => (country, region, &regions.ru),
                            RegionCode::ME => (country, region, &regions.me),
                            RegionCode::NAF => (country, region, &regions.naf),
                            RegionCode::MAF => (country, region, &regions.maf),
                            RegionCode::SAF => (country, region, &regions.saf),
                            RegionCode::IN => (country, region, &regions._in),
                            RegionCode::SEAS => (country, region, &regions.seas),
                            RegionCode::NEAS => (country, region, &regions.neas),
                            RegionCode::OC => (country, region, &regions.oc),
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
                            source, region_values
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

            // Not blackholed? (push values)
            if is_blackholed == false {
                // Acquire record TTL
                let record_ttl = record.ttl.unwrap_or(APP_CONF.dns.record_ttl);

                // Aggregate values (healthy ones only for DNS health check)
                let mut prepared_values = values
                    .iter()
                    .filter(|value| {
                        // Check if value was not checked as dead for zone name and record name
                        DNSHealth::status(zone_name, record_type, &record.name, &value)
                            != DNSHealthStatus::Dead
                    })
                    .collect::<Vec<_>>();

                // No aggregated value? Fallback on 'rescue' records? (if any)
                if prepared_values.is_empty() == true {
                    info!(
                        "all dns record values reported as dead, attempting to use rescue values"
                    );

                    if let Some(ref rescue) = record.rescue {
                        prepared_values.extend(rescue.iter());
                    }
                }

                // Replace CNAME values with their flattened value?
                let mut flat_values = None;

                if record.kind == RecordType::CNAME && record.flatten == Some(true) {
                    if record_type == &RecordType::CNAME {
                        debug!(
                            "cname requested and found, but record is flattened, so clearing it"
                        );

                        // If DNS query looks up CNAME value, it will give back an empty answer \
                        //   (as it should have been flattened for other query types)
                        flat_values = Some(Vec::new());
                    } else {
                        debug!("record is flattened, acquiring cname values");

                        // Flatten each CNAME value (if there are multiple ones)
                        for prepared_value in prepared_values.iter() {
                            // Notice: this will ignore any errored flattening pass, which may \
                            //   thus return an empty final DNS result if there is no flattened \
                            //   value.
                            if let Ok(flat_pass) = DNS_FLATTEN.pass(
                                record_type.to_owned(),
                                (*prepared_value).to_owned(),
                                record_ttl,
                            ) {
                                let mut flat_values_list = Vec::new();

                                for flat_value in flat_pass.iter() {
                                    // De-duplicate returned values, as multiple CNAMEs could \
                                    //   return the same flat value twice or more.
                                    if flat_values_list.contains(flat_value) == false {
                                        flat_values_list.push(flat_value.to_owned())
                                    }
                                }

                                flat_values = Some(flat_values_list);
                            }
                        }
                    }
                }

                // Build final values
                let (final_kind, final_type, mut final_values);

                if let Some(ref flat_values_list) = flat_values {
                    final_kind = record_type;
                    final_type = *query_type;
                    final_values = Vec::new();

                    for flat_value in flat_values_list.iter() {
                        final_values.push(flat_value);
                    }
                } else {
                    final_kind = &record.kind;
                    final_type = type_data;
                    final_values = prepared_values;
                }

                // Append final prepared values to response
                for value in final_values {
                    if let Ok(value_data) = value.to_trust(final_kind) {
                        records.push(Record::from_rdata(
                            query_name_client.to_owned(),
                            record_ttl,
                            final_type,
                            value_data,
                        ));
                    } else {
                        warn!(
                            "could not convert to dns record type: {} with value: {:?}",
                            final_kind.to_str(),
                            value
                        );
                    }
                }
            } else {
                info!("did not push record values because country is blackholed");
            }
        } else {
            warn!(
                "could not convert to dns record type: {}",
                record.kind.to_str()
            );
        }
    }

    fn serve_response_records(
        request: &Message,
        response: &mut Message,
        zone_name: &Option<ZoneName>,
        mut records: Vec<Record>,
        authority: &Authority,
        supported_algorithms: SupportedAlgorithms,
    ) {
        let has_records = !records.is_empty();

        // Stamp response with flags and required response data
        Self::stamp_response(
            request,
            response,
            authority,
            supported_algorithms,
            ResponseCode::NoError,
            zone_name,
            has_records,
        );

        // Add records to response?
        if has_records == true {
            // Randomize records order, as most DNS servers do to balance eg. IP resource usage
            if records.len() > 1 {
                records.shuffle(&mut thread_rng());
            }

            response.add_answers(records);
        }
    }

    fn stamp_response(
        request: &Message,
        response: &mut Message,
        authority: &Authority,
        supported_algorithms: SupportedAlgorithms,
        code: ResponseCode,
        zone_name: &Option<ZoneName>,
        has_records: bool,
    ) {
        // Stack answer code to metrics?
        if let Some(ref zone_name) = zone_name {
            let code_name = CodeName::from_trust(&code);

            METRICS_STORE.stack(zone_name, MetricsValue::AnswerCode(&code_name));
        }

        // Stamp with response code
        response.set_response_code(code);

        // Stamp response with 'AA' flag (we are authoritative on served zone)
        response.set_authoritative(true);

        // Stamp response with 'RD' flag? (if requested by client)
        if request.recursion_desired() == true {
            response.set_recursion_desired(true);
        }

        // Add SOA records? (if response is empty)
        if has_records == false {
            let soa_records = authority.soa_secure(false, supported_algorithms);

            if soa_records.is_empty() {
                warn!("no soa record for: {:?}", authority.origin());
            } else {
                response.add_name_servers(soa_records.iter().cloned());
            }
        }
    }

    fn check_name_exists(
        zone_name: &ZoneName,
        record_name: &RecordName,
    ) -> Result<bool, ResponseCode> {
        // Exhaust all record types
        for record_type in RecordType::list_choices() {
            // A record exists for name and type?
            match APP_STORE.check(zone_name, record_name, &record_type) {
                Ok(_) => {
                    // Record exists for name and type; abort there.
                    return Ok(true);
                }
                Err(StoreError::Disconnected) => {
                    // Store is down, consider it as a DNS server failure (this avoids polluting \
                    //   recursive DNS caches); abort there.
                    return Err(ResponseCode::ServFail);
                }
                _ => {}
            }
        }

        // No alternate record found, consider name as non-existing.
        return Ok(false);
    }
}
