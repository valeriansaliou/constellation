// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::rdata::{self as HickoryRData};
use hickory_proto::rr::{LowerName as HickoryLowerName, RData, RecordType as HickoryRecordType};
use hickory_proto::serialize::txt::RDataParser;
use regex::Regex;
use serde::de::{Error as DeserializeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp;
use std::collections::HashSet;
use std::ops::Deref;
use std::{fmt, str};

use crate::geo::country::CountryCode;

lazy_static! {
    static ref RECORD_NAME_REGEX: Regex = Regex::new(r"^(\*\.)?(([^\\/:@&\*]+)\.)?@$").unwrap();
}

static DATA_TXT_CHUNK_MAXIMUM: usize = 255;

serde_string_impls!(RecordType, from_str);
serde_string_impls!(RecordName, from_str);

gen_record_type_impls!(
    A -> "a",
    AAAA -> "aaaa",
    CNAME -> "cname",
    MX -> "mx",
    TXT -> "txt",
    CAA -> "caa",
    PTR -> "ptr",
);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RecordName(String);

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RecordValue(String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordValues(Vec<RecordValue>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordBlackhole(HashSet<CountryCode>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordRegions {
    pub nnam: Option<RecordValues>,
    pub snam: Option<RecordValues>,
    pub nsam: Option<RecordValues>,
    pub ssam: Option<RecordValues>,
    pub weu: Option<RecordValues>,
    pub ceu: Option<RecordValues>,
    pub eeu: Option<RecordValues>,
    pub ru: Option<RecordValues>,
    pub me: Option<RecordValues>,
    pub naf: Option<RecordValues>,
    pub maf: Option<RecordValues>,
    pub saf: Option<RecordValues>,
    pub seas: Option<RecordValues>,
    pub neas: Option<RecordValues>,
    pub oc: Option<RecordValues>,

    #[serde(rename = "in")]
    pub _in: Option<RecordValues>,
}

impl RecordName {
    pub fn from_str(value: &str) -> Option<RecordName> {
        if Self::validate(value) {
            Some(RecordName(value.to_string().to_lowercase()))
        } else {
            None
        }
    }

    pub fn from_hickory(
        zone_name: &HickoryLowerName,
        query_name: &HickoryLowerName,
    ) -> Option<RecordName> {
        let mut query_string = query_name.to_string().to_lowercase();
        let query_len = query_string.len();

        // Nuke authority section from domain?
        if query_len > 0 {
            let zone_string = zone_name.to_string().to_lowercase();
            let zone_len = zone_string.len();

            if query_string.get((query_len - 1)..query_len) == Some(".")
                && query_string.ends_with(&zone_string)
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

    pub fn to_subdomain(&self) -> &str {
        let raw_str = self.to_str();

        if raw_str.len() > 1 {
            &raw_str[..(raw_str.len() - 1)]
        } else {
            ""
        }
    }

    pub fn validate(value: &str) -> bool {
        RECORD_NAME_REGEX.is_match(value)
    }
}

impl RecordValue {
    pub fn to_hickory(&self, record_type: &RecordType) -> Result<RData, ()> {
        let hickory_record_type = record_type.to_hickory()?;

        match record_type {
            RecordType::A
            | RecordType::AAAA
            | RecordType::CNAME
            | RecordType::MX
            | RecordType::CAA
            | RecordType::PTR => {
                RDataParser::try_from_str(hickory_record_type, self.to_str()).or(Err(()))
            }
            RecordType::TXT => {
                // Split TXT records to parts of 255 characters (enforced by specs)
                // Notice: unfortunately, RDataParser does not work well with large TXT \
                //   records, such as DKIM keys. We have to implement a custom parser \
                //   here.
                let mut txt_splits = Vec::new();
                let mut last_value = self.to_str();

                while !last_value.is_empty() {
                    let (chunk_value, rest_value) =
                        last_value.split_at(cmp::min(DATA_TXT_CHUNK_MAXIMUM, last_value.len()));

                    txt_splits.push(chunk_value.to_string());

                    last_value = rest_value;
                }

                if !txt_splits.is_empty() {
                    Ok(RData::TXT(HickoryRData::txt::TXT::new(txt_splits)))
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl RecordBlackhole {
    pub fn has_items(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn has_country(&self, country: &CountryCode) -> bool {
        self.0.contains(country)
    }
}

impl RecordValues {
    pub fn new() -> RecordValues {
        RecordValues(Vec::new())
    }

    pub fn from_list(values: Vec<RecordValue>) -> RecordValues {
        RecordValues(values)
    }
}

impl RecordValue {
    pub fn from_string(value: String) -> RecordValue {
        RecordValue(value)
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
