// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::header::{HeaderValue, CONTENT_TYPE};
use actix_web::http::StatusCode as Status;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::Result;
use serde_json;

#[derive(Serialize)]
pub struct CatcherResponse {
    error: &'static str,
}

pub struct HTTPCatchers;

impl HTTPCatchers {
    pub fn errors() -> ErrorHandlers<EitherBody<BoxBody>> {
        ErrorHandlers::new()
            .handler(Status::BAD_REQUEST, Self::bad_request)
            .handler(Status::UNAUTHORIZED, Self::unauthorized)
            .handler(Status::FORBIDDEN, Self::forbidden)
            .handler(Status::NOT_FOUND, Self::not_found)
            .handler(Status::METHOD_NOT_ALLOWED, Self::method_not_allowed)
            .handler(Status::NOT_ACCEPTABLE, Self::not_acceptable)
            .handler(Status::PAYLOAD_TOO_LARGE, Self::payload_too_large)
            .handler(Status::INTERNAL_SERVER_ERROR, Self::internal_server_error)
    }

    fn respond<B>(
        mut response: ServiceResponse<B>,
        reason: &'static str,
    ) -> Result<ErrorHandlerResponse<B>> {
        // Insert JSON MIME type
        response
            .response_mut()
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Map new error body
        let body_json = serde_json::to_string(&CatcherResponse { error: reason })
            .expect("could not serialize catcher json body");

        let error_response = response.map_body(|_, _| BoxBody::new(body_json));

        Ok(ErrorHandlerResponse::Response(
            error_response.map_into_right_body(),
        ))
    }

    fn bad_request<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "bad_request")
    }

    fn unauthorized<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "unauthorized")
    }

    fn forbidden<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "forbidden")
    }

    fn not_found<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "not_found")
    }

    fn method_not_allowed<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "method_not_allowed")
    }

    fn not_acceptable<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "not_acceptable")
    }

    fn payload_too_large<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "payload_too_large")
    }

    fn internal_server_error<B>(response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
        Self::respond(response, "internal_server_error")
    }
}
