// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use bb8_redis::bb8::Pool;
use bb8_redis::redis::{AsyncCommands, ErrorKind};
use bb8_redis::RedisConnectionManager;
use serde_json::{self, Error as SerdeJSONError};
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime};

use super::cache::STORE_CACHE;
use super::key::StoreKey;
use crate::dns::record::{
    RecordBlackhole, RecordName, RecordRegions, RecordType, RecordValue, RecordValues,
};
use crate::dns::zone::ZoneName;

use crate::APP_CONF;

static KEY_TYPE: &'static str = "t";
static KEY_NAME: &'static str = "n";
static KEY_TTL: &'static str = "e";
static KEY_FLATTEN: &'static str = "m"; // Alias for 'minify'
static KEY_BLACKHOLE: &'static str = "b";
static KEY_REGION: &'static str = "r";
static KEY_RESCUE: &'static str = "f"; // Alias for 'failover'
static KEY_VALUE: &'static str = "v";

const LIMITS_GET_REMOTE_TIMESPAN_TOTAL: Duration = Duration::from_secs(10);
const LIMITS_GET_REMOTE_ALLOWANCE_THRESHOLD: Duration = Duration::from_secs(8);

type StoreGetType = (
    String,
    String,
    u32,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
);

pub struct StoreBuilder;

pub struct Store {
    pools: Vec<StorePool>,
    limits: StoreLimits,
}

pub struct StorePool {
    connection: Pool<RedisConnectionManager>,
    target: String,
    delinquent_until: RwLock<Option<Instant>>,
}

pub struct StoreLimits {
    rate: RwLock<StoreLimitsRate>,
}

pub struct StoreLimitsRate {
    time_last: Instant,
    time_spent: Duration,
}

#[derive(Debug, Clone)]
pub struct StoreRecord {
    pub kind: RecordType,
    pub name: RecordName,
    pub ttl: Option<u32>,
    pub flatten: Option<bool>,
    pub blackhole: Option<RecordBlackhole>,
    pub regions: Option<RecordRegions>,
    pub rescue: Option<RecordValues>,
    pub values: RecordValues,
}

pub enum StoreError {
    Corrupted,
    Encoding,
    Connector,
    NotFound,
    Disconnected,
}

#[derive(PartialEq, Clone, Copy)]
pub enum StoreAccessOrigin {
    External,
    Internal,
}

impl StoreBuilder {
    #[tokio::main]
    pub async fn new() -> Store {
        let mut pools = Vec::new();

        // Bind to master pool
        Self::pool_bind(
            &mut pools,
            &APP_CONF.redis.master.host,
            APP_CONF.redis.master.port,
            &APP_CONF.redis.master.password,
        )
        .await;

        // Bind to rescue pools (if any)
        if let Some(ref rescue_items) = APP_CONF.redis.rescue {
            for rescue in rescue_items {
                Self::pool_bind(&mut pools, &rescue.host, rescue.port, &rescue.password).await;
            }
        }

        // Initialize limits
        let limits = StoreLimits {
            rate: RwLock::new(StoreLimitsRate::default()),
        };

        Store { pools, limits }
    }

    async fn pool_bind(
        pools: &mut Vec<StorePool>,
        host: &str,
        port: u16,
        password: &Option<String>,
    ) {
        // Establish pool connection for this Redis target
        match Self::pool_connect(host, port, password).await {
            Ok(master_pool) => pools.push(master_pool),
            Err(err) => panic!("store error: {}", err),
        }
    }

    async fn pool_connect(
        host: &str,
        port: u16,
        password: &Option<String>,
    ) -> Result<StorePool, &'static str> {
        info!("binding to store backend at {}:{}", host, port);

