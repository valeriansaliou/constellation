// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use flate2::read::GzDecoder;
use http_req::error as HTTPError;
use http_req::request as HTTPRequest;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::thread;
use std::time::Duration;
use tar::{Archive, Entries};
use tempfile::tempfile;

use super::locate::Locator;
use crate::APP_CONF;

pub struct GeoUpdaterBuilder;
pub struct GeoUpdater;

impl GeoUpdaterBuilder {
    pub fn new() -> GeoUpdater {
        GeoUpdater {}
    }
}

impl GeoUpdater {
    pub fn run(&self) {
        let interval_duration = Duration::from_secs(APP_CONF.geo.update_interval);

        // Acquire update URL
        let update_url = APP_CONF
            .geo
            .update_url
            .to_owned()
            .expect("the geo updater requires 'geo.update_url' to be set in the configuration");

        debug!(
            "scheduled background geo updates every: {:?}",
            interval_duration
        );

        loop {
            // Hold for next poll run
            thread::sleep(interval_duration);

            info!("running a geo update operation...");

            // Hold on 2 seconds
            thread::sleep(Duration::from_secs(2));

            match Self::update_database(&update_url, false) {
                Ok(_) => {
                    info!("ran geo update operation");

                    match Locator::request_geo_refresh() {
                        Ok(_) => info!("refreshed geo reader"),
                        Err(err) => error!("failure to refresh geo reader: {}", err),
                    }
                }
                Err(Some(err)) => error!("failed running geo update operation: {}", err),
                Err(None) => error!("failed running geo update operation (no reason given)"),
            }
        }
    }

    fn extract_archive(entries: Entries<GzDecoder<File>>) -> bool {
        for file_entry in entries {
            if let Ok(mut file) = file_entry {
                // Copy path (ie. own it)
                let file_path = file.path().map(|path| path.into_owned());

                if let Ok(path) = file_path {
                    if path.ends_with(&APP_CONF.geo.database_file) == true {
                        let database_path = Locator::get_database_full_path();

                        match file.unpack(&database_path) {
                            Ok(_) => {
                                info!("unpacked geo database archive to file: {:?}", database_path);

                                return true;
                            }
                            Err(err) => {
                                error!("failed to unpack geo database archive file: {:?}", err);
                            }
                        }
                    }
                }
            }
        }

        return false;
    }

    fn update_database(update_url: &str, redirected: bool) -> Result<(), Option<HTTPError::Error>> {
        debug!("acquiring updated geo database");

        match tempfile() {
            Ok(mut tmp_file) => {
                match HTTPRequest::get(update_url, &mut tmp_file) {
                    Ok(response) => {
                        let status = response.status_code();

                        if status.is_success() {
                            debug!(
                                "downloaded updated geo database archive with status: {} to file: {:?}",
                                response.status_code(),
                                tmp_file
                            );

                            // Reset file cursor to the beginning (prepare for reading)
                            tmp_file.seek(SeekFrom::Start(0)).unwrap();

                            // Extract archive
                            let tar = GzDecoder::new(tmp_file);

                            match Archive::new(tar).entries() {
                                Ok(entries) => {
                                    if Self::extract_archive(entries) == true {
                                        Ok(())
                                    } else {
                                        error!(
                                            "no matching mmdb file found in geo database archive"
                                        );

                                        Err(None)
                                    }
                                }
                                Err(_) => {
                                    error!("failed to list entries in geo database archive");

                                    Err(None)
                                }
                            }
                        } else if status.is_redirect() {
                            // Redirect to target location?
                            // Important: if not already redirected!
                            if let Some(location_url) = response.headers().get("Location") {
                                if !location_url.is_empty() && redirected == false {
                                    info!(
                                        "redirected to download updated geo database archive: {}",
                                        location_url
                                    );

                                    return Self::update_database(location_url, true);
                                }
                            }

                            error!("bad redirection to download updated geo database archive");

                            Err(None)
                        } else {
                            error!(
                                "refused to download updated geo database archive (got status: {})",
                                status
                            );

                            Err(None)
                        }
                    }
                    Err(err) => {
                        error!("failed to download updated geo database archive");

                        Err(Some(err))
                    }
                }
            }
            Err(err) => {
                error!(
                    "failed to create temporary file for geo database download: {:?}",
                    err
                );

                Err(None)
            }
        }
    }
}
