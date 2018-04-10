// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use farmhash;

use dns::record::RecordType;

pub struct StoreKey;

pub static KEY_PREFIX: &'static str = "cl";

impl StoreKey {
    pub fn to_key(record_name: &str, record_type: &RecordType) -> String {
        let key = format!("{}:{}", KEY_PREFIX, Self::hash(record_name, record_type));

        debug!(
            "generated key: {} for record: {} on type: {:?}",
            key,
            record_name,
            record_type
        );

        key
    }

    fn hash(record_name: &str, record_type: &RecordType) -> String {
        debug!("hashing record: {} on type: {:?}", record_name, record_type);

        format!(
            "{:x}:{}",
            farmhash::fingerprint32(record_name.as_bytes()),
            record_type.to_str()
        )
    }
}
