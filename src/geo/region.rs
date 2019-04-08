// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Clone, Debug, PartialEq)]
pub enum RegionCode {
    NNAM,
    SNAM,
    NSAM,
    SSAM,
    WEU,
    CEU,
    EEU,
    RU,
    ME,
    NAF,
    MAF,
    SAF,
    IN,
    SEAS,
    NEAS,
    OC,
}

impl RegionCode {
    pub fn to_name(&self) -> &'static str {
        match *self {
            RegionCode::NNAM => "Northern North America",
            RegionCode::SNAM => "Southern North America",
            RegionCode::NSAM => "Northern South America",
            RegionCode::SSAM => "Southern South America",
            RegionCode::WEU => "Western Europe",
            RegionCode::CEU => "Central Europe",
            RegionCode::EEU => "Eastern Europe",
            RegionCode::RU => "Russia",
            RegionCode::ME => "Middle East",
            RegionCode::NAF => "Northern Africa",
            RegionCode::MAF => "Middle Africa",
            RegionCode::SAF => "Southern Africa",
            RegionCode::IN => "India",
            RegionCode::SEAS => "Southeast Asia",
            RegionCode::NEAS => "Northeast Asia",
            RegionCode::OC => "Oceania",
        }
    }
}
