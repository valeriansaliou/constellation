// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub struct DNSListenBuilder;
pub struct DNSListen;

use std::thread;
use std::time::Duration;

impl DNSListenBuilder {
    pub fn new() -> DNSListen {
        DNSListen {}
    }
}

impl DNSListen {
    pub fn run(&self) {
        // TODO
        thread::sleep(Duration::from_secs(120));
    }
}
