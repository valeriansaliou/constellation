// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::op::header::Header;
use hickory_proto::op::{LowerQuery, MessageType, OpCode, ResponseCode};
use hickory_proto::rr::{LowerName, Name, Record, RecordType as HickoryRecordType};
use hickory_server::authority::{
    AuthLookup, Authority, LookupOptions, MessageRequest, MessageResponseBuilder,
};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::store::in_memory::InMemoryAuthority;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::Error;
use std::net::IpAddr;

use super::code::CodeName;
use super::flatten::DNS_FLATTEN;
use super::health::{DNSHealth, DNSHealthStatus};
use super::metrics::{MetricsValue, METRICS_STORE};
use super::record::{RecordName, RecordType};
use super::zone::ZoneName;
use crate::geo::locate::Locator;
use crate::geo::region::RegionCode;
use crate::store::store::{StoreAccessOrigin, StoreError, StoreRecord};
use crate::APP_CONF;
use crate::APP_STORE;

pub type DNSAuthority = InMemoryAuthority;
type DNSResponse = Result<ResponseInfo, Error>;

pub struct DNSHandler {
    authorities: HashMap<LowerName, DNSAuthority>,
}

#[async_trait::async_trait]
impl RequestHandler for DNSHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        responder: R,
    ) -> ResponseInfo {
        match self.handle_request(responder, request).await {
            Ok(info) => {
                debug!("success handling dns request");

                info
            }
            Err(error) => {
                error!("error handling dns request: {}", error);

                let mut header = Header::new();

                header.set_response_code(ResponseCode::ServFail);
                header.into()
            }
        }
    }
}

impl DNSHandler {
    pub fn new() -> Self {
        DNSHandler {
            authorities: HashMap::new(),
        }
    }

    pub fn add_authority(&mut self, name: LowerName, authority: DNSAuthority) {
        self.authorities.insert(name, authority);
    }

    async fn handle_request<R: ResponseHandler>(
        &self,
        responder: R,
        request: &Request,
    ) -> DNSResponse {
        trace!("request: {:?} from: {}", request, request.src().ip());

        match request.message_type() {
            MessageType::Query => match request.op_code() {
                OpCode::Query => {
                    info!("lookup request with identifier: {}", request.id());

                    self.lookup(responder, request).await
                }
                code @ _ => {
                    error!("unimplemented dns operation code: {:?}", code);

                    self.abort(ResponseCode::NotImp)
                }
            },
            MessageType::Response => {
                warn!(
                    "got a response as a request from identifier: {}",
                    request.id()
                );

                self.abort(ResponseCode::NotImp)
            }
        }
    }

    async fn lookup<R: ResponseHandler>(&self, responder: R, request: &Request) -> DNSResponse {
        // Generate response header
        let mut header: Header = Header::new();

        header.set_id(request.id());
        header.set_op_code(OpCode::Query);
        header.set_message_type(MessageType::Response);

        // #1. Acquire base authority (ie. zone) for request
        // Notice: if zone cannot be found, then reject straight away.
        // Notice: since we checked the status of the unwrapped query variable, this is panic-safe.
        let query = request.query();
        let authority_lookup = self.find_authority_recurse(query.name());

        if authority_lookup.is_none() == true {
            return self
                .lookup_no_authority(responder, request, header, query)
                .await;
        }

        // #2. Handle the query
        let authority = authority_lookup.unwrap();
        let zone_name = ZoneName::from_hickory(&authority.origin());

        let soa_records = authority.soa().await.unwrap_or(AuthLookup::Empty);
        let soa_records_vec = soa_records.iter().collect();

        info!(
            "request: {} found authority: {}",
            request.id(),
            authority.origin()
        );

        // #3. Attempt to resolve from local store
        // Notice: this is used to serve local SOA and NS records.
        let records_local = authority
            .search(request.request_info(), LookupOptions::default())
            .await
            .unwrap_or(AuthLookup::Empty);

        if !records_local.is_empty() {
            let records_local_vec = records_local.iter().collect();

            return self
                .lookup_local(
                    responder,
                    request,
                    header,
                    query,
                    zone_name,
                    soa_records_vec,
                    records_local_vec,
                )
                .await;
        }

        // #4. Resolve from remote store
        // Notice: this is used to serve all records set with the HTTP API.
        return match Self::records_from_store(authority, &zone_name, request.src().ip(), query)
            .await
        {
            Ok(records_remote) => {
                // Serve response data?
                if let Some(records_remote_inner) = records_remote {
                    self.lookup_remote_some(
                        responder,
                        request,
                        header,
                        query,
                        zone_name,
                        soa_records_vec,
                        records_remote_inner,
                    )
                    .await
                } else {
                    self.lookup_remote_none(
                        responder,
                        request,
                        header,
                        query,
                        zone_name,
                        soa_records_vec,
                        records_local,
                    )
                    .await
                }
            }
            Err(err) => {
                self.lookup_remote_fail(
                    responder,
                    request,
                    header,
                    query,
                    zone_name,
                    soa_records_vec,
                    err,
                )
                .await
            }
        };
    }

