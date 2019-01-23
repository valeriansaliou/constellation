// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, str};
use std::cmp;
use std::ops::Deref;
use regex::Regex;
use rocket::request::FromParam;
use rocket::http::RawStr;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Unexpected, Error as DeserializeError};
use trust_dns::rr::{Name as TrustName, RecordType as TrustRecordType, RData as TrustRData};
use trust_dns::rr::rdata::mx::MX;
use trust_dns::rr::rdata::txt::TXT;

lazy_static! {
    static ref RECORD_NAME_REGEX: Regex = Regex::new(r"^(\*\.)?(([^\\/:@&\*]+)\.)?@$").unwrap();
}

static DATA_TXT_CHUNK_MAXIMUM: usize = 255;

serde_string_impls!(RecordType);
serde_string_impls!(RecordName);

#[derive(Clone, Debug, PartialEq)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    PTR,
}

#[derive(Clone, Debug)]
pub struct RecordName(String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordValue(String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordValues(Vec<RecordValue>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordRegions {
    pub eu: Option<RecordValues>,
    pub nam: Option<RecordValues>,
    pub sam: Option<RecordValues>,
    pub oc: Option<RecordValues>,
    pub me: Option<RecordValues>,
    pub af: Option<RecordValues>,

    #[serde(rename = "in")]
    pub _in: Option<RecordValues>,

    #[serde(rename = "as")]
    pub _as: Option<RecordValues>,
}

impl RecordType {
    pub fn from_str(value: &str) -> Option<RecordType> {
        match value {
            "a" => Some(RecordType::A),
            "aaaa" => Some(RecordType::AAAA),
            "cname" => Some(RecordType::CNAME),
            "mx" => Some(RecordType::MX),
            "txt" => Some(RecordType::TXT),
            "ptr" => Some(RecordType::PTR),
            _ => None,
        }
    }

    pub fn from_trust(record_type: &TrustRecordType) -> Option<RecordType> {
        match record_type {
            &TrustRecordType::A => Some(RecordType::A),
            &TrustRecordType::AAAA => Some(RecordType::AAAA),
            &TrustRecordType::CNAME => Some(RecordType::CNAME),
            &TrustRecordType::MX => Some(RecordType::MX),
            &TrustRecordType::TXT => Some(RecordType::TXT),
            &TrustRecordType::PTR => Some(RecordType::PTR),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match *self {
            RecordType::A => "a",
            RecordType::AAAA => "aaaa",
            RecordType::CNAME => "cname",
            RecordType::MX => "mx",
            RecordType::TXT => "txt",
            RecordType::PTR => "ptr",
        }
    }

    pub fn to_trust(&self) -> Result<TrustRecordType, ()> {
        match *self {
            RecordType::A => Ok(TrustRecordType::A),
            RecordType::AAAA => Ok(TrustRecordType::AAAA),
            RecordType::CNAME => Ok(TrustRecordType::CNAME),
            RecordType::MX => Ok(TrustRecordType::MX),
            RecordType::TXT => Ok(TrustRecordType::TXT),
            RecordType::PTR => Ok(TrustRecordType::PTR),
        }
    }

    pub fn list_choices() -> Vec<RecordType> {
        return vec![
            RecordType::A,
            RecordType::AAAA,
            RecordType::CNAME,
            RecordType::MX,
            RecordType::TXT,
            RecordType::PTR,
        ];
    }
}

impl RecordName {
    pub fn from_str(value: &str) -> Option<RecordName> {
        if Self::validate(value) {
            Some(RecordName(value.to_string().to_lowercase()))
        } else {
            None
        }
    }

    pub fn from_trust(zone_name: &TrustName, query_name: &TrustName) -> Option<RecordName> {
        let mut query_string = query_name.to_string().to_lowercase();
        let query_len = query_string.len();

        // Nuke authority section from domain?
        if query_len > 0 {
            let zone_string = zone_name.to_string().to_lowercase();
            let zone_len = zone_string.len();

            if query_string.get((query_len - 1)..query_len) == Some(".") &&
                query_string.ends_with(&zone_string)
            {
                query_string.truncate(query_len - zone_len);
            }
        }

        // Encode record name in internal format
        query_string = format!("{}@", query_string);

        RecordName::from_str(&query_string)
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }

    pub fn validate(value: &str) -> bool {
        RECORD_NAME_REGEX.is_match(value)
    }
}

impl RecordValue {
    pub fn to_trust(&self, record_type: &RecordType) -> Result<TrustRData, ()> {
        match record_type {
            RecordType::A => {
                // Parse A into actual IPv4
                self.parse().map(|value| TrustRData::A(value)).or(Err(()))
            }
            RecordType::AAAA => {
                // Parse AAAA into actual IPv6
                self.parse().map(|value| TrustRData::AAAA(value)).or(
                    Err(()),
                )
            }
            RecordType::CNAME => {
                // Parse CNAME into domain name
                TrustName::parse(self, Some(&TrustName::new()))
                    .map(|value| TrustRData::CNAME(value))
                    .or(Err(()))
            }
            RecordType::MX => {
                // Parse MX record into (priority, exchange) tuple
                let mut mx_parts = self.split(" ");

                let priority_str = mx_parts.next().unwrap_or("0");
                let exchange_str = mx_parts.next().unwrap_or("");

                if let (Ok(priority), Ok(exchange)) =
                    (
                        priority_str.parse::<u16>(),
                        TrustName::parse(exchange_str, Some(&TrustName::new())),
                    )
                {
                    Ok(TrustRData::MX(MX::new(priority, exchange)))
                } else {
                    Err(())
                }
            }
            RecordType::TXT => {
                // Split TXT records to parts of 255 characters (enforced by specs)
                let mut txt_splits = Vec::new();
                let mut last_value = self.0.as_str();

                while !last_value.is_empty() {
                    let (chunk_value, rest_value) =
                        last_value.split_at(cmp::min(DATA_TXT_CHUNK_MAXIMUM, last_value.len()));

                    txt_splits.push(chunk_value.to_string());

                    last_value = rest_value;
                }

                if !txt_splits.is_empty() {
                    Ok(TrustRData::TXT(TXT::new(txt_splits)))
                } else {
                    Err(())
                }
            }
            RecordType::PTR => {
                TrustName::parse(self, Some(&TrustName::new()))
                    .map(|value| TrustRData::PTR(value))
                    .or(Err(()))
            }
        }
    }
}

impl Deref for RecordValues {
    type Target = Vec<RecordValue>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RecordValue {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r> FromParam<'r> for RecordType {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        RecordType::from_str(param).ok_or(param)
    }
}

impl<'r> FromParam<'r> for RecordName {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        RecordName::from_str(param).ok_or(param)
    }
}
