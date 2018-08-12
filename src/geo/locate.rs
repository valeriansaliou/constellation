// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::IpAddr;

use super::country::CountryCode;
use super::region::RegionCode;

pub struct Locator;

impl Locator {
    pub fn ip_to_region(ip: &IpAddr) -> Option<(CountryCode, RegionCode)> {
        // TODO: geo-ip resolve and map country + region
        // TODO: ensure both IPv4 + IPv6 work

        None
    }
}
