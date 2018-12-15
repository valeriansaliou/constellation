// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket_contrib::json::Json;

#[derive(Serialize)]
pub struct CatcherResponse {
    error: &'static str,
}

#[catch(400)]
pub fn bad_request() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "bad_request" })
}

#[catch(401)]
pub fn unauthorized() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "unauthorized" })
}

#[catch(403)]
pub fn forbidden() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "forbidden" })
}

#[catch(404)]
pub fn not_found() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "not_found" })
}

#[catch(405)]
pub fn method_not_allowed() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "method_not_allowed" })
}

#[catch(406)]
pub fn not_acceptable() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "not_acceptable" })
}

#[catch(413)]
pub fn payload_too_large() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "payload_too_large" })
}

#[catch(500)]
pub fn internal_server_error() -> Json<CatcherResponse> {
    Json(CatcherResponse { error: "internal_server_error" })
}
