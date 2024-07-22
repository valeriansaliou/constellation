// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::BTreeMap;
use std::net::SocketAddr;
use url_serde::SerdeUrl;

use super::defaults;
use crate::dns::record::RecordName;
use crate::dns::zone::ZoneName;

#[derive(Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub dns: ConfigDNS,
    pub geo: ConfigGeo,
    pub http: ConfigHTTP,
    pub redis: ConfigRedis,
}

#[derive(Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_log_level")]
    pub log_level: String,

    #[serde(default = "defaults::server_identifier")]
    pub identifier: String,
}

#[derive(Deserialize)]
pub struct ConfigDNS {
    #[serde(default = "defaults::dns_inets")]
    pub inets: Vec<SocketAddr>,

    #[serde(default = "defaults::dns_tcp_timeout")]
    pub tcp_timeout: u64,

    pub nameservers: Vec<String>,

    pub soa_master: String,
    pub soa_responsible: String,

    #[serde(default = "defaults::dns_soa_refresh")]
    pub soa_refresh: i32,

    #[serde(default = "defaults::dns_soa_retry")]
    pub soa_retry: i32,

    #[serde(default = "defaults::dns_soa_expire")]
    pub soa_expire: i32,

    #[serde(default = "defaults::dns_soa_ttl")]
    pub soa_ttl: u32,

    #[serde(default = "defaults::dns_record_ttl")]
    pub record_ttl: u32,

    #[serde(default = "defaults::dns_zone")]
    pub zone: BTreeMap<String, ConfigDNSZone>,

    pub flatten: ConfigDNSFlatten,

    #[serde(default = "defaults::dns_health")]
    pub health: ConfigDNSHealth,
}

#[derive(Deserialize)]
pub struct ConfigDNSZone {}

#[derive(Default, Deserialize)]
pub struct ConfigDNSFlatten {
    pub resolvers: Vec<String>,
}

#[derive(Default, Deserialize)]
pub struct ConfigDNSHealth {
    #[serde(default = "defaults::dns_health_check_enable")]
    pub check_enable: bool,

    #[serde(default = "defaults::dns_health_check_interval")]
    pub check_interval: u64,

    #[serde(default = "defaults::dns_health_notify")]
    pub notify: ConfigDNSHealthNotify,

    #[serde(default = "defaults::dns_health_http")]
    pub http: Vec<ConfigDNSHealthHTTP>,
}

#[derive(Default, Deserialize)]
pub struct ConfigDNSHealthNotify {
    pub slack_hook_url: Option<SerdeUrl>,
}

#[derive(Deserialize)]
pub struct ConfigDNSHealthHTTP {
    pub zone: ZoneName,
    pub name: RecordName,

    #[serde(default = "defaults::dns_health_http_method")]
    pub method: ConfigDNSHealthHTTPMethod,

    pub host: Option<String>,

    #[serde(default = "defaults::dns_health_http_path")]
    pub path: String,

    #[serde(default = "defaults::dns_health_http_port")]
    pub port: u16,

    #[serde(default = "defaults::dns_health_http_secure")]
    pub secure: bool,

    #[serde(default = "defaults::dns_health_http_timeout")]
    pub timeout: u64,

    #[serde(default = "defaults::dns_health_http_max_attempts")]
    pub max_attempts: u8,

    #[serde(default = "defaults::dns_health_http_expected_status")]
    pub expected_status: Vec<u16>,

    pub expected_body: Option<Vec<String>>,
}

#[derive(Deserialize, PartialEq)]
pub enum ConfigDNSHealthHTTPMethod {
    #[serde(rename = "HEAD")]
    Head,

    #[serde(rename = "GET")]
    Get,
}

#[derive(Deserialize)]
pub struct ConfigGeo {
    #[serde(default = "defaults::geo_database_path")]
    pub database_path: String,

    #[serde(default = "defaults::geo_database_file")]
    pub database_file: String,

    #[serde(default = "defaults::geo_update_enable")]
    pub update_enable: bool,

    #[serde(default = "defaults::geo_update_interval")]
    pub update_interval: u64,

    pub update_url: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigHTTP {
    #[serde(default = "defaults::http_inet")]
    pub inet: SocketAddr,

    #[serde(default = "defaults::http_workers")]
    pub workers: usize,

    pub record_token: String,
}

#[derive(Deserialize)]
pub struct ConfigRedis {
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

    #[serde(default = "defaults::redis_delinquency_seconds")]
    pub delinquency_seconds: u64,

    #[serde(default = "defaults::redis_cache_refresh_seconds")]
    pub cache_refresh_seconds: u64,

    #[serde(default = "defaults::redis_cache_expire_seconds")]
    pub cache_expire_seconds: u64,

    #[serde(default = "defaults::redis_master")]
    pub master: ConfigRedisMaster,

    pub rescue: Option<Vec<ConfigRedisRescue>>,
}

#[derive(Default, Deserialize)]
pub struct ConfigRedisMaster {
    #[serde(default = "defaults::redis_master_host")]
    pub host: String,

    #[serde(default = "defaults::redis_master_port")]
    pub port: u16,

    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigRedisRescue {
    pub host: String,

    #[serde(default = "defaults::redis_rescue_port")]
    pub port: u16,

    pub password: Option<String>,
}

impl ConfigDNS {
    pub fn zone_exists(&self, name: &str) -> bool {
        self.zone.contains_key(name)
    }
}
