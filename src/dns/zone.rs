// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, str};
use rocket::request::FromParam;
use rocket::http::RawStr;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Unexpected, Error as DeserializeError};

use super::record::DOMAIN_NAME_REGEX;
use APP_CONF;

serde_string_impls!(ZoneName);

#[derive(Clone)]
pub struct ZoneName(String);

impl ZoneName {
    pub fn from_str(value: &str) -> Option<ZoneName> {
        if DOMAIN_NAME_REGEX.is_match(value) && APP_CONF.dns.zone_exists(value) {
            Some(ZoneName(value.to_string()))
        } else {
            None
        }
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl<'r> FromParam<'r> for ZoneName {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        ZoneName::from_str(param).ok_or(param)
    }
}