    async fn lookup_no_authority<R: ResponseHandler>(
        &self,
        responder: R,
        request: &MessageRequest,
        mut header: Header,
        query: &LowerQuery,
    ) -> DNSResponse {
        debug!(
            "domain authority not found for query: {:?} on request: {}",
            query,
            request.id()
        );

        header.set_response_code(ResponseCode::Refused);

        // Authority not found response dispatch
        Self::dispatch_response(responder, request, header, None, None).await
    }

    async fn lookup_local<'a, R: ResponseHandler>(
        &self,
        responder: R,
        request: &MessageRequest,
        header: Header,
        query: &LowerQuery,
        zone_name: Option<ZoneName>,
        soa_records: Vec<&'a Record>,
        local_records: Vec<&'a Record>,
    ) -> Result<ResponseInfo, Error> {
        debug!("found records for query from local store: {:?}", query);

        Self::serve_response_records(
            responder,
            request,
            header,
            &zone_name,
            local_records,
            soa_records,
        )
        .await
    }

    async fn lookup_remote_some<'a, R: ResponseHandler>(
        &self,
        responder: R,
        request: &MessageRequest,
        header: Header,
        query: &LowerQuery,
        zone_name: Option<ZoneName>,
        soa_records: Vec<&'a Record>,
        records_remote: Vec<Record>,
    ) -> Result<ResponseInfo, Error> {
        debug!(
            "found {} records for query from remote store: {:?}",
            records_remote.len(),
            query
        );

        let records_remote_vec = records_remote.iter().collect();

        // Dispatch request from this block, as we cannot escape generated \
        //   record values lifetimes out of this context.
        Self::serve_response_records(
            responder,
            request,
            header,
            &zone_name,
            records_remote_vec,
            soa_records,
        )
        .await
    }

    async fn lookup_remote_none<'a, R: ResponseHandler>(
        &self,
        responder: R,
        request: &MessageRequest,
        mut header: Header,
        query: &LowerQuery,
        zone_name: Option<ZoneName>,
        soa_records: Vec<&'a Record>,
        records_local: AuthLookup,
    ) -> Result<ResponseInfo, Error> {
        // Serve error code
        debug!("did not find records for query: {:?}", query);

        let response_error = match records_local {
            AuthLookup::Empty => {
                debug!("domain not found for query: {:?}", query);

                ResponseCode::NXDomain
            }
            AuthLookup::SOA { .. } => {
                debug!("domain found for query: {:?}", query);

                ResponseCode::NoError
            }
            AuthLookup::Records { .. } | AuthLookup::AXFR { .. } => {
                // This code path is unexpected and should never be reached
                panic!("error, should return noerror")
            }
        };

        Self::stamp_header(request, &mut header, response_error, &zone_name);

        // Dispatch empty records response
        Self::dispatch_response(responder, request, header, None, Some(soa_records)).await
    }

    async fn lookup_remote_fail<'a, R: ResponseHandler>(
        &self,
        responder: R,
        request: &MessageRequest,
        mut header: Header,
        query: &LowerQuery,
        zone_name: Option<ZoneName>,
        soa_records: Vec<&'a Record>,
        code: ResponseCode,
    ) -> Result<ResponseInfo, Error> {
        debug!("query refused for: {:?} because: {}", query, code);

        Self::stamp_header(request, &mut header, code, &zone_name);

        // Dispatch error response
        Self::dispatch_response(responder, request, header, None, Some(soa_records)).await
    }

    fn abort(&self, code: ResponseCode) -> DNSResponse {
        let mut header = Header::new();

        header.set_response_code(code);

        Ok(header.into())
    }

    async fn dispatch_response<'a, R: ResponseHandler>(
        mut responder: R,
        request: &MessageRequest,
        header: Header,
        records: Option<Vec<&'a Record>>,
        soa_records: Option<Vec<&Record>>,
    ) -> Result<ResponseInfo, Error> {
        let mut records = records.unwrap_or(vec![]);
        let has_records = !records.is_empty();

        // Add records to response?
        if has_records == true {
            // Randomize records order, as most DNS servers do to balance eg. IP resource usage
            if records.len() > 1 {
                records.shuffle(&mut thread_rng());
            }
        }

        // Acquire response SOA records
        // Notice: only append SOA records if this is an empty response
        let soa_records = if records.is_empty() {
            soa_records
        } else {
            None
        };

        // Dispatch final response message
        let response_message = MessageResponseBuilder::from_message_request(request).build(
            header,
            records,
            &[],
            soa_records.unwrap_or(vec![]),
            &[],
        );

        trace!("query response: {:?}", response_message);

        responder.send_response(response_message).await
    }

    fn stamp_header<'a, 'b>(
        request: &MessageRequest,
        header: &mut Header,
        code: ResponseCode,
        zone_name: &Option<ZoneName>,
    ) {
        // Stack answer code to metrics?
        if let Some(ref zone_name) = zone_name {
            let code_name = CodeName::from_hickory(&code);

            METRICS_STORE.stack(zone_name, MetricsValue::AnswerCode(&code_name));
        }

        // Stamp with response code
        header.set_response_code(code);

        // Stamp response with 'AA' flag (we are authoritative on served zone)
        header.set_authoritative(true);

        // Stamp response with 'RD' flag? (if requested by client)
        if request.recursion_desired() == true {
            header.set_recursion_desired(true);
        }
    }

    async fn serve_response_records<'a, 'b, R: ResponseHandler>(
        responder: R,
        request: &MessageRequest,
        mut header: Header,
        zone_name: &Option<ZoneName>,
        records: Vec<&'a Record>,
        soa_records: Vec<&'a Record>,
    ) -> DNSResponse {
        Self::stamp_header(request, &mut header, ResponseCode::NoError, zone_name);

        Self::dispatch_response(responder, request, header, Some(records), Some(soa_records)).await
    }

    fn find_authority_recurse(&self, name: &LowerName) -> Option<&DNSAuthority> {
        let authority = self.authorities.get(name);

        if authority.is_some() {
            return authority;
        } else {
            let name = name.base_name();

            if !name.is_root() {
                return self.find_authority_recurse(&name);
            }
        }

        None
    }

    async fn records_from_store(
        authority: &DNSAuthority,
        zone_name: &Option<ZoneName>,
        source: IpAddr,
        query: &LowerQuery,
    ) -> Result<Option<Vec<Record>>, ResponseCode> {
        let (query_name, query_type) = (query.name(), query.query_type());
        let record_type = RecordType::from_hickory(&query_type);

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
        )
        .await?;

        // Check if 'records' is empty
        let is_records_empty = if let Some(ref records_inner) = records {
            records_inner.is_empty()
        } else {
            records.is_none()
        };

        // Attempt with wildcard domain? (records empty)
        if is_records_empty == true {
            debug!(
                "got empty records from store, attempting wildcard for query: {:?}",
                query
            );

            if let Some(base_name) = query_name.to_string().splitn(2, ".").nth(1) {
                let wildcard_name_string = format!("*.{}", base_name);

                if let Ok(wildcard_name) = Name::parse(&wildcard_name_string, Some(&Name::new())) {
                    let wildcard_name_lower = LowerName::new(&wildcard_name);

                    if &wildcard_name_lower != query_name {
                        let records_wildcard = Self::records_from_store_attempt(
                            authority,
                            source,
                            &zone_name,
                            &query_name,
                            &wildcard_name_lower,
                            &query_type,
                            &record_type,
                        )
                        .await?;

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

    async fn records_from_store_attempt(
        authority: &DNSAuthority,
        source: IpAddr,
        zone_name: &Option<ZoneName>,
        query_name_client: &LowerName,
        query_name_effective: &LowerName,
        query_type: &HickoryRecordType,
        record_type: &Option<RecordType>,
    ) -> Result<Option<Vec<Record>>, ResponseCode> {
        let record_name = RecordName::from_hickory(&authority.origin(), query_name_effective);

        debug!(
            "lookup record in store for query: {} {} on zone: {:?}, record: {:?}, and type: {:?}",
            query_name_effective, query_type, zone_name, record_name, record_type
        );

        match (zone_name.as_ref(), record_name) {
            (Some(zone_name), Some(record_name)) => {
                let mut records = Vec::new();

                if let &Some(ref record_type_inner) = record_type {
                    match APP_STORE
                        .get(
                            &zone_name,
                            &record_name,
                            record_type_inner,
                            StoreAccessOrigin::External,
                        )
                        .await
                    {
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
                        match APP_STORE
                            .get(
                                &zone_name,
                                &record_name,
                                &RecordType::CNAME,
                                StoreAccessOrigin::External,
                            )
                            .await
                        {
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
                if Self::check_name_exists(&zone_name, &record_name, StoreAccessOrigin::External)
                    .await?
                    == true
                {
                    // Name exists, return empty records (ie. NOERROR)
                    return Ok(Some(vec![]));
                }
            }
            _ => {}
        };

        Ok(None)
    }

    fn parse_from_records(
        query_name_client: &LowerName,
        query_type: &HickoryRecordType,
        record_type: &RecordType,
        source: IpAddr,
        zone_name: &ZoneName,
        record: &StoreRecord,
        records: &mut Vec<Record>,
    ) {
        if let Ok(type_data) = record.kind.to_hickory() {
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
                        let (mut flat_values_list, mut has_flattened) = (Vec::new(), false);

                        for prepared_value in prepared_values.iter() {
                            // Notice: this will ignore any errored flattening pass, which may \
                            //   thus return an empty final DNS result if there is no flattened \
                            //   value.
                            if let Ok(flat_pass) = DNS_FLATTEN.pass(
                                record_type.to_owned(),
                                (*prepared_value).to_owned(),
                                record_ttl,
                            ) {
                                has_flattened = true;

                                for flat_value in flat_pass.iter() {
                                    // De-duplicate returned values, as multiple CNAMEs could \
                                    //   return the same flat value twice or more.
                                    if flat_values_list.contains(flat_value) == false {
                                        flat_values_list.push(flat_value.to_owned())
                                    }
                                }
                            }
                        }

                        // Delegate flattened & merged values to outer context?
                        if has_flattened == true {
                            flat_values = Some(flat_values_list);
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
                    if let Ok(value_data) = value.to_hickory(final_kind) {
                        let mut record = Record::from_rdata(
                            Name::from(query_name_client.to_owned()),
                            record_ttl,
                            value_data,
                        );

                        record.set_record_type(final_type);

                        records.push(record);
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

    async fn check_name_exists(
        zone_name: &ZoneName,
        record_name: &RecordName,
        origin: StoreAccessOrigin,
    ) -> Result<bool, ResponseCode> {
        // Exhaust all record types
        for record_type in RecordType::list_choices() {
            // A record exists for name and type?
            // Notice: instead of performing a simple exist check, we acquire full record data, \
            //   as this lets us use the local store and therefore prevent non-existing domain \
            //   attacks on the remote store.
            match APP_STORE
                .get(zone_name, record_name, &record_type, origin)
                .await
            {
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
