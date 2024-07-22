// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;
use std::net::SocketAddr;

use super::config::{
    ConfigDNSHealth, ConfigDNSHealthHTTP, ConfigDNSHealthHTTPMethod, ConfigDNSHealthNotify,
    ConfigDNSZone, ConfigRedisMaster,
};

pub fn server_log_level() -> String {
    "error".to_string()
}

pub fn server_identifier() -> String {
    "constellation/0".to_string()
}

pub fn dns_inets() -> Vec<SocketAddr> {
    vec!["0.0.0.0:53".parse().unwrap(), "[::]:53".parse().unwrap()]
}

pub fn dns_tcp_timeout() -> u64 {
    2
}

pub fn dns_soa_refresh() -> i32 {
    10000
}

pub fn dns_soa_retry() -> i32 {
    2400
}

pub fn dns_soa_expire() -> i32 {
    604800
}

pub fn dns_soa_ttl() -> u32 {
    3600
}

pub fn dns_record_ttl() -> u32 {
    3600
}

pub fn dns_zone() -> BTreeMap<String, ConfigDNSZone> {
    BTreeMap::new()
}

pub fn dns_health() -> ConfigDNSHealth {
    ConfigDNSHealth {
        check_enable: dns_health_check_enable(),
        check_interval: dns_health_check_interval(),
        notify: dns_health_notify(),
        http: Vec::new(),
    }
}

pub fn dns_health_check_enable() -> bool {
    false
}

pub fn dns_health_check_interval() -> u64 {
    60
}

pub fn dns_health_notify() -> ConfigDNSHealthNotify {
    ConfigDNSHealthNotify {
        slack_hook_url: None,
    }
}

pub fn dns_health_http() -> Vec<ConfigDNSHealthHTTP> {
    Vec::new()
}

pub fn dns_health_http_method() -> ConfigDNSHealthHTTPMethod {
    ConfigDNSHealthHTTPMethod::Get
}

pub fn dns_health_http_path() -> String {
    "/".to_string()
}

pub fn dns_health_http_port() -> u16 {
    443
}

pub fn dns_health_http_secure() -> bool {
    true
}

pub fn dns_health_http_timeout() -> u64 {
    10
}

pub fn dns_health_http_max_attempts() -> u8 {
    3
}

pub fn dns_health_http_expected_status() -> Vec<u16> {
    vec![200]
}

pub fn geo_database_path() -> String {
    "./res/geo/".to_string()
}

pub fn geo_database_file() -> String {
    "GeoLite2-Country.mmdb".to_string()
}

pub fn geo_update_enable() -> bool {
    false
}

pub fn geo_update_interval() -> u64 {
    864000
}

pub fn http_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn http_workers() -> usize {
    2
}

pub fn redis_database() -> u8 {
    0
}

pub fn redis_pool_size() -> u32 {
    8
}

pub fn redis_max_lifetime_seconds() -> u64 {
    20
}

pub fn redis_idle_timeout_seconds() -> u64 {
    600
}

pub fn redis_connection_timeout_seconds() -> u64 {
    2
}

pub fn redis_delinquency_seconds() -> u64 {
    10
}

pub fn redis_cache_refresh_seconds() -> u64 {
    60
}

pub fn redis_cache_expire_seconds() -> u64 {
    600
}

pub fn redis_master() -> ConfigRedisMaster {
    ConfigRedisMaster {
        host: redis_master_host(),
        port: redis_master_port(),
        password: None,
    }
}

pub fn redis_master_host() -> String {
    "localhost".to_string()
}

pub fn redis_master_port() -> u16 {
    6379
}

pub fn redis_rescue_port() -> u16 {
    6379
}