        let addr_auth = match password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw = format!(
            "redis://{}{}:{}/{}",
            &addr_auth, host, port, APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let builder = Pool::builder()
                    .test_on_check_out(true)
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

                match builder.build(manager).await {
                    Ok(pool) => {
                        info!("connected to redis at: {}", tcp_addr_raw);

                        Ok(StorePool {
                            connection: pool,
                            target: tcp_addr_raw,
                            delinquent_until: RwLock::new(None),
                        })
                    }
                    Err(_) => Err("could not spawn redis pool"),
                }
            }
            Err(_) => Err("could not create redis connection manager"),
        }
    }
}

impl Store {
    pub async fn get(
        &self,
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
        origin: StoreAccessOrigin,
    ) -> Result<StoreRecord, StoreError> {
        let store_key = StoreKey::to_key(zone_name, record_name, record_type);

        // #1. Get from local cache?
        if let Ok(cached_records) = STORE_CACHE.get(&store_key) {
            debug!(
                "get from local store from any on type: {:?}, zone: {:?}, record: {:?}",
                record_type, zone_name, record_name
            );

            return match cached_records {
                Some(cached_records) => Ok(cached_records),
                None => Err(StoreError::NotFound),
            };
        }

        // #2. Get from store (internal origin? ie. DOS-safe)
        // Notice: if origin is 'internal' pass-through, otherwise do check limiting policy
        if origin == StoreAccessOrigin::Internal {
            debug!(
                "get from remote store from internal on type: {:?}, zone: {:?}, record: {:?}",
                record_type, zone_name, record_name
            );

            // Read result from remote store
            return self.raw_get_remote(&store_key, None).await;
        }

        // #3. Get from store (external origin, ie. DOS-unsafe, thus we need to apply limits)
        // Notice: this prevents against DOS attacks that exploit the expensive remote store of \
        //   Constellation, as it creates a pending task on the event loop whenever a Redis \
        //   query is pending. Some attackers may overwhelm the event loop by issuing a large \
        //   number of DNS queries on non-cached records (random non-existing records). To avoid \
        //   overwhelming the server event loop with LOADS of pending tasks (waiting for network) \
        //   for the duration of the attack, we limit the total time spent querying Redis to 80% \
        //   of each limiting timespans (of 10 seconds). This way, DOS attacks can not continue \
        //   stacking a huge number of tasks on the event loop, defeating the purpose of the \
        //   attack. Note that this applies to DNS queries coming from external requesters \
        //   only, meaning that cache refresh queries will not be subject to this policy, nor \
        //   health check queries. Also, already-cached records will be served as normal and \
        //   not be subject to those limits (those are almost free requests).
        debug!(
            "get from remote store from external on type: {:?}, zone: {:?}, record: {:?}",
            record_type, zone_name, record_name
        );

        // First, check if limit counters need to be reset and acquire time spent
        let (time_spent_current_timespan, start_instant) = {
            let mut limits_rate_write = self.limits.rate.write().unwrap();
            let now_instant = Instant::now();

            // Counters need to be reset?
            if now_instant.duration_since(limits_rate_write.time_last)
                >= LIMITS_GET_REMOTE_TIMESPAN_TOTAL
            {
                limits_rate_write.time_last = now_instant;
                limits_rate_write.time_spent = Duration::new(0, 0);

                debug!(
                    "started a new time spent chunk in remote store from external ({:?} chunks)",
                    LIMITS_GET_REMOTE_TIMESPAN_TOTAL
                );
            }

            (limits_rate_write.time_spent, now_instant)
        };

        // Time spent in current timespan is already too great? Reject DNS query.
        if time_spent_current_timespan >= LIMITS_GET_REMOTE_ALLOWANCE_THRESHOLD {
            error!(
                "limited remote store get from external on type: {:?}, zone: {:?}, record: {:?}",
                record_type, zone_name, record_name
            );

            // Consider the remote store server to be disconnected, as its network channel is \
            //   overwhelmed with requests.
            return Err(StoreError::Disconnected);
        }

        // Read result from remote store
        let result_remote = self.raw_get_remote(&store_key, None).await;

        // Update time spent in current timespan
        {
            let mut limits_rate_write = self.limits.rate.write().unwrap();

            limits_rate_write.time_spent += start_instant.elapsed();

            debug!(
                "updated time spent in remote store from external to: {:?} in current chunk",
                limits_rate_write.time_spent
            );
        }

        result_remote
    }

