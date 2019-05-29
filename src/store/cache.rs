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

lazy_static! {
    pub static ref STORE_CACHE: StoreCache = StoreCacheBuilder::new();
}

struct StoreCacheBuilder;

pub struct StoreCache {
    cache: RwLock<HashMap<String, (Option<StoreRecord>, SystemTime)>>,
}

pub struct StoreCacheFlush;

impl StoreCacheBuilder {
    fn new() -> StoreCache {
        StoreCache {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl StoreCache {
    pub fn has(&self, store_key: &str) -> bool {
        let cache_read = self.cache.read().unwrap();

        debug!("store cache has on key: {}", store_key);

        cache_read.contains_key(store_key)
    }

    pub fn get(&self, store_key: &str) -> Result<Option<StoreRecord>, ()> {
        let cache_read = self.cache.read().unwrap();

        debug!("store cache get on key: {}", store_key);

        if let Some(store_record) = cache_read.get(store_key) {
            debug!("store cache get got records for key: {}", store_key);

            Ok(store_record.0.clone())
        } else {
            debug!("store cache get did not get records for key: {}", store_key);

            Err(())
        }
    }

    pub fn push(&self, store_key: &str, store_record: Option<StoreRecord>) {
        let mut cache_write = self.cache.write().unwrap();

        debug!("store cache push on key: {}", store_key);

        cache_write.insert(store_key.to_string(), (store_record, SystemTime::now()));
    }

    pub fn pop(&self, store_key: &str) {
        let mut cache_write = self.cache.write().unwrap();

        debug!("store cache pop on key: {}", store_key);

        cache_write.remove(store_key);
    }
}

impl StoreCacheFlush {
    pub fn expired() {
        debug!("flushing expired store cache records");

        let mut flush_register: Vec<String> = Vec::new();

        // Scan for expired store items
        {
            let cache_read = STORE_CACHE.cache.read().unwrap();
            let now_time = SystemTime::now();

            for (store_key, store) in cache_read.iter() {
                let store_elapsed = now_time.duration_since(store.1).unwrap().as_secs();

                if store_elapsed >= APP_CONF.redis.cache_expire_seconds {
                    flush_register.push(store_key.to_owned());
                }
            }
        }

        // Any store item to flush?
        if flush_register.is_empty() == false {
            let mut cache_write = STORE_CACHE.cache.write().unwrap();

            for store_key in &flush_register {
                cache_write.remove(store_key);
            }
        }

        debug!(
            "flushed expired store cache records (count: {})",
            flush_register.len()
        );
    }
}
