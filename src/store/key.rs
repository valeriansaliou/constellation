// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use farmhash;

use crate::dns::record::{RecordName, RecordType};
use crate::dns::zone::ZoneName;

pub struct StoreKey;

pub static KEY_PREFIX: &'static str = "cl";

impl StoreKey {
    pub fn to_key(
        zone_name: &ZoneName,
        record_name: &RecordName,
        record_type: &RecordType,
    ) -> String {
        let key = format!(
            "{}:{}",
            KEY_PREFIX,
            Self::hash(zone_name, record_name, record_type)
        );

        debug!(
            "generated key: {} for record: {} on type: {}",
            key,
            record_name.to_str(),
            record_type.to_str()
        );

        key
    }

    fn hash(zone_name: &ZoneName, record_name: &RecordName, record_type: &RecordType) -> String {
        debug!(
            "hashing record: {} on type: {} for zone: {}",
            record_name.to_str(),
            record_type.to_str(),
            zone_name.to_str()
        );

        format!(
            "{:x}:{:x}:{}",
            farmhash::fingerprint32(zone_name.to_str().as_bytes()),
            farmhash::fingerprint32(record_name.to_str().as_bytes()),
            record_type.to_str()
        )
    }
}