    pub async fn set(&self, zone_name: &ZoneName, record: StoreRecord) -> Result<(), StoreError> {
        get_cache_store_client!(&self.pools, StoreError::Disconnected, client {
            let flatten_encoder: Result<String, SerdeJSONError> = match record.flatten {
                Some(true) => {
                    Ok("1".to_owned())
                },
                _ => Ok("".to_owned())
            };
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
            let rescue_encoder = match record.rescue {
                Some(ref rescue) => {
                    if rescue.is_empty() == false {
                        serde_json::to_string(rescue)
                    } else {
                        Ok("".to_owned())
                    }
                },
                None => Ok("".to_owned())
            };

            match (
                serde_json::to_string(&record.values),
                flatten_encoder,
                blackhole_encoder,
                region_encoder,
                rescue_encoder
            ) {
                (Ok(values), Ok(flatten), Ok(blackhole), Ok(regions), Ok(rescue)) => {
                    let store_key = StoreKey::to_key(zone_name, &record.name, &record.kind);

                    // Clean from local cache
                    STORE_CACHE.pop(&store_key);

                    // Store in remote
                    client.hset_multiple(
                        store_key, &[
                            (KEY_TYPE, record.kind.to_str()),
                            (KEY_NAME, record.name.to_str()),
                            (KEY_TTL, &record.ttl.unwrap_or(0).to_string()),
                            (KEY_FLATTEN, &flatten),
                            (KEY_BLACKHOLE, &blackhole),
                            (KEY_REGION, &regions),
                            (KEY_RESCUE, &rescue),
                            (KEY_VALUE, &values),
                        ]
                    ).await.or(Err(StoreError::Connector))
                },
                (Err(_), _, _, _, _) |
                (_, Err(_), _, _, _) |
                (_, _, Err(_), _, _) |
                (_, _, _, Err(_), _) |
                (_, _, _, _, Err(_)) => {
                    Err(StoreError::Encoding)
                }
            }
        })
    }

    pub async fn remove(
        &self,
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
    ) -> Result<(), StoreError> {
        get_cache_store_client!(&self.pools, StoreError::Disconnected, client {
            let store_key = StoreKey::to_key(zone_name, record_name, record_type);

            // Clean from local cache
            STORE_CACHE.pop(&store_key);

            // Delete from remote
            client.del(store_key).await.or(Err(StoreError::Connector))
        })
    }

