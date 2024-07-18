// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod config;
mod dns;
mod geo;
mod http;
mod store;

use std::ops::Deref;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use clap::{Arg, Command};
use log::LevelFilter;

use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use dns::flatten::{DNSFlattenBootstrapBuilder, DNSFlattenMaintainBuilder};
use dns::health::DNSHealthBuilder;
use dns::listen::DNSListenBuilder;
use dns::metrics::DNSMetricsTickBuilder;
use geo::locate::DB_READER;
use geo::updater::GeoUpdaterBuilder;
use http::listen::HTTPListenBuilder;
use store::flush::StoreFlushBuilder;
use store::store::{Store, StoreBuilder};

struct AppArgs {
    config: String,
}

pub static THREAD_NAME_DNS: &'static str = "constellation-dns";
pub static THREAD_NAME_HTTP: &'static str = "constellation-http";
pub static THREAD_NAME_STORE_FLUSH: &'static str = "constellation-store-flush";
pub static THREAD_NAME_DNS_METRICS: &'static str = "constellation-dns-metrics";
pub static THREAD_NAME_DNS_HEALTH: &'static str = "constellation-dns-health";
pub static THREAD_NAME_DNS_FLATTEN_BOOTSTRAP: &'static str = "constellation-dns-flatten-bootstrap";
pub static THREAD_NAME_DNS_FLATTEN_MAINTAIN: &'static str = "constellation-dns-flatten-maintain";
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
    "store_flush",
    spawn_store_flush,
    THREAD_NAME_STORE_FLUSH,
    StoreFlushBuilder::new().run()
);
gen_spawn_managed!(
    "dns_metrics",
    spawn_dns_metrics,
    THREAD_NAME_DNS_METRICS,
    DNSMetricsTickBuilder::new().run()
);
gen_spawn_managed!(
    "dns_health",
    spawn_dns_health,
    THREAD_NAME_DNS_HEALTH,
    DNSHealthBuilder::new().run()
);
gen_spawn_managed!(
    "dns_flatten_maintain",
    spawn_dns_flatten_maintain,
    THREAD_NAME_DNS_FLATTEN_MAINTAIN,
    DNSFlattenBootstrapBuilder::new().run()
);
gen_spawn_managed!(
    "dns_flatten_bootstrap",
    spawn_dns_flatten_bootstrap,
    THREAD_NAME_DNS_FLATTEN_BOOTSTRAP,
    DNSFlattenMaintainBuilder::new().run()
);
gen_spawn_managed!(
    "geo_updater",
    spawn_geo_updater,
    THREAD_NAME_GEO_UPDATER,
    GeoUpdaterBuilder::new().run()
);

fn make_app_args() -> AppArgs {
    let matches = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .default_value("./config.cfg"),
        )
        .get_matches();

    // Generate owned app arguments
    AppArgs {
        config: matches
            .get_one::<String>("config")
            .expect("invalid config value")
            .to_owned(),
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

    // Ensure that there is at least a DNS flatten resolver configured
    assert_eq!(
        APP_CONF.dns.flatten.resolvers.is_empty(),
        false,
        "dns flatten resolver list is empty, please provide at least a resolver in [{}]",
        "dns.flatten.resolvers"
    );
}

fn main() {
    let _logger = ConfigLogger::init(
        LevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Spawn store flush
    thread::spawn(spawn_store_flush);

    // Spawn DNS metrics
    thread::spawn(spawn_dns_metrics);

    // Spawn DNS flattener
    thread::spawn(spawn_dns_flatten_bootstrap);
    thread::spawn(spawn_dns_flatten_maintain);

    // Spawn DNS health checker? (background thread)
    if APP_CONF.dns.health.check_enable == true {
        thread::spawn(spawn_dns_health);
    }

    // Spawn geo updater? (background thread)
    if APP_CONF.geo.update_enable == true {
        thread::spawn(spawn_geo_updater);
    }

    // Run DNS server (background thread)
    thread::spawn(spawn_dns);

    // Spawn HTTP server (foreground thread)
    spawn_http();

    error!("could not start");
}
