// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use redis::{Commands, RedisError};
use serde_json::{self, Error as SerdeJSONError};
use std::time::Duration;

use super::cache::STORE_CACHE;
use super::key::StoreKey;
use crate::dns::record::{RecordBlackhole, RecordName, RecordRegions, RecordType, RecordValues};
use crate::dns::zone::ZoneName;

use crate::APP_CONF;

static KEY_TYPE: &'static str = "t";
static KEY_NAME: &'static str = "n";
static KEY_TTL: &'static str = "e";
static KEY_BLACKHOLE: &'static str = "b";
static KEY_REGION: &'static str = "r";
static KEY_VALUE: &'static str = "v";

type StoreGetType = (String, String, u32, Option<String>, Option<String>, String);

pub struct StoreBuilder;

pub struct Store {
    pool: Pool<RedisConnectionManager>,
}

#[derive(Debug, Clone)]
pub struct StoreRecord {
    pub kind: RecordType,
    pub name: RecordName,
    pub ttl: Option<u32>,
    pub blackhole: Option<RecordBlackhole>,
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
            APP_CONF.redis.host, APP_CONF.redis.port
        );

        let addr_auth = match APP_CONF.redis.password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw = format!(
            "redis://{}{}:{}/{}",
            &addr_auth, APP_CONF.redis.host, APP_CONF.redis.port, APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let builder = Pool::builder()
                    .test_on_check_out(false)
                    .max_size(APP_CONF.redis.pool_size)
                    .max_lifetime(Some(Duration::from_secs(
                        APP_CONF.redis.max_lifetime_seconds,
                    )))
                    .idle_timeout(Some(Duration::from_secs(
                        APP_CONF.redis.idle_timeout_seconds,
                    )))
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
        let store_key = StoreKey::to_key(zone_name, record_name, record_type);

        // Check from local cache?
        if STORE_CACHE.has(&store_key) == true {
            return Ok(());
        }

        // Check from store
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.exists::<&str, bool>(&store_key)
            .map_err(|err| {
                StoreError::Connector(err)
            })
            .and_then(|exists| {
                if exists == true {
                    Ok(())
                } else {
                    // Store in local cache (no value)
                    STORE_CACHE.push(&store_key, None);

                    // Consider as not found
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
        let store_key = StoreKey::to_key(zone_name, record_name, record_type);

        // Get from local cache?
        if let Ok(cached_records) = STORE_CACHE.get(&store_key) {
            return match cached_records {
                Some(cached_records) => Ok(cached_records),
                None => Err(StoreError::NotFound),
            };
        }

        // Get from store
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            match client.hget::<_, _, StoreGetType>(
                &store_key,
                (KEY_TYPE, KEY_NAME, KEY_TTL, KEY_BLACKHOLE, KEY_REGION, KEY_VALUE),
            ) {
                Ok(values) => {
                    if let (Some(kind_value), Some(name_value), Ok(value_value)) = (
                        RecordType::from_str(&values.0),
                        RecordName::from_str(&values.1),
                        serde_json::from_str(&values.5)
                    ) {
                        let ttl = if values.2 > 0 {
                            Some(values.2)
                        } else {
                            None
                        };

                        let blackhole = values.3.and_then(|blackhole_raw| {
                            serde_json::from_str::<RecordBlackhole>(&blackhole_raw).ok()
                        });
                        let regions = values.4.and_then(|region_raw| {
                            serde_json::from_str::<RecordRegions>(&region_raw).ok()
                        });

                        debug!(
                            "read store record with kind: {:?}, name: {:?} and values: {:?}",
                            kind_value,
                            name_value,
                            value_value
                        );

                        if blackhole.is_some() == true {
                            debug!(
                                "store record with kind: {:?}, name: {:?} has blackhole: {:?}",
                                kind_value,
                                name_value,
                                blackhole
                            );
                        }
                        if regions.is_some() == true {
                            debug!(
                                "store record with kind: {:?}, name: {:?} has regions: {:?}",
                                kind_value,
                                name_value,
                                regions
                            );
                        }

                        let record = StoreRecord {
                            kind: kind_value,
                            name: name_value,
                            ttl: ttl,
                            blackhole: blackhole,
                            regions: regions,
                            values: value_value,
                        };

                        // Store in local cache
                        STORE_CACHE.push(&store_key, Some(record.clone()));

                        Ok(record)
                    } else {
                        Err(StoreError::Corrupted)
                    }
                },
                Err(_) => {
                    // Store in local cache (no value)
                    STORE_CACHE.push(&store_key, None);

                    // Consider as not found
                    Err(StoreError::NotFound)
                },
            }
        })
    }

    pub fn set(&self, zone_name: &ZoneName, record: StoreRecord) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            let blackhole_encoder = match record.blackhole {
                Some(ref blackhole) => {
                    if blackhole.has_items() == true {
                        serde_json::to_string(blackhole)
                    } else {
                        Ok("".to_owned())
                    }
                },
                None => Ok("".to_owned())
            };
            let region_encoder = match record.regions {
                Some(ref regions) => serde_json::to_string(regions),
                None => Ok("".to_owned())
            };

            match (serde_json::to_string(&record.values), blackhole_encoder, region_encoder) {
                (Ok(values), Ok(blackhole), Ok(regions)) => {
                    let store_key = StoreKey::to_key(zone_name, &record.name, &record.kind);

                    // Clean from local cache
                    STORE_CACHE.pop(&store_key);

                    // Store in remote
                    client.hset_multiple(
                        store_key, &[
                            (KEY_TYPE, record.kind.to_str()),
                            (KEY_NAME, record.name.to_str()),
                            (KEY_TTL, &record.ttl.unwrap_or(0).to_string()),
                            (KEY_BLACKHOLE, &blackhole),
                            (KEY_REGION, &regions),
                            (KEY_VALUE, &values),
                        ]
                    ).map_err(|err| {
                        StoreError::Connector(err)
                    })
                },
                (Err(err), _, _) | (_, Err(err), _) | (_, _, Err(err)) => {
                    Err(StoreError::Encoding(err))
                }
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
            let store_key = StoreKey::to_key(zone_name, record_name, record_type);

            // Clean from local cache
            STORE_CACHE.pop(&store_key);

            // Delete from remote
            client.del(store_key).map_err(|err| {
                StoreError::Connector(err)
            })
        })
    }
}
