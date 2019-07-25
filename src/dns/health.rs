// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http_req::error::Error;
use http_req::request::{Method, RequestBuilder};
use http_req::response::{Headers, Response, StatusCode};
use http_req::tls;
use http_req::uri::Uri;
use std::collections::HashSet;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::config::config::{ConfigDNSHealthHTTP, ConfigDNSHealthHTTPMethod};
use crate::dns::record::{RecordName, RecordType, RecordValue};
use crate::dns::zone::ZoneName;
use crate::APP_CONF;
use crate::APP_STORE;

lazy_static! {
    static ref HEALTH_DEAD_REGISTER: RwLock<HashSet<(ZoneName, RecordName, RecordValue)>> =
        RwLock::new(HashSet::new());
}

const HEALTH_CHECK_RECORD_TYPES: [RecordType; 3] =
    [RecordType::A, RecordType::AAAA, RecordType::CNAME];
const HEALTH_CHECK_FAILED_STATUS: StatusCode = StatusCode::new(503);
const HEALTH_CHECK_PROBE_USERAGENT: &'static str = "constellation (health-check)";

pub struct DNSHealthBuilder;
pub struct DNSHealth;

struct DNSHealthHTTP;

#[derive(PartialEq)]
pub enum DNSHealthStatus {
    Unchecked,
    Healthy,
    Dead,
}

impl DNSHealthBuilder {
    pub fn new() -> DNSHealth {
        DNSHealth {}
    }
}

impl DNSHealth {
    pub fn run(&self) {
        let interval_duration = Duration::from_secs(APP_CONF.dns.health.check_interval);

        debug!("scheduled dns health checks every: {:?}", interval_duration);

        loop {
            info!("running a dns health check operation...");

            // Hold on 1 second
            thread::sleep(Duration::from_secs(1));

            Self::run_checks();

            info!("ran dns health check operation");

            // Hold for next check run
            thread::sleep(interval_duration);
        }
    }

    pub fn status(
        zone_name: &ZoneName,
        record_type: &RecordType,
        record_name: &RecordName,
        record_value: &RecordValue,
    ) -> DNSHealthStatus {
        // Check if record value is seen as dead?
        if APP_CONF.dns.health.check_enable == true && Self::should_check_record(record_type) {
            debug!(
                "checking local dns health status for record: {:?} at chain: {:?} / {:?} / {:?}",
                record_type, zone_name, record_name, record_value
            );

            // Record contained in dead register?
            // Notice: there is unfortunately no other way around than doing clones there
            if HEALTH_DEAD_REGISTER.read().unwrap().contains(&(
                zone_name.clone(),
                record_name.clone(),
                record_value.clone(),
            )) == true
            {
                DNSHealthStatus::Dead
            } else {
                DNSHealthStatus::Healthy
            }
        } else {
            DNSHealthStatus::Unchecked
        }
    }

    fn should_check_record(record_type: &RecordType) -> bool {
        match record_type {
            RecordType::A | RecordType::AAAA | RecordType::CNAME => true,
            _ => false,
        }
    }

    fn run_checks() {
        // Run HTTP checks
        DNSHealthHTTP::run();
    }
}

impl DNSHealthHTTP {
    fn run() {
        debug!("running dns health checks for the http protocol...");

        for domain in &APP_CONF.dns.health.http {
            Self::check_domain(domain);
        }

        debug!("ran dns health checks for the http protocol");
    }

    fn check_domain(domain: &ConfigDNSHealthHTTP) {
        for record_type in HEALTH_CHECK_RECORD_TYPES.iter() {
            debug!(
                "checking dns health for target: {} on zone: {} with type: {:?}",
                domain.name.to_str(),
                domain.zone.to_str(),
                record_type
            );

            if let Ok(record) = APP_STORE.get(&domain.zone, &domain.name, record_type) {
                let unique_values = record.list_record_values();

                for value in unique_values {
                    Self::check_domain_record(domain, value, 1);
                }
            }
        }
    }

    fn check_domain_record(domain: &ConfigDNSHealthHTTP, record_value: &RecordValue, attempt: u8) {
        // Generate request URL
        let request_url = Self::generate_request_url(
            &domain.zone,
            &domain.name,
            domain.port,
            &domain.path,
            domain.secure,
            &record_value,
        );

        if let (Ok(request_url), request_virtual_host) = request_url {
            debug!(
                "triggered a dns health check on target: {} on zone: {} with url: {}",
                domain.name.to_str(),
                domain.zone.to_str(),
                request_url.to_string()
            );

            // Acquire target host and port
            let target_host = request_url.host().unwrap();
            let target_port = request_url.port().unwrap();

            // Generate request
            let mut request = &mut RequestBuilder::new(&request_url);

            // Apply request method
            if domain.method == ConfigDNSHealthHTTPMethod::Head {
                request = request.method(Method::HEAD);
            } else {
                request = request.method(Method::GET);
            }

            // Apply request headers
            let mut headers = Headers::new();

            headers.insert("Host", &request_virtual_host);
            headers.insert("Connection", "Close");
            headers.insert("User-Agent", HEALTH_CHECK_PROBE_USERAGENT);

            request = request.headers(headers);

            // Create response body writer
            let mut response_body: Vec<u8> = Vec::new();

            // Dispatch request and acquire response
            let response = Self::dispatch_request(
                request,
                domain,
                &request_url,
                &request_virtual_host,
                &target_host,
                target_port,
                &mut response_body,
            );

            // Handle response
            match response {
                Ok(response_headers) => {
                    // Handle received status
                    Self::handle_domain_record_status(
                        domain,
                        record_value,
                        response_headers.status_code(),
                        response_body,
                    );
                }
                Err(_) => {
                    // Attempt once more?
                    if attempt < domain.max_attempts {
                        info!(
                            "dns health check error on target: {} on zone: {}, attempting again",
                            domain.name.to_str(),
                            domain.zone.to_str()
                        );

                        // Hold on a bit
                        thread::sleep(Duration::from_millis(500));

                        // Dispatch new attempt
                        Self::check_domain_record(domain, record_value, attempt + 1);
                    } else {
                        warn!(
                            "dns health check error on target: {} on zone: {}, stopping there",
                            domain.name.to_str(),
                            domain.zone.to_str()
                        );

                        // Assume a 'Service Unavailable' HTTP error
                        Self::handle_domain_record_status(
                            domain,
                            record_value,
                            HEALTH_CHECK_FAILED_STATUS,
                            response_body,
                        );
                    }
                }
            }
        } else {
            error!(
                "failed generating url for dns health check on http target: {} on zone: {}",
                domain.name.to_str(),
                domain.zone.to_str()
            );
        }
    }

