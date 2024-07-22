// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::dev::ServiceRequest;
use actix_web::middleware::NormalizePath;
use actix_web::{rt, App, Error as ActixError, HttpServer};
use actix_web_httpauth::{
    extractors::{
        basic::{BasicAuth, Config as ConfigAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};

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
        let runtime = rt::System::new();

        let server = HttpServer::new(|| {
            App::new()
                .wrap(NormalizePath::trim())
                .wrap(HttpAuthentication::basic(authenticate))
                .wrap(catchers::HTTPCatchers::errors())
                .service(routes::head_zone_record)
                .service(routes::get_zone_record)
                .service(routes::put_zone_record)
                .service(routes::delete_zone_record)
                .service(routes::get_metrics_query_types)
                .service(routes::get_metrics_query_origins)
                .service(routes::get_metrics_answer_codes)
        })
        .workers(APP_CONF.http.workers)
        .bind(APP_CONF.http.inet)
        .unwrap()
        .run();

        runtime.block_on(server).unwrap()
    }
}

async fn authenticate(
    request: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    let password = if let Some(password) = credentials.password() {
        &*password
    } else {
        ""
    };

    if password == APP_CONF.http.record_token {
        Ok(request)
    } else {
        let mut error = AuthenticationError::from(
            request
                .app_data::<ConfigAuth>()
                .map(|data| data.clone())
                .unwrap_or_else(ConfigAuth::default),
        );

        *error.status_code_mut() = actix_web::http::StatusCode::FORBIDDEN;

        Err((error.into(), request))
    }
}
