// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::Duration;

use crate::config::config::ConfigDNSHealthHTTP;
use crate::APP_CONF;

pub struct DNSHealthBuilder;
pub struct DNSHealth;

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
            // Hold for next check run
            thread::sleep(interval_duration);

            info!("running a dns health check operation...");

            // Hold on 1 second
            thread::sleep(Duration::from_secs(1));

            Self::run_checks();

            info!("ran dns health check operation");
        }
    }

    fn run_checks() {
        // Run HTTP checks
        Self::run_checks_http();
    }

    fn run_checks_http() {
        debug!("running dns health checks for the http protocol...");

        for check_domain in &APP_CONF.dns.health.http {
            Self::run_check_http_domain(check_domain);
        }

        debug!("ran dns health checks for the http protocol");
    }

    fn run_check_http_domain(check_domain: &ConfigDNSHealthHTTP) {
        debug!(
            "triggered a dns health check on http target: {} on zone: {}",
            check_domain.name.to_str(),
            check_domain.zone.to_str()
        );

        // TODO
    }
}
