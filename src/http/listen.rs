// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket;
use rocket::config::{Config, Environment};
use APP_CONF;

use super::{routes, catchers};

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
        rocket::custom(config, false)
            .mount(
                "/",
                routes![
                    routes::head_record,
                    routes::get_record,
                    routes::put_record,
                    routes::delete_record,
                ],
            )
            .catch(errors![
                catchers::bad_request,
                catchers::unauthorized,
                catchers::forbidden,
                catchers::not_found,
                catchers::internal_server_error,
            ])
            .launch();
    }
}
