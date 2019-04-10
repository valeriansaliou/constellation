// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket;
extern crate base64;
extern crate rand;
extern crate farmhash;
extern crate flate2;
extern crate http_req;
extern crate maxminddb;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate regex;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate tar;
extern crate tempfile;
extern crate toml;
extern crate trust_dns;
extern crate trust_dns_server;

mod config;
mod dns;
mod geo;
mod http;
mod store;

use std::ops::Deref;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use log::LevelFilter;

use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use dns::listen::DNSListenBuilder;
use geo::locate::DB_READER;
use geo::updater::GeoUpdaterBuilder;
use http::listen::HTTPListenBuilder;
use store::store::{Store, StoreBuilder};

struct AppArgs {
    config: String,
}

pub static THREAD_NAME_DNS: &'static str = "constellation-dns";
pub static THREAD_NAME_HTTP: &'static str = "constellation-http";
pub static THREAD_NAME_GEO_UPDATER: &'static str = "constellation-geo-updater";

macro_rules! gen_spawn_managed {
    ($name:expr, $method:ident, $thread_name:ident, $managed_fn:expr) => {
        fn $method() {
            debug!("spawn managed thread: {}", $name);

            let worker = thread::Builder::new()
                .name($thread_name.to_string())
                .spawn(|| $managed_fn);

            // Block on worker thread (join it)
            let has_error = if let Ok(worker_thread) = worker {
                worker_thread.join().is_err()
            } else {
                true
            };

            // Worker thread crashed?
            if has_error == true {
                error!("managed thread crashed ({}), setting it up again", $name);

                // Prevents thread start loop floods
                thread::sleep(Duration::from_secs(2));

                $method();
            }
        }
    };
}

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
    static ref APP_STORE: Store = StoreBuilder::new();
}

gen_spawn_managed!(
    "dns",
    spawn_dns,
    THREAD_NAME_DNS,
    DNSListenBuilder::new().run()
);
gen_spawn_managed!(
    "http",
    spawn_http,
    THREAD_NAME_HTTP,
    HTTPListenBuilder::new().run()
);
gen_spawn_managed!(
    "geo_updater",
    spawn_geo_updater,
    THREAD_NAME_GEO_UPDATER,
    GeoUpdaterBuilder::new().run()
);

fn make_app_args() -> AppArgs {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Path to configuration file")
                .default_value("./config.cfg")
                .takes_value(true),
        )
        .get_matches();

    // Generate owned app arguments
    AppArgs {
        config: String::from(matches.value_of("config").expect("invalid config value")),
    }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    let (_, _, _, _) = (
        APP_ARGS.deref(),
        APP_CONF.deref(),
        APP_STORE.deref(),
        DB_READER.deref(),
    );
}

fn main() {
    let _logger = ConfigLogger::init(
        LevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Spawn geo updater? (background thread)
    if APP_CONF.geo.update_enable == true {
        thread::spawn(spawn_geo_updater);
    }

    // Spawn HTTP server (background thread)
    thread::spawn(spawn_http);

    // Run DNS server (from main thread, maintain thread active if down)
    spawn_dns();

    error!("could not start");
}
