// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::Duration;

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

            // TODO

            info!("ran dns health check operation");
        }
    }
}
