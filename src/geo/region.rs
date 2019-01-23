// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Clone, Debug, PartialEq)]
pub enum RegionCode {
    EU,
    NAM,
    SAM,
    OC,
    ME,
    IN,
    AS,
    AF,
}

impl RegionCode {
    pub fn to_name(&self) -> &'static str {
        match *self {
            RegionCode::EU => "Europe",
            RegionCode::NAM => "North America",
            RegionCode::SAM => "South America",
            RegionCode::OC => "Oceania",
            RegionCode::ME => "Middle East",
            RegionCode::IN => "India",
            RegionCode::AS => "Asia",
            RegionCode::AF => "Africa",
        }
    }
}
