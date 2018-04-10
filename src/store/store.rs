// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use redis::{RedisError, Commands};

use super::key::StoreKey;
use dns::record::{RecordType, RecordName};

use APP_CONF;

static KEY_TYPE: &'static str = "t";
static KEY_NAME: &'static str = "n";
static KEY_TTL: &'static str = "e";
static KEY_VALUE: &'static str = "v";

pub struct StoreBuilder;

pub struct Store {
    pool: Pool<RedisConnectionManager>,
}

pub struct StoreRecord {
    pub kind: RecordType,
    pub name: RecordName,
    pub ttl: u32,
    pub value: RecordName,
}

pub enum StoreError {
    Corrupted,
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
        record_name: RecordName,
        record_type: RecordType,
    ) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.exists::<&str, bool>(&StoreKey::to_key(&record_name, &record_type))
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
        record_name: RecordName,
        record_type: RecordType,
    ) -> Result<StoreRecord, StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            match client.hget::<_, _, (String, String, u32, String)>(
                StoreKey::to_key(&record_name, &record_type),
                (KEY_TYPE, KEY_NAME, KEY_TTL, KEY_VALUE),
            ) {
                Ok(values) => {
                    if let (Some(kind_value), Some(name_value), Some(value_value)) = (
                        RecordType::from_str(&values.0),
                        RecordName::from_str(&values.1),
                        RecordName::from_str(&values.3)
                    ) {
                        Ok(StoreRecord {
                            kind: kind_value,
                            name: name_value,
                            ttl: values.2,
                            value: value_value,
                        })
                    } else {
                        Err(StoreError::Corrupted)
                    }
                },
                Err(err) => Err(StoreError::Connector(err)),
            }
        })
    }

    pub fn set(&self, record: StoreRecord) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.hset_multiple(
                StoreKey::to_key(&record.name, &record.kind), &[
                    (KEY_TYPE, record.kind.to_str()),
                    (KEY_NAME, record.name.to_str()),
                    (KEY_TTL, &record.ttl.to_string()),
                    (KEY_VALUE, record.value.to_str()),
                ]
            ).map_err(|err| {
                StoreError::Connector(err)
            })
        })
    }

    pub fn remove(
        &self,
        record_name: RecordName,
        record_type: RecordType,
    ) -> Result<(), StoreError> {
        get_cache_store_client!(self.pool, StoreError::Disconnected, client {
            client.del(StoreKey::to_key(&record_name, &record_type)).map_err(|err| {
                StoreError::Connector(err)
            })
        })
    }
}
