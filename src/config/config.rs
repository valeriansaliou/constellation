// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::collections::BTreeMap;

use super::defaults;

#[derive(Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub dns: ConfigDNS,
    pub http: ConfigHTTP,
    pub redis: ConfigRedis,
}

#[derive(Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_log_level")]
    pub log_level: String,
}

#[derive(Deserialize)]
pub struct ConfigDNS {
    #[serde(default = "defaults::dns_inets")]
    pub inets: Vec<SocketAddr>,

    #[serde(default = "defaults::dns_tcp_timeout")]
    pub tcp_timeout: u64,

    #[serde(default = "defaults::dns_zone")]
    pub zone: BTreeMap<String, ConfigDNSZone>,
}

#[derive(Deserialize)]
pub struct ConfigDNSZone {}

#[derive(Deserialize)]
pub struct ConfigHTTP {
    #[serde(default = "defaults::http_inet")]
    pub inet: SocketAddr,

    #[serde(default = "defaults::http_workers")]
    pub workers: u16,

    pub record_token: String,
}

#[derive(Deserialize)]
pub struct ConfigRedis {
    #[serde(default = "defaults::redis_host")]
    pub host: String,

    #[serde(default = "defaults::redis_port")]
    pub port: u16,

    pub password: Option<String>,

    #[serde(default = "defaults::redis_database")]
    pub database: u8,

    #[serde(default = "defaults::redis_pool_size")]
    pub pool_size: u32,

    #[serde(default = "defaults::redis_max_lifetime_seconds")]
    pub max_lifetime_seconds: u64,

    #[serde(default = "defaults::redis_idle_timeout_seconds")]
    pub idle_timeout_seconds: u64,

    #[serde(default = "defaults::redis_connection_timeout_seconds")]
    pub connection_timeout_seconds: u64,
}

impl ConfigDNS {
    pub fn zone_exists(&self, name: &str) -> bool {
        self.zone.contains_key(name)
    }
}
