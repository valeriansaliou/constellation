// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use regex::Regex;
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::de::{Error as DeserializeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str};
use trust_dns_proto::rr::Name as TrustName;

use crate::APP_CONF;

lazy_static! {
    static ref ZONE_NAME_REGEX: Regex =
        Regex::new(r"^(([^\\/:@&_\*]+)\.)[^\\/:@&_\*\-\.]{2,63}$").unwrap();
}

serde_string_impls!(ZoneName);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ZoneName(String);

impl ZoneName {
    pub fn from_str(value: &str) -> Option<ZoneName> {
        if ZONE_NAME_REGEX.is_match(value) {
            Some(ZoneName(value.to_string().to_lowercase()))
        } else {
            None
        }
    }

    pub fn from_trust(query_name: &TrustName) -> Option<ZoneName> {
        let zone_string = query_name.to_string().to_lowercase();
        let mut zone_len = zone_string.len();

        if zone_len > 0 {
            if zone_string.get((zone_len - 1)..zone_len) == Some(".") {
                zone_len = zone_len - 1;
            }

            ZoneName::from_str_exists(&zone_string[..zone_len])
        } else {
            None
        }
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }

    fn from_str_exists(value: &str) -> Option<ZoneName> {
        // As regular `from_str` method is called from configuration builder, the `zone_exists` \
        //   method cannot be called from there, otherwise it would incur a cyclic dependency, \
        //   resulting in an infinite loop while parsing the configuration file. Thus, the \
        //   specialized `from_str_exists` method is used for later `ZoneName` parsing (ie. after \
        //   startup).
        if APP_CONF.dns.zone_exists(value) {
            Self::from_str(value)
        } else {
            None
        }
    }
}

impl<'r> FromParam<'r> for ZoneName {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        ZoneName::from_str_exists(param).ok_or(param)
    }
}
