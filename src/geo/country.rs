// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::region::RegionCode;

//- Source: https://github.com/lukes/ISO-3166-Countries-with-Regional-Codes/blob/master/all/all.csv
//- Last update: 13 August 2018

#[derive(Clone, Debug, PartialEq)]
pub enum CountryCode {
    AF,
    AX,
    AL,
    DZ,
    AS,
    AD,
    AO,
    AI,
    AG,
    AR,
    AM,
    AW,
    AU,
    AT,
    AZ,
    BS,
    BH,
    BD,
    BB,
    BY,
    BE,
    BZ,
    BJ,
    BM,
    BT,
    BO,
    BQ,
    BA,
    BW,
    BV,
    BR,
    IO,
    BN,
    BG,
    BF,
    BI,
    CV,
    KH,
    CM,
    CA,
    KY,
    CF,
    TD,
    CL,
    CN,
    CX,
    CC,
    CO,
    KM,
    CG,
    CD,
    CK,
    CR,
    CI,
    HR,
    CU,
    CW,
    CY,
    CZ,
    DK,
    DJ,
    DM,
    DO,
    EC,
    EG,
    SV,
    GQ,
    ER,
    EE,
    SZ,
    ET,
    FK,
    FO,
    FJ,
    FI,
    FR,
    GF,
    PF,
    TF,
    GA,
    GM,
    GE,
    DE,
    GH,
    GI,
    GR,
    GL,
    GD,
    GP,
    GU,
    GT,
    GG,
    GN,
    GW,
    GY,
    HT,
    HM,
    VA,
    HN,
    HK,
    HU,
    IS,
    IN,
    ID,
    IR,
    IQ,
    IE,
    IM,
    IL,
    IT,
    JM,
    JP,
    JE,
    JO,
    KZ,
    KE,
    KI,
    KP,
    KR,
    KW,
    KG,
    LA,
    LV,
    LB,
    LS,
    LR,
    LY,
    LI,
    LT,
    LU,
    MO,
    MK,
    MG,
    MW,
    MY,
    MV,
    ML,
    MT,
    MH,
    MQ,
    MR,
    MU,
    YT,
    MX,
    FM,
    MD,
    MC,
    MN,
    ME,
    MS,
    MA,
    MZ,
    MM,
    NA,
    NR,
    NP,
    NL,
    NC,
    NZ,
    NI,
    NE,
    NG,
    NU,
    NF,
    MP,
    NO,
    OM,
    PK,
    PW,
    PS,
    PA,
    PG,
    PY,
    PE,
    PH,
    PN,
    PL,
    PT,
    PR,
    QA,
    RE,
    RO,
    RU,
    RW,
    BL,
    SH,
    KN,
    LC,
    MF,
    PM,
    VC,
    WS,
    SM,
    ST,
    SA,
    SN,
    RS,
    SC,
    SL,
    SG,
    SX,
    SK,
    SI,
    SB,
    SO,
    ZA,
    GS,
    SS,
    ES,
    LK,
    SD,
    SR,
    SJ,
    SE,
    CH,
    SY,
    TW,
    TJ,
    TZ,
    TH,
    TL,
    TG,
    TK,
    TO,
    TT,
    TN,
    TR,
    TM,
    TC,
    TV,
    UG,
    UA,
    AE,
    GB,
    US,
    UM,
    UY,
    UZ,
    VU,
    VE,
    VN,
    VG,
    VI,
    WF,
    EH,
    YE,
    ZM,
    ZW,
}

