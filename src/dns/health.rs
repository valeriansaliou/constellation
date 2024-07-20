// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http_req::error::Error;
use http_req::request::{Method, Request, RequestBuilder};
use http_req::response::{Headers, Response, StatusCode};
use http_req::tls;
use http_req::uri::Uri;
use serde_json;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::config::config::{ConfigDNSHealthHTTP, ConfigDNSHealthHTTPMethod};
use crate::dns::record::{RecordName, RecordType, RecordValue};
use crate::dns::zone::ZoneName;
use crate::store::store::StoreAccessOrigin;
use crate::APP_CONF;
use crate::APP_STORE;

lazy_static! {
    static ref HEALTH_DEAD_REGISTER: RwLock<HashSet<(ZoneName, RecordName, RecordValue)>> =
        RwLock::new(HashSet::new());
}

const HEALTH_CHECK_RECORD_TYPES: [RecordType; 3] =
    [RecordType::A, RecordType::AAAA, RecordType::CNAME];
const HEALTH_CHECK_FAILED_STATUS: StatusCode = StatusCode::new(503);

const HEALTH_CHECK_PROBE_USERAGENT: &'static str = "constellation (health-check-probe)";
const HEALTH_CHECK_NOTIFY_USERAGENT: &'static str = "constellation (health-check-notify)";
const HEALTH_CHECK_NOTIFY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct DNSHealthBuilder;
pub struct DNSHealth;

struct DNSHealthHTTP;

struct DNSHealthNotify {
    events: Vec<(
        ZoneName,
        RecordName,
        RecordValue,
        DNSHealthStatus,
        Option<String>,
    )>,
}

#[derive(Serialize)]
struct DNSHealthNotifySlackPayload<'a> {
    text: String,
    attachments: Vec<DNSHealthNotifySlackPayloadAttachment<'a>>,
}

#[derive(Serialize)]
struct DNSHealthNotifySlackPayloadAttachment<'a> {
    fallback: String,
    color: &'a str,
    fields: Vec<DNSHealthNotifySlackPayloadAttachmentField>,
}

#[derive(Serialize)]
struct DNSHealthNotifySlackPayloadAttachmentField {
    title: String,
    value: String,
    short: bool,
}

#[derive(PartialEq, Debug)]
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
            if Self::register_has(&(zone_name.clone(), record_name.clone(), record_value.clone())) {
                DNSHealthStatus::Dead
            } else {
                DNSHealthStatus::Healthy
            }
        } else {
            DNSHealthStatus::Unchecked
        }
    }

    pub fn register_has(chain: &(ZoneName, RecordName, RecordValue)) -> bool {
        HEALTH_DEAD_REGISTER.read().unwrap().contains(chain)
    }

    fn should_check_record(record_type: &RecordType) -> bool {
        match record_type {
            RecordType::A | RecordType::AAAA | RecordType::CNAME => true,
            _ => false,
        }
    }

    fn run_checks() {
        // Build shared notifier
        let mut notifier = DNSHealthNotify::new();

        // Run HTTP checks
        DNSHealthHTTP::run(&mut notifier);

        // Dispatch notifier (if any notification to be processed)
        notifier.dispatch();
    }
}

impl DNSHealthHTTP {
    #[tokio::main]
    async fn run(notifier: &mut DNSHealthNotify) {
        debug!("running dns health checks for the http protocol...");

        for domain in &APP_CONF.dns.health.http {
            Self::check_domain(domain, notifier).await;
        }

        debug!("ran dns health checks for the http protocol");
    }

    async fn check_domain(domain: &ConfigDNSHealthHTTP, notifier: &mut DNSHealthNotify) {
        for record_type in HEALTH_CHECK_RECORD_TYPES.iter() {
            debug!(
                "checking dns health for target: {} on zone: {} with type: {:?}",
                domain.name.to_str(),
                domain.zone.to_str(),
                record_type
            );

            if let Ok(record) = APP_STORE
                .get(
                    &domain.zone,
                    &domain.name,
                    record_type,
                    StoreAccessOrigin::Internal,
                )
                .await
            {
                let unique_values = record.list_record_values();

                for record_value in unique_values {
                    Self::check_domain_record(domain, record_type, record_value, notifier, 1);
                }
            }
        }
    }

