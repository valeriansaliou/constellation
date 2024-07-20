// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::SystemTime;

use super::store::StoreRecord;
use crate::APP_CONF;
use crate::APP_STORE;

lazy_static! {
    pub static ref STORE_CACHE: StoreCache = StoreCacheBuilder::new();
}

struct StoreCacheBuilder;

pub struct StoreCache {
    cache: RwLock<HashMap<String, StoreCacheEntry>>,
}

pub struct StoreCacheFlush;

struct StoreCacheEntry {
    record: Option<StoreRecord>,
    refreshed_at: SystemTime,
    accessed_at: SystemTime,
}

impl StoreCacheBuilder {
    fn new() -> StoreCache {
        StoreCache {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl StoreCache {
    pub fn get(&self, store_key: &str) -> Result<Option<StoreRecord>, ()> {
        let mut cache_write = self.cache.write().unwrap();

        debug!("store cache get on key: {}", store_key);

        if let Some(ref mut store_record) = cache_write.get_mut(store_key) {
            debug!("store cache get got records for key: {}", store_key);

            // Bump last cache access time
            store_record.accessed_at = SystemTime::now();

            Ok(store_record.record.clone())
        } else {
            debug!("store cache get did not get records for key: {}", store_key);

            Err(())
        }
    }

    pub fn push(
        &self,
        store_key: &str,
        store_record: Option<StoreRecord>,
        accessed_at: Option<SystemTime>,
    ) {
        let mut cache_write = self.cache.write().unwrap();

        debug!("store cache push on key: {}", store_key);

        cache_write.insert(
            store_key.to_string(),
            StoreCacheEntry::new(store_record, accessed_at),
        );
    }

    pub fn pop(&self, store_key: &str) {
        let mut cache_write = self.cache.write().unwrap();

        debug!("store cache pop on key: {}", store_key);

        cache_write.remove(store_key);
    }
}

impl StoreCacheFlush {
    pub fn expire() {
        debug!("flushing expired store cache records");

        let mut expire_register: Vec<String> = Vec::new();

        // Scan for expired store items
        {
            let cache_read = STORE_CACHE.cache.read().unwrap();
            let now_time = SystemTime::now();

            for (store_key, store) in cache_read.iter() {
                let store_elapsed = now_time
                    .duration_since(store.accessed_at)
                    .unwrap()
                    .as_secs();

                if store_elapsed >= APP_CONF.redis.cache_expire_seconds {
                    expire_register.push(store_key.to_owned());
                }
            }
        }

        // Any store item to expire?
        if expire_register.is_empty() == false {
            let mut cache_write = STORE_CACHE.cache.write().unwrap();

            for store_key in &expire_register {
                cache_write.remove(store_key);
            }
        }

        debug!(
            "flushed expired store cache records (count: {})",
            expire_register.len()
        );
    }

    pub async fn refresh() {
        debug!("flushing to-be-refreshed store cache records");

        let mut refresh_register: Vec<(String, SystemTime)> = Vec::new();

        // Scan for to-be-refreshed store items
        {
            let cache_read = STORE_CACHE.cache.read().unwrap();
            let now_time = SystemTime::now();

            for (store_key, store) in cache_read.iter() {
                let store_elapsed = now_time
                    .duration_since(store.refreshed_at)
                    .unwrap()
                    .as_secs();

                if store_elapsed >= APP_CONF.redis.cache_refresh_seconds {
                    refresh_register.push((store_key.to_owned(), store.accessed_at));
                }
            }
        }

        // Any store item to refresh?
        if refresh_register.is_empty() == false {
            for (store_key, store_accessed_at) in &refresh_register {
                // Notice: restore 'accessed_at' time, otherwise a never-accessed cache entry \
                //   would never be expired.
                APP_STORE
                    .raw_get_remote(store_key, Some(*store_accessed_at))
                    .await
                    .ok();
            }
        }

        debug!(
            "flushed to-be-refreshed store cache records (count: {})",
            refresh_register.len()
        );
    }
}

impl StoreCacheEntry {
    fn new(record: Option<StoreRecord>, accessed_at: Option<SystemTime>) -> StoreCacheEntry {
        let time_now = SystemTime::now();

        StoreCacheEntry {
            record: record,
            refreshed_at: time_now,
            accessed_at: accessed_at.unwrap_or(time_now),
        }
    }
}