impl CountryCode {
    pub fn from_str(code: &str) -> Option<CountryCode> {
        match code {
            "AF" => Some(CountryCode::AF),
            "AX" => Some(CountryCode::AX),
            "AL" => Some(CountryCode::AL),
            "DZ" => Some(CountryCode::DZ),
            "AS" => Some(CountryCode::AS),
            "AD" => Some(CountryCode::AD),
            "AO" => Some(CountryCode::AO),
            "AI" => Some(CountryCode::AI),
            "AG" => Some(CountryCode::AG),
            "AR" => Some(CountryCode::AR),
            "AM" => Some(CountryCode::AM),
            "AW" => Some(CountryCode::AW),
            "AU" => Some(CountryCode::AU),
            "AT" => Some(CountryCode::AT),
            "AZ" => Some(CountryCode::AZ),
            "BS" => Some(CountryCode::BS),
            "BH" => Some(CountryCode::BH),
            "BD" => Some(CountryCode::BD),
            "BB" => Some(CountryCode::BB),
            "BY" => Some(CountryCode::BY),
            "BE" => Some(CountryCode::BE),
            "BZ" => Some(CountryCode::BZ),
            "BJ" => Some(CountryCode::BJ),
            "BM" => Some(CountryCode::BM),
            "BT" => Some(CountryCode::BT),
            "BO" => Some(CountryCode::BO),
            "BQ" => Some(CountryCode::BQ),
            "BA" => Some(CountryCode::BA),
            "BW" => Some(CountryCode::BW),
            "BV" => Some(CountryCode::BV),
            "BR" => Some(CountryCode::BR),
            "IO" => Some(CountryCode::IO),
            "BN" => Some(CountryCode::BN),
            "BG" => Some(CountryCode::BG),
            "BF" => Some(CountryCode::BF),
            "BI" => Some(CountryCode::BI),
            "CV" => Some(CountryCode::CV),
            "KH" => Some(CountryCode::KH),
            "CM" => Some(CountryCode::CM),
            "CA" => Some(CountryCode::CA),
            "KY" => Some(CountryCode::KY),
            "CF" => Some(CountryCode::CF),
            "TD" => Some(CountryCode::TD),
            "CL" => Some(CountryCode::CL),
            "CN" => Some(CountryCode::CN),
            "CX" => Some(CountryCode::CX),
            "CC" => Some(CountryCode::CC),
            "CO" => Some(CountryCode::CO),
            "KM" => Some(CountryCode::KM),
            "CG" => Some(CountryCode::CG),
            "CD" => Some(CountryCode::CD),
            "CK" => Some(CountryCode::CK),
            "CR" => Some(CountryCode::CR),
            "CI" => Some(CountryCode::CI),
            "HR" => Some(CountryCode::HR),
            "CU" => Some(CountryCode::CU),
            "CW" => Some(CountryCode::CW),
            "CY" => Some(CountryCode::CY),
            "CZ" => Some(CountryCode::CZ),
            "DK" => Some(CountryCode::DK),
            "DJ" => Some(CountryCode::DJ),
            "DM" => Some(CountryCode::DM),
            "DO" => Some(CountryCode::DO),
            "EC" => Some(CountryCode::EC),
            "EG" => Some(CountryCode::EG),
            "SV" => Some(CountryCode::SV),
            "GQ" => Some(CountryCode::GQ),
            "ER" => Some(CountryCode::ER),
            "EE" => Some(CountryCode::EE),
            "SZ" => Some(CountryCode::SZ),
            "ET" => Some(CountryCode::ET),
            "FK" => Some(CountryCode::FK),
            "FO" => Some(CountryCode::FO),
            "FJ" => Some(CountryCode::FJ),
            "FI" => Some(CountryCode::FI),
            "FR" => Some(CountryCode::FR),
            "GF" => Some(CountryCode::GF),
            "PF" => Some(CountryCode::PF),
            "TF" => Some(CountryCode::TF),
            "GA" => Some(CountryCode::GA),
            "GM" => Some(CountryCode::GM),
            "GE" => Some(CountryCode::GE),
            "DE" => Some(CountryCode::DE),
            "GH" => Some(CountryCode::GH),
            "GI" => Some(CountryCode::GI),
            "GR" => Some(CountryCode::GR),
            "GL" => Some(CountryCode::GL),
            "GD" => Some(CountryCode::GD),
            "GP" => Some(CountryCode::GP),
            "GU" => Some(CountryCode::GU),
            "GT" => Some(CountryCode::GT),
            "GG" => Some(CountryCode::GG),
            "GN" => Some(CountryCode::GN),
            "GW" => Some(CountryCode::GW),
            "GY" => Some(CountryCode::GY),
            "HT" => Some(CountryCode::HT),
            "HM" => Some(CountryCode::HM),
            "VA" => Some(CountryCode::VA),
            "HN" => Some(CountryCode::HN),
            "HK" => Some(CountryCode::HK),
            "HU" => Some(CountryCode::HU),
            "IS" => Some(CountryCode::IS),
            "IN" => Some(CountryCode::IN),
            "ID" => Some(CountryCode::ID),
            "IR" => Some(CountryCode::IR),
            "IQ" => Some(CountryCode::IQ),
            "IE" => Some(CountryCode::IE),
            "IM" => Some(CountryCode::IM),
            "IL" => Some(CountryCode::IL),
            "IT" => Some(CountryCode::IT),
            "JM" => Some(CountryCode::JM),
            "JP" => Some(CountryCode::JP),
            "JE" => Some(CountryCode::JE),
            "JO" => Some(CountryCode::JO),
            "KZ" => Some(CountryCode::KZ),
            "KE" => Some(CountryCode::KE),
            "KI" => Some(CountryCode::KI),
            "KP" => Some(CountryCode::KP),
            "KR" => Some(CountryCode::KR),
            "KW" => Some(CountryCode::KW),
            "KG" => Some(CountryCode::KG),
            "LA" => Some(CountryCode::LA),
            "LV" => Some(CountryCode::LV),
            "LB" => Some(CountryCode::LB),
            "LS" => Some(CountryCode::LS),
            "LR" => Some(CountryCode::LR),
            "LY" => Some(CountryCode::LY),
            "LI" => Some(CountryCode::LI),
            "LT" => Some(CountryCode::LT),
            "LU" => Some(CountryCode::LU),
            "MO" => Some(CountryCode::MO),
            "MK" => Some(CountryCode::MK),
            "MG" => Some(CountryCode::MG),
            "MW" => Some(CountryCode::MW),
            "MY" => Some(CountryCode::MY),
            "MV" => Some(CountryCode::MV),
            "ML" => Some(CountryCode::ML),
            "MT" => Some(CountryCode::MT),
            "MH" => Some(CountryCode::MH),
            "MQ" => Some(CountryCode::MQ),
            "MR" => Some(CountryCode::MR),
            "MU" => Some(CountryCode::MU),
            "YT" => Some(CountryCode::YT),
            "MX" => Some(CountryCode::MX),
            "FM" => Some(CountryCode::FM),
            "MD" => Some(CountryCode::MD),
            "MC" => Some(CountryCode::MC),
            "MN" => Some(CountryCode::MN),
            "ME" => Some(CountryCode::ME),
            "MS" => Some(CountryCode::MS),
            "MA" => Some(CountryCode::MA),
            "MZ" => Some(CountryCode::MZ),
            "MM" => Some(CountryCode::MM),
            "NA" => Some(CountryCode::NA),
            "NR" => Some(CountryCode::NR),
            "NP" => Some(CountryCode::NP),
            "NL" => Some(CountryCode::NL),
            "NC" => Some(CountryCode::NC),
            "NZ" => Some(CountryCode::NZ),
            "NI" => Some(CountryCode::NI),
            "NE" => Some(CountryCode::NE),
            "NG" => Some(CountryCode::NG),
            "NU" => Some(CountryCode::NU),
            "NF" => Some(CountryCode::NF),
            "MP" => Some(CountryCode::MP),
            "NO" => Some(CountryCode::NO),
            "OM" => Some(CountryCode::OM),
            "PK" => Some(CountryCode::PK),
            "PW" => Some(CountryCode::PW),
            "PS" => Some(CountryCode::PS),
            "PA" => Some(CountryCode::PA),
            "PG" => Some(CountryCode::PG),
            "PY" => Some(CountryCode::PY),
            "PE" => Some(CountryCode::PE),
            "PH" => Some(CountryCode::PH),
            "PN" => Some(CountryCode::PN),
            "PL" => Some(CountryCode::PL),
            "PT" => Some(CountryCode::PT),
            "PR" => Some(CountryCode::PR),
            "QA" => Some(CountryCode::QA),
            "RE" => Some(CountryCode::RE),
            "RO" => Some(CountryCode::RO),
            "RU" => Some(CountryCode::RU),
            "RW" => Some(CountryCode::RW),
            "BL" => Some(CountryCode::BL),
            "SH" => Some(CountryCode::SH),
            "KN" => Some(CountryCode::KN),
            "LC" => Some(CountryCode::LC),
            "MF" => Some(CountryCode::MF),
            "PM" => Some(CountryCode::PM),
            "VC" => Some(CountryCode::VC),
            "WS" => Some(CountryCode::WS),
            "SM" => Some(CountryCode::SM),
            "ST" => Some(CountryCode::ST),
            "SA" => Some(CountryCode::SA),
            "SN" => Some(CountryCode::SN),
            "RS" => Some(CountryCode::RS),
            "SC" => Some(CountryCode::SC),
            "SL" => Some(CountryCode::SL),
            "SG" => Some(CountryCode::SG),
            "SX" => Some(CountryCode::SX),
            "SK" => Some(CountryCode::SK),
            "SI" => Some(CountryCode::SI),
            "SB" => Some(CountryCode::SB),
            "SO" => Some(CountryCode::SO),
            "ZA" => Some(CountryCode::ZA),
            "GS" => Some(CountryCode::GS),
            "SS" => Some(CountryCode::SS),
            "ES" => Some(CountryCode::ES),
            "LK" => Some(CountryCode::LK),
            "SD" => Some(CountryCode::SD),
            "SR" => Some(CountryCode::SR),
            "SJ" => Some(CountryCode::SJ),
            "SE" => Some(CountryCode::SE),
            "CH" => Some(CountryCode::CH),
            "SY" => Some(CountryCode::SY),
            "TW" => Some(CountryCode::TW),
            "TJ" => Some(CountryCode::TJ),
            "TZ" => Some(CountryCode::TZ),
            "TH" => Some(CountryCode::TH),
            "TL" => Some(CountryCode::TL),
            "TG" => Some(CountryCode::TG),
            "TK" => Some(CountryCode::TK),
            "TO" => Some(CountryCode::TO),
            "TT" => Some(CountryCode::TT),
            "TN" => Some(CountryCode::TN),
            "TR" => Some(CountryCode::TR),
            "TM" => Some(CountryCode::TM),
            "TC" => Some(CountryCode::TC),
            "TV" => Some(CountryCode::TV),
            "UG" => Some(CountryCode::UG),
            "UA" => Some(CountryCode::UA),
            "AE" => Some(CountryCode::AE),
            "GB" => Some(CountryCode::GB),
            "US" => Some(CountryCode::US),
            "UM" => Some(CountryCode::UM),
            "UY" => Some(CountryCode::UY),
            "UZ" => Some(CountryCode::UZ),
            "VU" => Some(CountryCode::VU),
            "VE" => Some(CountryCode::VE),
            "VN" => Some(CountryCode::VN),
            "VG" => Some(CountryCode::VG),
            "VI" => Some(CountryCode::VI),
            "WF" => Some(CountryCode::WF),
            "EH" => Some(CountryCode::EH),
            "YE" => Some(CountryCode::YE),
            "ZM" => Some(CountryCode::ZM),
            "ZW" => Some(CountryCode::ZW),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match *self {
            CountryCode::AF => "Afghanistan",
            CountryCode::AX => "Åland Islands",
            CountryCode::AL => "Albania",
            CountryCode::DZ => "Algeria",
            CountryCode::AS => "American Samoa",
            CountryCode::AD => "Andorra",
            CountryCode::AO => "Angola",
            CountryCode::AI => "Anguilla",
            CountryCode::AG => "Antigua and Barbuda",
            CountryCode::AR => "Argentina",
            CountryCode::AM => "Armenia",
            CountryCode::AW => "Aruba",
            CountryCode::AU => "Australia",
            CountryCode::AT => "Austria",
            CountryCode::AZ => "Azerbaijan",
            CountryCode::BS => "Bahamas",
            CountryCode::BH => "Bahrain",
            CountryCode::BD => "Bangladesh",
            CountryCode::BB => "Barbados",
            CountryCode::BY => "Belarus",
            CountryCode::BE => "Belgium",
            CountryCode::BZ => "Belize",
            CountryCode::BJ => "Benin",
            CountryCode::BM => "Bermuda",
            CountryCode::BT => "Bhutan",
            CountryCode::BO => "Bolivia (Plurinational State of)",
            CountryCode::BQ => "Bonaire, Sint Eustatius and Saba",
            CountryCode::BA => "Bosnia and Herzegovina",
            CountryCode::BW => "Botswana",
            CountryCode::BV => "Bouvet Island",
            CountryCode::BR => "Brazil",
            CountryCode::IO => "British Indian Ocean Territory",
            CountryCode::BN => "Brunei Darussalam",
            CountryCode::BG => "Bulgaria",
            CountryCode::BF => "Burkina Faso",
            CountryCode::BI => "Burundi",
            CountryCode::CV => "Cabo Verde",
            CountryCode::KH => "Cambodia",
            CountryCode::CM => "Cameroon",
            CountryCode::CA => "Canada",
            CountryCode::KY => "Cayman Islands",
            CountryCode::CF => "Central African Republic",
            CountryCode::TD => "Chad",
            CountryCode::CL => "Chile",
            CountryCode::CN => "China",
            CountryCode::CX => "Christmas Island",
            CountryCode::CC => "Cocos (Keeling) Islands",
            CountryCode::CO => "Colombia",
            CountryCode::KM => "Comoros",
            CountryCode::CG => "Congo",
            CountryCode::CD => "Congo (Democratic Republic of the)",
            CountryCode::CK => "Cook Islands",
            CountryCode::CR => "Costa Rica",
            CountryCode::CI => "Côte d'Ivoire",
            CountryCode::HR => "Croatia",
            CountryCode::CU => "Cuba",
            CountryCode::CW => "Curaçao",
            CountryCode::CY => "Cyprus",
            CountryCode::CZ => "Czechia",
            CountryCode::DK => "Denmark",
            CountryCode::DJ => "Djibouti",
            CountryCode::DM => "Dominica",
            CountryCode::DO => "Dominican Republic",
            CountryCode::EC => "Ecuador",
            CountryCode::EG => "Egypt",
            CountryCode::SV => "El Salvador",
            CountryCode::GQ => "Equatorial Guinea",
            CountryCode::ER => "Eritrea",
            CountryCode::EE => "Estonia",
            CountryCode::SZ => "Eswatini",
            CountryCode::ET => "Ethiopia",
            CountryCode::FK => "Falkland Islands (Malvinas)",
            CountryCode::FO => "Faroe Islands",
            CountryCode::FJ => "Fiji",
            CountryCode::FI => "Finland",
            CountryCode::FR => "France",
            CountryCode::GF => "French Guiana",
            CountryCode::PF => "French Polynesia",
            CountryCode::TF => "French Southern Territories",
            CountryCode::GA => "Gabon",
            CountryCode::GM => "Gambia",
            CountryCode::GE => "Georgia",
            CountryCode::DE => "Germany",
            CountryCode::GH => "Ghana",
            CountryCode::GI => "Gibraltar",
            CountryCode::GR => "Greece",
            CountryCode::GL => "Greenland",
            CountryCode::GD => "Grenada",
            CountryCode::GP => "Guadeloupe",
            CountryCode::GU => "Guam",
            CountryCode::GT => "Guatemala",
            CountryCode::GG => "Guernsey",
            CountryCode::GN => "Guinea",
            CountryCode::GW => "Guinea-Bissau",
            CountryCode::GY => "Guyana",
            CountryCode::HT => "Haiti",
            CountryCode::HM => "Heard Island and McDonald Islands",
            CountryCode::VA => "Holy See",
            CountryCode::HN => "Honduras",
            CountryCode::HK => "Hong Kong",
            CountryCode::HU => "Hungary",
            CountryCode::IS => "Iceland",
            CountryCode::IN => "India",
            CountryCode::ID => "Indonesia",
            CountryCode::IR => "Iran (Islamic Republic of)",
            CountryCode::IQ => "Iraq",
            CountryCode::IE => "Ireland",
            CountryCode::IM => "Isle of Man",
            CountryCode::IL => "Israel",
            CountryCode::IT => "Italy",
            CountryCode::JM => "Jamaica",
            CountryCode::JP => "Japan",
            CountryCode::JE => "Jersey",
            CountryCode::JO => "Jordan",
            CountryCode::KZ => "Kazakhstan",
            CountryCode::KE => "Kenya",
            CountryCode::KI => "Kiribati",
            CountryCode::KP => "Korea (Democratic People's Republic of)",
            CountryCode::KR => "Korea (Republic of)",
            CountryCode::KW => "Kuwait",
            CountryCode::KG => "Kyrgyzstan",
            CountryCode::LA => "Lao People's Democratic Republic",
            CountryCode::LV => "Latvia",
            CountryCode::LB => "Lebanon",
            CountryCode::LS => "Lesotho",
            CountryCode::LR => "Liberia",
            CountryCode::LY => "Libya",
            CountryCode::LI => "Liechtenstein",
            CountryCode::LT => "Lithuania",
            CountryCode::LU => "Luxembourg",
            CountryCode::MO => "Macao",
            CountryCode::MK => "Macedonia (the former Yugoslav Republic of)",
            CountryCode::MG => "Madagascar",
            CountryCode::MW => "Malawi",
            CountryCode::MY => "Malaysia",
            CountryCode::MV => "Maldives",
            CountryCode::ML => "Mali",
            CountryCode::MT => "Malta",
            CountryCode::MH => "Marshall Islands",
            CountryCode::MQ => "Martinique",
            CountryCode::MR => "Mauritania",
            CountryCode::MU => "Mauritius",
            CountryCode::YT => "Mayotte",
            CountryCode::MX => "Mexico",
            CountryCode::FM => "Micronesia (Federated States of)",
            CountryCode::MD => "Moldova (Republic of)",
            CountryCode::MC => "Monaco",
            CountryCode::MN => "Mongolia",
            CountryCode::ME => "Montenegro",
            CountryCode::MS => "Montserrat",
            CountryCode::MA => "Morocco",
            CountryCode::MZ => "Mozambique",
            CountryCode::MM => "Myanmar",
            CountryCode::NA => "Namibia",
            CountryCode::NR => "Nauru",
            CountryCode::NP => "Nepal",
            CountryCode::NL => "Netherlands",
            CountryCode::NC => "New Caledonia",
            CountryCode::NZ => "New Zealand",
            CountryCode::NI => "Nicaragua",
            CountryCode::NE => "Niger",
            CountryCode::NG => "Nigeria",
            CountryCode::NU => "Niue",
            CountryCode::NF => "Norfolk Island",
            CountryCode::MP => "Northern Mariana Islands",
            CountryCode::NO => "Norway",
            CountryCode::OM => "Oman",
            CountryCode::PK => "Pakistan",
            CountryCode::PW => "Palau",
            CountryCode::PS => "Palestine, State of",
            CountryCode::PA => "Panama",
            CountryCode::PG => "Papua New Guinea",
            CountryCode::PY => "Paraguay",
            CountryCode::PE => "Peru",
            CountryCode::PH => "Philippines",
            CountryCode::PN => "Pitcairn",
            CountryCode::PL => "Poland",
            CountryCode::PT => "Portugal",
            CountryCode::PR => "Puerto Rico",
            CountryCode::QA => "Qatar",
            CountryCode::RE => "Réunion",
            CountryCode::RO => "Romania",
            CountryCode::RU => "Russian Federation",
            CountryCode::RW => "Rwanda",
            CountryCode::BL => "Saint Barthélemy",
            CountryCode::SH => "Saint Helena, Ascension and Tristan da Cunha",
            CountryCode::KN => "Saint Kitts and Nevis",
            CountryCode::LC => "Saint Lucia",
            CountryCode::MF => "Saint Martin (French part)",
            CountryCode::PM => "Saint Pierre and Miquelon",
            CountryCode::VC => "Saint Vincent and the Grenadines",
            CountryCode::WS => "Samoa",
            CountryCode::SM => "San Marino",
            CountryCode::ST => "Sao Tome and Principe",
            CountryCode::SA => "Saudi Arabia",
            CountryCode::SN => "Senegal",
            CountryCode::RS => "Serbia",
            CountryCode::SC => "Seychelles",
            CountryCode::SL => "Sierra Leone",
            CountryCode::SG => "Singapore",
            CountryCode::SX => "Sint Maarten (Dutch part)",
            CountryCode::SK => "Slovakia",
            CountryCode::SI => "Slovenia",
            CountryCode::SB => "Solomon Islands",
            CountryCode::SO => "Somalia",
            CountryCode::ZA => "South Africa",
            CountryCode::GS => "South Georgia and the South Sandwich Islands",
            CountryCode::SS => "South Sudan",
            CountryCode::ES => "Spain",
            CountryCode::LK => "Sri Lanka",
            CountryCode::SD => "Sudan",
            CountryCode::SR => "Suriname",
            CountryCode::SJ => "Svalbard and Jan Mayen",
            CountryCode::SE => "Sweden",
            CountryCode::CH => "Switzerland",
            CountryCode::SY => "Syrian Arab Republic",
            CountryCode::TW => "Taiwan, Province of China",
            CountryCode::TJ => "Tajikistan",
            CountryCode::TZ => "Tanzania, United Republic of",
            CountryCode::TH => "Thailand",
            CountryCode::TL => "Timor-Leste",
            CountryCode::TG => "Togo",
            CountryCode::TK => "Tokelau",
            CountryCode::TO => "Tonga",
            CountryCode::TT => "Trinidad and Tobago",
            CountryCode::TN => "Tunisia",
            CountryCode::TR => "Turkey",
            CountryCode::TM => "Turkmenistan",
            CountryCode::TC => "Turks and Caicos Islands",
            CountryCode::TV => "Tuvalu",
            CountryCode::UG => "Uganda",
            CountryCode::UA => "Ukraine",
            CountryCode::AE => "United Arab Emirates",
            CountryCode::GB => "United Kingdom of Great Britain and Northern Ireland",
            CountryCode::US => "United States of America",
            CountryCode::UM => "United States Minor Outlying Islands",
            CountryCode::UY => "Uruguay",
            CountryCode::UZ => "Uzbekistan",
            CountryCode::VU => "Vanuatu",
            CountryCode::VE => "Venezuela (Bolivarian Republic of)",
            CountryCode::VN => "Viet Nam",
            CountryCode::VG => "Virgin Islands (British)",
            CountryCode::VI => "Virgin Islands (U.S.)",
            CountryCode::WF => "Wallis and Futuna",
            CountryCode::EH => "Western Sahara",
            CountryCode::YE => "Yemen",
            CountryCode::ZM => "Zambia",
            CountryCode::ZW => "Zimbabwe",
        }
    }
}

impl CountryCode {
    pub fn to_region_code(&self) -> RegionCode {
        match *self {
            CountryCode::AF => RegionCode::AS,
            CountryCode::AX => RegionCode::EU,
            CountryCode::AL => RegionCode::EU,
            CountryCode::DZ => RegionCode::AF,
            CountryCode::AS => RegionCode::OC,
            CountryCode::AD => RegionCode::EU,
            CountryCode::AO => RegionCode::AF,
            CountryCode::AI => RegionCode::SAM,
            CountryCode::AG => RegionCode::SAM,
            CountryCode::AR => RegionCode::SAM,
            CountryCode::AM => RegionCode::AS,
            CountryCode::AW => RegionCode::SAM,
            CountryCode::AU => RegionCode::OC,
            CountryCode::AT => RegionCode::EU,
            CountryCode::AZ => RegionCode::AS,
            CountryCode::BS => RegionCode::SAM,
            CountryCode::BH => RegionCode::ME,
            CountryCode::BD => RegionCode::AS,
            CountryCode::BB => RegionCode::SAM,
            CountryCode::BY => RegionCode::EU,
            CountryCode::BE => RegionCode::EU,
            CountryCode::BZ => RegionCode::SAM,
            CountryCode::BJ => RegionCode::AF,
            CountryCode::BM => RegionCode::NAM,
            CountryCode::BT => RegionCode::AS,
            CountryCode::BO => RegionCode::SAM,
            CountryCode::BQ => RegionCode::SAM,
            CountryCode::BA => RegionCode::EU,
            CountryCode::BW => RegionCode::AF,
            CountryCode::BV => RegionCode::SAM,
            CountryCode::BR => RegionCode::SAM,
            CountryCode::IO => RegionCode::AF,
            CountryCode::BN => RegionCode::AS,
            CountryCode::BG => RegionCode::EU,
            CountryCode::BF => RegionCode::AF,
            CountryCode::BI => RegionCode::AF,
            CountryCode::CV => RegionCode::AF,
            CountryCode::KH => RegionCode::AS,
            CountryCode::CM => RegionCode::AF,
            CountryCode::CA => RegionCode::NAM,
            CountryCode::KY => RegionCode::SAM,
            CountryCode::CF => RegionCode::AF,
            CountryCode::TD => RegionCode::AF,
            CountryCode::CL => RegionCode::SAM,
            CountryCode::CN => RegionCode::AS,
            CountryCode::CX => RegionCode::OC,
            CountryCode::CC => RegionCode::OC,
            CountryCode::CO => RegionCode::SAM,
            CountryCode::KM => RegionCode::AF,
            CountryCode::CG => RegionCode::AF,
            CountryCode::CD => RegionCode::AF,
            CountryCode::CK => RegionCode::OC,
            CountryCode::CR => RegionCode::SAM,
            CountryCode::CI => RegionCode::AF,
            CountryCode::HR => RegionCode::EU,
            CountryCode::CU => RegionCode::SAM,
            CountryCode::CW => RegionCode::SAM,
            CountryCode::CY => RegionCode::ME,
            CountryCode::CZ => RegionCode::EU,
            CountryCode::DK => RegionCode::EU,
            CountryCode::DJ => RegionCode::AF,
            CountryCode::DM => RegionCode::SAM,
            CountryCode::DO => RegionCode::SAM,
            CountryCode::EC => RegionCode::SAM,
            CountryCode::EG => RegionCode::ME,
            CountryCode::SV => RegionCode::SAM,
            CountryCode::GQ => RegionCode::AF,
            CountryCode::ER => RegionCode::AF,
            CountryCode::EE => RegionCode::EU,
            CountryCode::SZ => RegionCode::AF,
            CountryCode::ET => RegionCode::AF,
            CountryCode::FK => RegionCode::SAM,
            CountryCode::FO => RegionCode::EU,
            CountryCode::FJ => RegionCode::OC,
            CountryCode::FI => RegionCode::EU,
            CountryCode::FR => RegionCode::EU,
            CountryCode::GF => RegionCode::SAM,
            CountryCode::PF => RegionCode::OC,
            CountryCode::TF => RegionCode::AF,
            CountryCode::GA => RegionCode::AF,
            CountryCode::GM => RegionCode::AF,
            CountryCode::GE => RegionCode::AS,
            CountryCode::DE => RegionCode::EU,
            CountryCode::GH => RegionCode::AF,
            CountryCode::GI => RegionCode::EU,
            CountryCode::GR => RegionCode::EU,
            CountryCode::GL => RegionCode::NAM,
            CountryCode::GD => RegionCode::SAM,
            CountryCode::GP => RegionCode::SAM,
            CountryCode::GU => RegionCode::OC,
            CountryCode::GT => RegionCode::SAM,
            CountryCode::GG => RegionCode::EU,
            CountryCode::GN => RegionCode::AF,
            CountryCode::GW => RegionCode::AF,
            CountryCode::GY => RegionCode::SAM,
            CountryCode::HT => RegionCode::SAM,
            CountryCode::HM => RegionCode::OC,
            CountryCode::VA => RegionCode::EU,
            CountryCode::HN => RegionCode::SAM,
            CountryCode::HK => RegionCode::AS,
            CountryCode::HU => RegionCode::EU,
            CountryCode::IS => RegionCode::EU,
            CountryCode::IN => RegionCode::IN,
            CountryCode::ID => RegionCode::AS,
            CountryCode::IR => RegionCode::ME,
            CountryCode::IQ => RegionCode::ME,
            CountryCode::IE => RegionCode::EU,
            CountryCode::IM => RegionCode::EU,
            CountryCode::IL => RegionCode::ME,
            CountryCode::IT => RegionCode::EU,
            CountryCode::JM => RegionCode::SAM,
            CountryCode::JP => RegionCode::AS,
            CountryCode::JE => RegionCode::EU,
            CountryCode::JO => RegionCode::ME,
            CountryCode::KZ => RegionCode::AS,
            CountryCode::KE => RegionCode::AF,
            CountryCode::KI => RegionCode::OC,
            CountryCode::KP => RegionCode::AS,
            CountryCode::KR => RegionCode::AS,
            CountryCode::KW => RegionCode::ME,
            CountryCode::KG => RegionCode::AS,
            CountryCode::LA => RegionCode::AS,
            CountryCode::LV => RegionCode::EU,
            CountryCode::LB => RegionCode::ME,
            CountryCode::LS => RegionCode::AF,
            CountryCode::LR => RegionCode::AF,
            CountryCode::LY => RegionCode::AF,
            CountryCode::LI => RegionCode::EU,
            CountryCode::LT => RegionCode::EU,
            CountryCode::LU => RegionCode::EU,
            CountryCode::MO => RegionCode::AS,
            CountryCode::MK => RegionCode::EU,
            CountryCode::MG => RegionCode::AF,
            CountryCode::MW => RegionCode::AF,
            CountryCode::MY => RegionCode::AS,
            CountryCode::MV => RegionCode::AS,
            CountryCode::ML => RegionCode::AF,
            CountryCode::MT => RegionCode::EU,
            CountryCode::MH => RegionCode::OC,
            CountryCode::MQ => RegionCode::SAM,
            CountryCode::MR => RegionCode::AF,
            CountryCode::MU => RegionCode::AF,
            CountryCode::YT => RegionCode::AF,
            CountryCode::MX => RegionCode::SAM,
            CountryCode::FM => RegionCode::OC,
            CountryCode::MD => RegionCode::EU,
            CountryCode::MC => RegionCode::EU,
            CountryCode::MN => RegionCode::AS,
            CountryCode::ME => RegionCode::EU,
            CountryCode::MS => RegionCode::SAM,
            CountryCode::MA => RegionCode::AF,
            CountryCode::MZ => RegionCode::AF,
            CountryCode::MM => RegionCode::AS,
            CountryCode::NA => RegionCode::AF,
            CountryCode::NR => RegionCode::OC,
            CountryCode::NP => RegionCode::AS,
            CountryCode::NL => RegionCode::EU,
            CountryCode::NC => RegionCode::OC,
            CountryCode::NZ => RegionCode::OC,
            CountryCode::NI => RegionCode::SAM,
            CountryCode::NE => RegionCode::AF,
            CountryCode::NG => RegionCode::AF,
            CountryCode::NU => RegionCode::OC,
            CountryCode::NF => RegionCode::OC,
            CountryCode::MP => RegionCode::OC,
            CountryCode::NO => RegionCode::EU,
            CountryCode::OM => RegionCode::ME,
            CountryCode::PK => RegionCode::AS,
            CountryCode::PW => RegionCode::OC,
            CountryCode::PS => RegionCode::ME,
            CountryCode::PA => RegionCode::SAM,
            CountryCode::PG => RegionCode::OC,
            CountryCode::PY => RegionCode::SAM,
            CountryCode::PE => RegionCode::SAM,
            CountryCode::PH => RegionCode::AS,
            CountryCode::PN => RegionCode::OC,
            CountryCode::PL => RegionCode::EU,
            CountryCode::PT => RegionCode::EU,
            CountryCode::PR => RegionCode::SAM,
            CountryCode::QA => RegionCode::ME,
            CountryCode::RE => RegionCode::AF,
            CountryCode::RO => RegionCode::EU,
            CountryCode::RU => RegionCode::EU,
            CountryCode::RW => RegionCode::AF,
            CountryCode::BL => RegionCode::SAM,
            CountryCode::SH => RegionCode::AF,
            CountryCode::KN => RegionCode::SAM,
            CountryCode::LC => RegionCode::SAM,
            CountryCode::MF => RegionCode::SAM,
            CountryCode::PM => RegionCode::NAM,
            CountryCode::VC => RegionCode::SAM,
            CountryCode::WS => RegionCode::OC,
            CountryCode::SM => RegionCode::EU,
            CountryCode::ST => RegionCode::AF,
            CountryCode::SA => RegionCode::ME,
            CountryCode::SN => RegionCode::AF,
            CountryCode::RS => RegionCode::EU,
            CountryCode::SC => RegionCode::AF,
            CountryCode::SL => RegionCode::AF,
            CountryCode::SG => RegionCode::AS,
            CountryCode::SX => RegionCode::SAM,
            CountryCode::SK => RegionCode::EU,
            CountryCode::SI => RegionCode::EU,
            CountryCode::SB => RegionCode::OC,
            CountryCode::SO => RegionCode::AF,
            CountryCode::ZA => RegionCode::AF,
            CountryCode::GS => RegionCode::SAM,
            CountryCode::SS => RegionCode::AF,
            CountryCode::ES => RegionCode::EU,
            CountryCode::LK => RegionCode::AS,
            CountryCode::SD => RegionCode::AF,
            CountryCode::SR => RegionCode::SAM,
            CountryCode::SJ => RegionCode::EU,
            CountryCode::SE => RegionCode::EU,
            CountryCode::CH => RegionCode::EU,
            CountryCode::SY => RegionCode::ME,
            CountryCode::TW => RegionCode::AS,
            CountryCode::TJ => RegionCode::AS,
            CountryCode::TZ => RegionCode::AF,
            CountryCode::TH => RegionCode::AS,
            CountryCode::TL => RegionCode::AS,
            CountryCode::TG => RegionCode::AF,
            CountryCode::TK => RegionCode::OC,
            CountryCode::TO => RegionCode::OC,
            CountryCode::TT => RegionCode::SAM,
            CountryCode::TN => RegionCode::AF,
            CountryCode::TR => RegionCode::ME,
            CountryCode::TM => RegionCode::AS,
            CountryCode::TC => RegionCode::SAM,
            CountryCode::TV => RegionCode::OC,
            CountryCode::UG => RegionCode::AF,
            CountryCode::UA => RegionCode::EU,
            CountryCode::AE => RegionCode::ME,
            CountryCode::GB => RegionCode::EU,
            CountryCode::US => RegionCode::NAM,
            CountryCode::UM => RegionCode::OC,
            CountryCode::UY => RegionCode::SAM,
            CountryCode::UZ => RegionCode::AS,
            CountryCode::VU => RegionCode::OC,
            CountryCode::VE => RegionCode::SAM,
            CountryCode::VN => RegionCode::AS,
            CountryCode::VG => RegionCode::SAM,
            CountryCode::VI => RegionCode::SAM,
            CountryCode::WF => RegionCode::OC,
            CountryCode::EH => RegionCode::AF,
            CountryCode::YE => RegionCode::ME,
            CountryCode::ZM => RegionCode::AF,
            CountryCode::ZW => RegionCode::AF,
        }
    }
}
