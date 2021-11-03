// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use maxminddb::{geoip2, MaxMindDBError, Reader as GeoReader};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use super::country::CountryCode;
use crate::APP_CONF;

type GeoReaderType = GeoReader<Vec<u8>>;

pub struct Locator;

lazy_static! {
    pub static ref DB_READER: Arc<RwLock<GeoReaderType>> =
        Arc::new(RwLock::new(Locator::geo_open()));
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

                info!("geo database refreshed");

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn geo_acquire() -> Result<GeoReaderType, MaxMindDBError> {
        let database_path = Self::get_database_full_path();

        debug!("acquiring geo database at: {:?}", database_path);

        GeoReader::open_readfile(database_path.to_str().unwrap())
    }

    fn geo_open() -> GeoReaderType {
        match Self::geo_acquire() {
            Ok(reader) => {
                info!("geo database opened");

                reader
            }
            Err(_) => {
                panic!("geo database not found; download geolite2 country to [geo.database_path]");
            }
        }
    }
}
