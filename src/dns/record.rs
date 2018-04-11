// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt, str};
use regex::Regex;
use rocket::request::FromParam;
use rocket::http::RawStr;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Unexpected, Error as DeserializeError};

lazy_static! {
    static ref RECORD_NAME_REGEX: Regex = Regex::new(r"^(([^\\/:@&_\*]+)\.)?@$").unwrap();
}

serde_string_impls!(RecordType);
serde_string_impls!(RecordName);

#[derive(Clone)]
pub enum RecordType {
    A,
    AAAA,
    AFSDB,
    APL,
    CAA,
    CDNSKEY,
    CDS,
    CERT,
    CNAME,
    DHCID,
    DLV,
    DNAME,
    DNSKEY,
    DS,
    HIP,
    IPSECKEY,
    KEY,
    KX,
    LOC,
    MX,
    NAPTR,
    NS,
    NSEC,
    NSEC3,
    NSEC3PARAM,
    OPENPGPKEY,
    PTR,
    RRSIG,
    RP,
    SIG,
    SOA,
    SRV,
    SSHFP,
    TA,
    TKEY,
    TLSA,
    TSIG,
    TXT,
    URI,
}

#[derive(Clone)]
pub struct RecordName(String);

impl RecordType {
    pub fn from_str(value: &str) -> Option<RecordType> {
        match value {
            "a" => Some(RecordType::A),
            "aaaa" => Some(RecordType::AAAA),
            "afsdb" => Some(RecordType::AFSDB),
            "apl" => Some(RecordType::APL),
            "caa" => Some(RecordType::CAA),
            "cdnskey" => Some(RecordType::CDNSKEY),
            "cds" => Some(RecordType::CDS),
            "cert" => Some(RecordType::CERT),
            "cname" => Some(RecordType::CNAME),
            "dhcid" => Some(RecordType::DHCID),
            "dlv" => Some(RecordType::DLV),
            "dname" => Some(RecordType::DNAME),
            "dnskey" => Some(RecordType::DNSKEY),
            "ds" => Some(RecordType::DS),
            "hip" => Some(RecordType::HIP),
            "ipseckey" => Some(RecordType::IPSECKEY),
            "key" => Some(RecordType::KEY),
            "kx" => Some(RecordType::KX),
            "loc" => Some(RecordType::LOC),
            "mx" => Some(RecordType::MX),
            "naptr" => Some(RecordType::NAPTR),
            "ns" => Some(RecordType::NS),
            "nsec" => Some(RecordType::NSEC),
            "nsec3" => Some(RecordType::NSEC3),
            "nsec3param" => Some(RecordType::NSEC3PARAM),
            "openpgpkey" => Some(RecordType::OPENPGPKEY),
            "ptr" => Some(RecordType::PTR),
            "rrsig" => Some(RecordType::RRSIG),
            "rp" => Some(RecordType::RP),
            "sig" => Some(RecordType::SIG),
            "soa" => Some(RecordType::SOA),
            "srv" => Some(RecordType::SRV),
            "sshfp" => Some(RecordType::SSHFP),
            "ta" => Some(RecordType::TA),
            "tkey" => Some(RecordType::TKEY),
            "tlsa" => Some(RecordType::TLSA),
            "tsig" => Some(RecordType::TSIG),
            "txt" => Some(RecordType::TXT),
            "uri" => Some(RecordType::URI),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match *self {
            RecordType::A => "a",
            RecordType::AAAA => "aaaa",
            RecordType::AFSDB => "afsdb",
            RecordType::APL => "apl",
            RecordType::CAA => "caa",
            RecordType::CDNSKEY => "cdnskey",
            RecordType::CDS => "cds",
            RecordType::CERT => "cert",
            RecordType::CNAME => "cname",
            RecordType::DHCID => "dhcid",
            RecordType::DLV => "dlv",
            RecordType::DNAME => "dname",
            RecordType::DNSKEY => "dnskey",
            RecordType::DS => "ds",
            RecordType::HIP => "hip",
            RecordType::IPSECKEY => "ipseckey",
            RecordType::KEY => "key",
            RecordType::KX => "kx",
            RecordType::LOC => "loc",
            RecordType::MX => "mx",
            RecordType::NAPTR => "naptr",
            RecordType::NS => "ns",
            RecordType::NSEC => "nsec",
            RecordType::NSEC3 => "nsec3",
            RecordType::NSEC3PARAM => "nsec3param",
            RecordType::OPENPGPKEY => "openpgpkey",
            RecordType::PTR => "ptr",
            RecordType::RRSIG => "rrsig",
            RecordType::RP => "rp",
            RecordType::SIG => "sig",
            RecordType::SOA => "soa",
            RecordType::SRV => "srv",
            RecordType::SSHFP => "sshfp",
            RecordType::TA => "ta",
            RecordType::TKEY => "tkey",
            RecordType::TLSA => "tlsa",
            RecordType::TSIG => "tsig",
            RecordType::TXT => "txt",
            RecordType::URI => "uri",
        }
    }
}

impl RecordName {
    pub fn from_str(value: &str) -> Option<RecordName> {
        if Self::validate(value) {
            Some(RecordName(value.to_string()))
        } else {
            None
        }
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }

    pub fn validate(value: &str) -> bool {
        RECORD_NAME_REGEX.is_match(value)
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