    fn check_domain_record(
        domain: &ConfigDNSHealthHTTP,
        record_type: &RecordType,
        record_value: &RecordValue,
        notifier: &mut DNSHealthNotify,
        attempt: u8,
    ) {
        // Generate request URL
        let (request_url, request_virtual_host) = Self::generate_request_url(
            &domain.zone,
            &domain.name,
            domain.port,
            &domain.host,
            &domain.path,
            domain.secure,
            record_type,
            record_value,
        );

        if let Ok(request_url) = Uri::try_from(request_url.as_str()) {
            debug!(
                "triggered a dns health check on target: {} on zone: {} with url: {} on host: {}",
                domain.name.to_str(),
                domain.zone.to_str(),
                request_url.to_string(),
                request_virtual_host
            );

            // Acquire target host and port (extracted inner host)
            let target_host = Self::extract_inner_host(record_type, request_url.host().unwrap());
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

            headers.insert("Host", &format!("{}:{}", request_virtual_host, target_port));
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

            // Handle response status code
            let status_code = if let Ok(response_headers) = response {
                // HTTP response received, acquire HTTP status code
                response_headers.status_code()
            } else {
                // Assume a 'Service Unavailable' HTTP error
                HEALTH_CHECK_FAILED_STATUS
            };

            // Check for expected status and body
            let is_success = status_code.is(|code| domain.expected_status.contains(&code))
                && Self::check_body_matches(domain, response_body);

            // This check has failed expectations
            if is_success == false {
                // Attempt once more?
                if attempt < domain.max_attempts {
                    info!(
                        "dns health check error on target: {} on zone: {}, attempting again",
                        domain.name.to_str(),
                        domain.zone.to_str()
                    );

                    // Hold on a bit
                    thread::sleep(Duration::from_secs(1));

                    // Dispatch new attempt
                    Self::check_domain_record(
                        domain,
                        record_type,
                        record_value,
                        notifier,
                        attempt + 1,
                    );
                } else {
                    warn!(
                        "dns health check error on target: {} on zone: {}, stopping there",
                        domain.name.to_str(),
                        domain.zone.to_str()
                    );

                    // Handle final failure
                    Self::handle_domain_record_status(
                        domain,
                        record_value,
                        status_code,
                        notifier,
                        false,
                    );
                }
            } else {
                // Handle final success
                Self::handle_domain_record_status(
                    domain,
                    record_value,
                    status_code,
                    notifier,
                    true,
                );
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
        notifier: &mut DNSHealthNotify,
        is_success: bool,
    ) {
        // Notice: there is unfortunately no other way around than doing clones there
        let record_key = (
            domain.zone.clone(),
            domain.name.clone(),
            record_value.clone(),
        );

        if is_success == true {
            debug!(
                "got healthy for dns health check on target: {} on zone: {} (status: {})",
                domain.name.to_str(),
                domain.zone.to_str(),
                status_code
            );

            // Dispatch 'healthy' notification? (record key was set as 'dead')
            if DNSHealth::register_has(&record_key) {
                notifier.stack(
                    &domain.zone,
                    &domain.name,
                    record_value,
                    DNSHealthStatus::Healthy,
                    None,
                );
            }

            // Consider record as healthy (remove from register)
            HEALTH_DEAD_REGISTER.write().unwrap().remove(&record_key);
        } else {
            warn!(
                "got dead for dns health check on target: {} on zone: {} (status: {})",
                domain.name.to_str(),
                domain.zone.to_str(),
                status_code
            );

            // Dispatch 'dead' notification? (record key was set as 'healthy')
            if !DNSHealth::register_has(&record_key) {
                notifier.stack(
                    &domain.zone,
                    &domain.name,
                    record_value,
                    DNSHealthStatus::Dead,
                    Some(format!("Got HTTP status: {} or invalid body", status_code)),
                );
            }

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

    fn generate_request_url<'a>(
        zone: &ZoneName,
        name: &RecordName,
        port: u16,
        host: &Option<String>,
        path: &str,
        secure: bool,
        kind: &RecordType,
        value: &'a RecordValue,
    ) -> (String, String) {
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
        if kind == &RecordType::AAAA {
            request_url.push_str("[");
        }

        request_url.push_str(value.as_str());

        if kind == &RecordType::AAAA {
            request_url.push_str("]");
        }

        // #4. Append port
        request_url.push_str(&format!(":{}", port));

        // #5. Append path
        request_url.push_str(path);

        // Generate virtual host
        let virtual_host = if let Some(host) = host {
            host.to_owned()
        } else {
            format!("{}{}", name.to_subdomain(), zone.to_str())
        };

        return (request_url, virtual_host);
    }

    fn extract_inner_host<'a>(record_type: &RecordType, outer_host: &'a str) -> &'a str {
        // IPv6 are formatted as `[::]`; which needs to be transformed to inner `::`
        if record_type == &RecordType::AAAA && outer_host.len() > 2 {
            &outer_host[1..(outer_host.len() - 1)]
        } else {
            outer_host
        }
    }
}

impl DNSHealthNotify {
    fn new() -> DNSHealthNotify {
        DNSHealthNotify { events: Vec::new() }
    }