    pub async fn raw_get_remote(
        &self,
        store_key: &str,
        cache_accessed_at: Option<SystemTime>,
    ) -> Result<StoreRecord, StoreError> {
        get_cache_store_client!(&self.pools, StoreError::Disconnected, client {
            match client.hget::<_, _, StoreGetType>(
                store_key,

                (
                    KEY_TYPE,
                    KEY_NAME,
                    KEY_TTL,
                    KEY_FLATTEN,
                    KEY_BLACKHOLE,
                    KEY_REGION,
                    KEY_RESCUE,
                    KEY_VALUE
                ),
            ).await {
                Ok(values) => {
                    if let (Some(kind_value), Some(name_value), Ok(value_value)) = (
                        RecordType::from_str(&values.0),
                        RecordName::from_str(&values.1),
                        serde_json::from_str(&values.7)
                    ) {
                        let ttl = if values.2 > 0 {
                            Some(values.2)
                        } else {
                            None
                        };

                        let flatten = values.3.and_then(|flatten_raw| {
                            if flatten_raw == "1" {
                                Some(true)
                            } else {
                                None
                            }
                        });
                        let blackhole = values.4.and_then(|blackhole_raw| {
                            serde_json::from_str::<RecordBlackhole>(&blackhole_raw).ok()
                        });
                        let regions = values.5.and_then(|region_raw| {
                            serde_json::from_str::<RecordRegions>(&region_raw).ok()
                        });
                        let rescue = values.6.and_then(|rescue_raw| {
                            serde_json::from_str::<RecordValues>(&rescue_raw).ok()
                        });

                        debug!(
                            "read store record with kind: {:?}, name: {:?} and values: {:?}",
                            kind_value,
                            name_value,
                            value_value
                        );

                        if flatten.is_some() == true {
                            debug!(
                                "store record with kind: {:?}, name: {:?} has flatten: {:?}",
                                kind_value,
                                name_value,
                                flatten
                            );
                        }
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
                        if rescue.is_some() == true {
                             debug!(
                                "store record with kind: {:?}, name: {:?} has rescue: {:?}",
                                kind_value,
                                name_value,
                                rescue
                            );
                        }

                        let record = StoreRecord {
                            kind: kind_value,
                            name: name_value,
                            ttl: ttl,
                            flatten: flatten,
                            blackhole: blackhole,
                            regions: regions,
                            rescue: rescue,
                            values: value_value,
                        };

                        // Store in local cache
                        STORE_CACHE.push(store_key, Some(record.clone()), cache_accessed_at);

                        Ok(record)
                    } else {
                        Err(StoreError::Corrupted)
                    }
                },
                Err(err) => {
                    debug!("could not read store record at key: {}, because: {}", store_key, err);

                    // Store in local cache? (no value)
                    // Notice: do not store an empty cache if error is not a type error (meaning: \
                    //   no such value exist; this avoids storing a blank cache entry for I/O \
                    //   and network timeout errors, which would corrupt the cache)
                    if err.kind() == ErrorKind::TypeError {
                        STORE_CACHE.push(store_key, None, cache_accessed_at);
                    }

                    // Consider as not found
                    Err(StoreError::NotFound)
                },
            }
        })
    }
}

impl Default for StoreLimitsRate {
    fn default() -> Self {
        Self {
            time_last: Instant::now(),
            time_spent: Duration::new(0, 0),
        }
    }
}

impl StoreRecord {
    pub fn list_record_values<'a>(&'a self) -> HashSet<&'a RecordValue> {
        let mut unique_values = HashSet::new();

        // Insert base values
        for value in self.values.iter() {
            unique_values.insert(value);
        }

        // Insert all geographic region values?
        if let Some(ref regions) = self.regions {
            self.insert_record_values(&regions.nnam, &mut unique_values);
            self.insert_record_values(&regions.snam, &mut unique_values);
            self.insert_record_values(&regions.nsam, &mut unique_values);
            self.insert_record_values(&regions.ssam, &mut unique_values);
            self.insert_record_values(&regions.weu, &mut unique_values);
            self.insert_record_values(&regions.ceu, &mut unique_values);
            self.insert_record_values(&regions.eeu, &mut unique_values);
            self.insert_record_values(&regions.ru, &mut unique_values);
            self.insert_record_values(&regions.me, &mut unique_values);
            self.insert_record_values(&regions.naf, &mut unique_values);
            self.insert_record_values(&regions.maf, &mut unique_values);
            self.insert_record_values(&regions.saf, &mut unique_values);
            self.insert_record_values(&regions.seas, &mut unique_values);
            self.insert_record_values(&regions.neas, &mut unique_values);
            self.insert_record_values(&regions.oc, &mut unique_values);
            self.insert_record_values(&regions._in, &mut unique_values);
        }

        unique_values
    }

    fn insert_record_values<'a>(
        &'a self,
        record_values: &'a Option<RecordValues>,
        unique_values: &mut HashSet<&'a RecordValue>,
    ) {
        if let Some(record_values) = record_values {
            for value in record_values.iter() {
                unique_values.insert(value);
            }
        }
    }
}