    fn dispatch_request(
        request: &mut RequestBuilder,
        domain: &ConfigDNSHealthHTTP,
        request_url: &Uri,
        request_virtual_host: &str,
        target_host: &str,
        target_port: u16,
        response_body: &mut Vec<u8>,
    ) -> Result<Response, Error> {
        let timeout = Duration::from_secs(domain.timeout);

        // Open stream
        let mut stream = Self::connect_stream_timeout(target_host, target_port, timeout)?;

        // Apply request timeouts
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;

        // Dispatch secure request?
        if request_url.scheme() == "https" {
            // Create TLS stream to virtual host
            // Notice: as we may be connecting to a direct IP for health check purposes, we \
            //   still need to announce the corresponding virtual host so that proper TLS \
            //   certificate validation is done.
            let mut stream_secure = tls::Config::default().connect(request_virtual_host, stream)?;

            request.send(&mut stream_secure, response_body)
        } else {
            request.send(&mut stream, response_body)
        }
    }

    fn connect_stream_timeout(host: &str, port: u16, timeout: Duration) -> io::Result<TcpStream> {
        let addresses: Vec<_> = (host.as_ref(), port).to_socket_addrs()?.collect();
        let count = addresses.len();

        for (index, address) in addresses.into_iter().enumerate() {
            match TcpStream::connect_timeout(&address, timeout) {
                Ok(stream) => return Ok(stream),
                Err(err) => match err.kind() {
                    io::ErrorKind::TimedOut => return Err(err),
                    _ => {
                        if index + 1 == count {
                            return Err(err);
                        }
                    }
                },
            };
        }

        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "Could not resolve address",
        ))
    }

    fn handle_domain_record_status(
        domain: &ConfigDNSHealthHTTP,
        record_value: &RecordValue,
        status_code: StatusCode,
        body: Vec<u8>,
    ) {
        // Notice: there is unfortunately no other way around than doing clones there
        let record_key = (
            domain.zone.clone(),
            domain.name.clone(),
            record_value.clone(),
        );

        if status_code.is(|code| domain.expected_status.contains(&code))
            && Self::check_body_matches(domain, body)
        {
            debug!(
                "got healthy for dns health check on target: {} on zone: {} (status: {})",
                domain.name.to_str(),
                domain.zone.to_str(),
                status_code
            );

            // Consider record as healthy (remove from register)
            HEALTH_DEAD_REGISTER.write().unwrap().remove(&record_key);
        } else {
            warn!(
                "got dead for dns health check on target: {} on zone: {} (status: {})",
                domain.name.to_str(),
                domain.zone.to_str(),
                status_code
            );

            // Consider record as dead (add to register)
            HEALTH_DEAD_REGISTER.write().unwrap().insert(record_key);
        }
    }

    fn check_body_matches(domain: &ConfigDNSHealthHTTP, body: Vec<u8>) -> bool {
        if let Some(ref expected_body_list) = domain.expected_body {
            if expected_body_list.len() > 0 {
                // Convert response body to string
                if let Ok(body) = String::from_utf8(body) {
                    // Scan for a response body match with an expected body
                    for expected_body in expected_body_list {
                        if body.contains(expected_body) {
                            // Body matches (expected body configured)
                            return true;
                        }
                    }
                }

                // Default: does not match (expected body configured)
                return false;
            }
        }

        // Default: matches (no expected body configured)
        true
    }

    fn generate_request_url(
        zone: &ZoneName,
        name: &RecordName,
        port: u16,
        path: &str,
        secure: bool,
        value: &RecordValue,
    ) -> (Result<Uri, Error>, String) {
        // Format: [protocol]://[name].[zone]:[port]/[path]

        let mut request_url = String::new();

        // #1. Append 'protocol'
        if secure == true {
            request_url.push_str("https:");
        } else {
            request_url.push_str("http:");
        }

        // #2. Append URI delimiter
        request_url.push_str("//");

        // #3. Append target IP
        request_url.push_str(value.as_str());

        // #4. Append port
        request_url.push_str(&format!(":{}", port));

        // #5. Append path
        request_url.push_str(path);

        return (
            request_url.parse(),
            format!("{}{}", name.to_subdomain(), zone.to_str()),
        );
    }
}
