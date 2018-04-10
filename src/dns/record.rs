// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::request::FromParam;
use rocket::http::RawStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum RecordType {
    #[serde(rename = "a")]
    A,

    #[serde(rename = "aaaa")]
    AAAA,

    #[serde(rename = "afsdb")]
    AFSDB,

    #[serde(rename = "apl")]
    APL,

    #[serde(rename = "caa")]
    CAA,

    #[serde(rename = "cdnskey")]
    CDNSKEY,

    #[serde(rename = "cds")]
    CDS,

    #[serde(rename = "cert")]
    CERT,

    #[serde(rename = "cname")]
    CNAME,

    #[serde(rename = "dhcid")]
    DHCID,

    #[serde(rename = "dlv")]
    DLV,

    #[serde(rename = "dname")]
    DNAME,

    #[serde(rename = "dnskey")]
    DNSKEY,

    #[serde(rename = "ds")]
    DS,

    #[serde(rename = "hip")]
    HIP,

    #[serde(rename = "ipseckey")]
    IPSECKEY,

    #[serde(rename = "key")]
    KEY,

    #[serde(rename = "kx")]
    KX,

    #[serde(rename = "loc")]
    LOC,

    #[serde(rename = "mx")]
    MX,

    #[serde(rename = "naptr")]
    NAPTR,

    #[serde(rename = "ns")]
    NS,

    #[serde(rename = "nsec")]
    NSEC,

    #[serde(rename = "nsec3")]
    NSEC3,

    #[serde(rename = "nsec3param")]
    NSEC3PARAM,

    #[serde(rename = "openpgpkey")]
    OPENPGPKEY,

    #[serde(rename = "ptr")]
    PTR,

    #[serde(rename = "rrsig")]
    RRSIG,

    #[serde(rename = "rp")]
    RP,

    #[serde(rename = "sig")]
    SIG,

    #[serde(rename = "soa")]
    SOA,

    #[serde(rename = "srv")]
    SRV,

    #[serde(rename = "sshfp")]
    SSHFP,

    #[serde(rename = "ta")]
    TA,

    #[serde(rename = "tkey")]
    TKEY,

    #[serde(rename = "tlsa")]
    TLSA,

    #[serde(rename = "tsig")]
    TSIG,

    #[serde(rename = "txt")]
    TXT,

    #[serde(rename = "uri")]
    URI,
}

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

impl<'r> FromParam<'r> for RecordType {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        RecordType::from_str(param).ok_or(param)
    }
}
