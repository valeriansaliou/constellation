// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use redis::{RedisError, Commands};
use serde_json::{self, Error as SerdeJSONError};

use super::key::StoreKey;
use dns::zone::ZoneName;
use dns::record::{RecordType, RecordName, RecordRegions, RecordValues};

use APP_CONF;

static KEY_TYPE: &'static str = "t";
static KEY_NAME: &'static str = "n";
static KEY_TTL: &'static str = "e";
static KEY_REGION: &'static str = "r";
static KEY_VALUE: &'static str = "v";

pub struct StoreBuilder;

pub struct Store {
    pool: Pool<RedisConnectionManager>,
}

#[derive(Debug)]
pub struct StoreRecord {
    pub kind: RecordType,
    pub name: RecordName,
    pub ttl: Option<u32>,
    pub regions: Option<RecordRegions>,
    pub values: RecordValues,
}

pub enum StoreError {
    Corrupted,
    Encoding(SerdeJSONError),
    Connector(RedisError),
    NotFound,
    Disconnected,
}

impl StoreBuilder {
    pub fn new() -> Store {
        info!(
            "binding to store backend at {}:{}",
            APP_CONF.redis.host,
            APP_CONF.redis.port
        );

        let addr_auth = match APP_CONF.redis.password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw =
            format!(
            "redis://{}{}:{}/{}",
            &addr_auth,
            APP_CONF.redis.host,
            APP_CONF.redis.port,
            APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let builder = Pool::builder()
                    .test_on_check_out(false)
                    .max_size(APP_CONF.redis.pool_size)
                    .max_lifetime(Some(
                        Duration::from_secs(APP_CONF.redis.max_lifetime_seconds),
                    ))
                    .idle_timeout(Some(
                        Duration::from_secs(APP_CONF.redis.idle_timeout_seconds),
                    ))
                    .connection_timeout(Duration::from_secs(
                        APP_CONF.redis.connection_timeout_seconds,
                    ));

                match builder.build(manager) {
                    Ok(pool) => {
                        info!("bound to store backend");

                        Store { pool: pool }
                    }
                    Err(_) => panic!("could not spawn redis pool"),
                }
            }
            Err(_) => panic!("could not create redis connection manager"),
        }
    }
}

impl Store {
    pub fn check(
        &self,
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
    ) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.exists::<&str, bool>(&StoreKey::to_key(zone_name, record_name, record_type))
            .map_err(|err| {
                StoreError::Connector(err)
            })
            .and_then(|exists| {
                if exists == true {
                    Ok(())
                } else {
                    Err(StoreError::NotFound)
                }
            })
        })
    }

    pub fn get(
        &self,
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
    ) -> Result<StoreRecord, StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            match client.hget::<_, _, (String, String, u32, Option<String>, String)>(
                StoreKey::to_key(zone_name, record_name, record_type),
                (KEY_TYPE, KEY_NAME, KEY_TTL, KEY_REGION, KEY_VALUE),
            ) {
                Ok(values) => {
                    if let (Some(kind_value), Some(name_value), Ok(value_value)) = (
                        RecordType::from_str(&values.0),
                        RecordName::from_str(&values.1),
                        serde_json::from_str(&values.4)
                    ) {
                        let ttl = if values.2 > 0 {
                            Some(values.2)
                        } else {
                            None
                        };

                        let regions = values.3.and_then(|region_raw| {
                            serde_json::from_str::<RecordRegions>(&region_raw).ok()
                        });

                        debug!(
                            "read store record with kind: {:?}, name: {:?} and values: {:?}",
                            kind_value,
                            name_value,
                            value_value
                        );

                        if regions.is_some() == true {
                            debug!(
                                "store record with kind: {:?}, name: {:?} has regions: {:?}",
                                kind_value,
                                name_value,
                                regions
                            );
                        }

                        Ok(StoreRecord {
                            kind: kind_value,
                            name: name_value,
                            ttl: ttl,
                            regions: regions,
                            values: value_value,
                        })
                    } else {
                        Err(StoreError::Corrupted)
                    }
                },
                Err(err) => Err(StoreError::Connector(err)),
            }
        })
    }

    pub fn set(&self, zone_name: &ZoneName, record: StoreRecord) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            let region_encoder = match record.regions {
                Some(ref regions) => serde_json::to_string(regions),
                None => Ok("".to_owned())
            };

            match (serde_json::to_string(&record.values), region_encoder) {
                (Ok(values), Ok(regions)) => {
                    client.hset_multiple(
                        StoreKey::to_key(zone_name, &record.name, &record.kind), &[
                            (KEY_TYPE, record.kind.to_str()),
                            (KEY_NAME, record.name.to_str()),
                            (KEY_TTL, &record.ttl.unwrap_or(0).to_string()),
                            (KEY_REGION, &regions),
                            (KEY_VALUE, &values),
                        ]
                    ).map_err(|err| {
                        StoreError::Connector(err)
                    })
                },
                (Err(err), _) | (_, Err(err)) => Err(StoreError::Encoding(err))
            }
        })
    }

    pub fn remove(
        &self,
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
    ) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.del(StoreKey::to_key(zone_name, record_name, record_type)).map_err(|err| {
                StoreError::Connector(err)
            })
        })
    }
}
