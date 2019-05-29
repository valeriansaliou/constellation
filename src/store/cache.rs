// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::RwLock;

use super::store::StoreRecord;

lazy_static! {
    pub static ref RECORD_CACHE: RwLock<HashMap<String, Option<StoreRecord>>> =
        RwLock::new(HashMap::new());
}

pub struct StoreCache;

impl StoreCache {
    pub fn has(store_key: &str) -> bool {
        let cache_read = RECORD_CACHE.read().unwrap();

        debug!("store cache has on key: {}", store_key);

        cache_read.contains_key(store_key)
    }

    pub fn get(store_key: &str) -> Result<Option<StoreRecord>, ()> {
        let cache_read = RECORD_CACHE.read().unwrap();

        debug!("store cache get on key: {}", store_key);

        if let Some(store_record) = cache_read.get(store_key) {
            debug!("store cache get got records for key: {}", store_key);

            Ok(store_record.clone())
        } else {
            debug!("store cache get did not get records for key: {}", store_key);

            Err(())
        }
    }

    pub fn push(store_key: &str, store_record: Option<StoreRecord>) {
        let mut cache_write = RECORD_CACHE.write().unwrap();

        debug!("store cache push on key: {}", store_key);

        cache_write.insert(store_key.to_string(), store_record);
    }

    pub fn pop(store_key: &str) {
        let mut cache_write = RECORD_CACHE.write().unwrap();

        debug!("store cache pop on key: {}", store_key);

        cache_write.remove(store_key);
    }
}
