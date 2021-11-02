// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde_json;
use actix_web::http::{self, header::ContentType, StatusCode};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{
    body::{Body, ResponseBody},
    dev::ServiceResponse,
    web, App, HttpRequest, HttpResponse, Result,
};

#[derive(Serialize)]
pub struct CatcherResponse {
    error: &'static str,
}

pub struct HTTPCatchers;

impl HTTPCatchers {
    pub fn errors() -> ErrorHandlers<Body> {
        ErrorHandlers::new()
            .handler(
                StatusCode::BAD_REQUEST,
                Self::catch_bad_request,
            )
            .handler(
                StatusCode::UNAUTHORIZED,
                Self::catch_unauthorized,
            )
            .handler(
                StatusCode::FORBIDDEN,
                Self::catch_forbidden,
            )
            .handler(
                StatusCode::NOT_FOUND,
                Self::catch_not_found,
            )
            .handler(
                StatusCode::METHOD_NOT_ALLOWED,
                Self::catch_method_not_allowed,
            )
            .handler(
                StatusCode::NOT_ACCEPTABLE,
                Self::catch_not_acceptable,
            )
            .handler(
                StatusCode::PAYLOAD_TOO_LARGE,
                Self::catch_payload_too_large,
            )
            .handler(
                StatusCode::INTERNAL_SERVER_ERROR,
                Self::catch_internal_server_error,
            )
    }

    fn respond<B>(
        mut response: ServiceResponse<B>,
        reason: &'static str,
    ) -> Result<ErrorHandlerResponse<B>> {
        // Insert JSON MIME type
        response.response_mut().headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );

        // Map new error body
        let body_json = serde_json::to_string(&CatcherResponse {
            error: reason
        }).expect("could not serialize catcher json body");

        let error: ServiceResponse<B> = response
            .map_body(|_, _| ResponseBody::Other(Body::Message(Box::new(body_json))));

        Ok(ErrorHandlerResponse::Response(error))
    }

    fn catch_bad_request<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "bad_request")
    }

    fn catch_unauthorized<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "unauthorized")
    }

    fn catch_forbidden<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "forbidden")
    }

    fn catch_not_found<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "not_found")
    }

    fn catch_method_not_allowed<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "method_not_allowed")
    }

    fn catch_not_acceptable<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "not_acceptable")
    }

    fn catch_payload_too_large<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "payload_too_large")
    }

    fn catch_internal_server_error<B>(
        mut response: ServiceResponse<B>,
    ) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "internal_server_error")
    }
}
