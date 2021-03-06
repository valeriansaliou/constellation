// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket;
use rocket::config::{Config, Environment};

use crate::APP_CONF;

use super::{catchers, routes};

pub struct HTTPListenBuilder;
pub struct HTTPListen;

impl HTTPListenBuilder {
    pub fn new() -> HTTPListen {
        HTTPListen {}
    }
}

impl HTTPListen {
    pub fn run(&self) {
        // Build Rocket configuration
        let config = Config::build(Environment::Production)
            .address(APP_CONF.http.inet.ip().to_string())
            .port(APP_CONF.http.inet.port())
            .workers(APP_CONF.http.workers)
            .finalize()
            .unwrap();

        // Build and run Rocket instance
        rocket::custom(config)
            .mount(
                "/",
                routes![
                    // Zone
                    routes::head_zone_record,
                    routes::get_zone_record,
                    routes::put_zone_record,
                    routes::delete_zone_record,
                    // Metrics
                    routes::get_metrics_query_types,
                    routes::get_metrics_query_origins,
                    routes::get_metrics_answer_codes,
                ],
            )
            .register(catchers![
                catchers::bad_request,
                catchers::unauthorized,
                catchers::forbidden,
                catchers::not_found,
                catchers::method_not_allowed,
                catchers::not_acceptable,
                catchers::payload_too_large,
                catchers::internal_server_error,
            ])
            .launch();
    }
}
