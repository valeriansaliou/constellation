// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::{
    dev::ServiceRequest,
    guard,
    middleware::{self, normalize::TrailingSlash},
    rt, web, App, Error as ActixError, HttpServer,
};
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
        let mut runtime = rt::System::new("http");

        // TODO: routes
        // TODO: error catchers
        // TODO: restore last missed things?

        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
                .wrap(HttpAuthentication::basic(authenticate))
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
) -> Result<ServiceRequest, ActixError> {
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

        Err(error.into())
    }
}