    fn dispatch(&self) {
        if self.events.len() > 0 {
            info!("should dispatch notifications for dns health check");

            // Dispatch on each configured channel
            self.dispatch_slack();
        } else {
            debug!("no notifications to dispatch for dns health check");
        }
    }

    fn dispatch_slack(&self) {
        if let Some(ref slack_hook_url) = APP_CONF.dns.health.notify.slack_hook_url {
            debug!("will dispatch notification to slack hooks for dns health check");

            // Build paylaod
            let message_text = String::from("DNS health check probe events occured.");

            let mut payload = DNSHealthNotifySlackPayload {
                text: message_text.to_owned(),
                attachments: Vec::new(),
            };

            let mut attachment = DNSHealthNotifySlackPayloadAttachment {
                fallback: message_text,
                color: "warning",
                fields: Vec::new(),
            };

            // Reveal origin replica identifier
            let replica_origin = format!(" Replica: *{}*.", APP_CONF.server.identifier);

            payload.text.push_str(&replica_origin);
            attachment.fallback.push_str(&replica_origin);

            // Append attachment fields
            for event in &self.events {
                attachment
                    .fields
                    .push(DNSHealthNotifySlackPayloadAttachmentField {
                        title: format!("{}{}", event.1.to_subdomain(), event.0.to_str()),
                        value: if event.3 == DNSHealthStatus::Healthy {
                            format!("✅ {}", event.2.to_str())
                        } else {
                            let mut dead_value = format!("🆘 {}", event.2.to_str());

                            if let Some(ref dead_reason) = event.4 {
                                dead_value.push_str(" `");
                                dead_value.push_str(dead_reason);
                                dead_value.push_str("`");
                            }

                            dead_value
                        },
                        short: false,
                    });
            }

            // Append attachment
            payload.attachments.push(attachment);

            // Encore payload to string
            // Notice: fail hard if payload is invalid (it should never be)
            let payload_json = serde_json::to_vec(&payload).expect("invalid slack hooks payload");

            // Submit payload to Slack
            // Notice: fail hard if Slack Hooks URI is invalid
            let (slack_hook_url_raw, mut response_sink) = (slack_hook_url.as_str(), io::sink());

            let response =
                Request::new(&Uri::try_from(slack_hook_url_raw).expect("invalid slack hooks uri"))
                    .connect_timeout(Some(HEALTH_CHECK_NOTIFY_TIMEOUT))
                    .read_timeout(Some(HEALTH_CHECK_NOTIFY_TIMEOUT))
                    .write_timeout(Some(HEALTH_CHECK_NOTIFY_TIMEOUT))
                    .method(Method::POST)
                    .header("User-Agent", HEALTH_CHECK_NOTIFY_USERAGENT)
                    .header("Content-Type", "application/json")
                    .header("Content-Length", &payload_json.len())
                    .body(&payload_json)
                    .send(&mut response_sink);

            match response {
                Ok(response) => {
                    if response.status_code().is_success() {
                        info!("dispatched notification to slack hooks for dns health check");
                    } else {
                        error!(
                            "got invalid status in slack hooks for dns health check: {}",
                            response.status_code()
                        );
                    }
                }
                Err(err) => {
                    error!(
                        "notification dispatch to slack hooks for dns health check failed: {}",
                        err
                    );
                }
            }
        }
    }

    fn stack(
        &mut self,
        zone: &ZoneName,
        name: &RecordName,
        value: &RecordValue,
        health_status: DNSHealthStatus,
        reason: Option<String>,
    ) {
        debug!(
            "stacked {:?} notification for dns health check for chain: {:?} / {:?} / {:?}",
            zone, name, value, health_status
        );

        // Notice: there is unfortunately no other way around than doing clones there
        self.events.push((
            zone.clone(),
            name.clone(),
            value.clone(),
            health_status,
            reason,
        ));
    }
}
