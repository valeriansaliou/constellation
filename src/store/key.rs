// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use farmhash;

use dns::record::{RecordType, RecordName};

pub struct StoreKey;

pub static KEY_PREFIX: &'static str = "cl";

impl StoreKey {
    pub fn to_key(record_name: &RecordName, record_type: &RecordType) -> String {
        let key = format!("{}:{}", KEY_PREFIX, Self::hash(record_name, record_type));

        log::debug!(
            "generated key: {} for record: {} on type: {}",
            key,
            record_name.to_str(),
            record_type.to_str()
        );

        key
    }

    fn hash(record_name: &RecordName, record_type: &RecordType) -> String {
        log::debug!(
            "hashing record: {} on type: {}",
            record_name.to_str(),
            record_type.to_str()
        );

        format!(
            "{:x}:{}",
            farmhash::fingerprint32(record_name.to_str().as_bytes()),
            record_type.to_str()
        )
    }
}
