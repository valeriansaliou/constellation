// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub struct HTTPListenBuilder;
pub struct HTTPListen;

impl HTTPListenBuilder {
    pub fn new() -> HTTPListen {
        HTTPListen {}
    }
}

impl HTTPListen {
    pub fn run(&self) {
        // TODO
    }
}
