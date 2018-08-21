// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::RwLock;
use std::sync::Arc;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use maxminddb::{geoip2, Reader as GeoReader, MaxMindDBError};

use super::country::CountryCode;
use APP_CONF;

pub struct Locator;

lazy_static! {
    pub static ref DB_READER: Arc<RwLock<GeoReader>> = Arc::new(RwLock::new(Locator::geo_open()));
}

impl Locator {
    pub fn ip_to_country(ip: IpAddr) -> Option<CountryCode> {
        if let Ok(ref reader) = DB_READER.read() {
            // Lookup IP address to country
            if let Ok(result) = reader.lookup::<geoip2::Country>(ip) {
                // Country found?
                if let Some(country) = result.country {
                    if let Some(iso_code) = country.iso_code {
                        if let Some(country_code) = CountryCode::from_str(&iso_code) {
                            return Some(country_code);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_database_full_path() -> PathBuf {
        Path::new(&APP_CONF.geo.database_path).join(&APP_CONF.geo.database_file)
    }

    pub fn request_geo_refresh() -> Result<(), MaxMindDBError> {
        match Self::geo_acquire() {
            Ok(reader) => {
                let mut store = DB_READER.write().unwrap();

                *store = reader;

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn geo_acquire() -> Result<GeoReader, MaxMindDBError> {
        GeoReader::open(Self::get_database_full_path().to_str().unwrap())
    }

    fn geo_open() -> GeoReader {
        Self::geo_acquire().expect(
            "geo database not found; download geolite2 country and store it in [geo.database_path]",
        )
    }
}
