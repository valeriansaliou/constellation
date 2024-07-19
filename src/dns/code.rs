// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::op::ResponseCode;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub enum CodeName {
    NoError,
    FormErr,
    ServFail,
    NXDomain,
    NotImp,
    Refused,
    YXDomain,
    YXRRSet,
    NXRRSet,
    NotAuth,
    NotZone,
}

impl CodeName {
    pub fn from_hickory(response_code: &ResponseCode) -> Option<CodeName> {
        match response_code {
            &ResponseCode::NoError => Some(CodeName::NoError),
            &ResponseCode::FormErr => Some(CodeName::FormErr),
            &ResponseCode::ServFail => Some(CodeName::ServFail),
            &ResponseCode::NXDomain => Some(CodeName::NXDomain),
            &ResponseCode::NotImp => Some(CodeName::NotImp),
            &ResponseCode::Refused => Some(CodeName::Refused),
            &ResponseCode::YXDomain => Some(CodeName::YXDomain),
            &ResponseCode::YXRRSet => Some(CodeName::YXRRSet),
            &ResponseCode::NXRRSet => Some(CodeName::NXRRSet),
            &ResponseCode::NotAuth => Some(CodeName::NotAuth),
            &ResponseCode::NotZone => Some(CodeName::NotZone),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match *self {
            CodeName::NoError => "NoError",
            CodeName::FormErr => "FormErr",
            CodeName::ServFail => "ServFail",
            CodeName::NXDomain => "NXDomain",
            CodeName::NotImp => "NotImp",
            CodeName::Refused => "Refused",
            CodeName::YXDomain => "YXDomain",
            CodeName::YXRRSet => "YXRRSet",
            CodeName::NXRRSet => "NXRRSet",
            CodeName::NotAuth => "NotAuth",
            CodeName::NotZone => "NotZone",
        }
    }
}
