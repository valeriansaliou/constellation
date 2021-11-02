// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::{delete, get, head, patch, post, put, web, web::Data, web::Json, HttpResponse};

#[head("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn head_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(String, String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[get("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn get_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(String, String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[put("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn put_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(String, String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[delete("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn delete_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(String, String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/query/types")]
async fn get_metrics_query_types(
    web::Path((zone_name, metrics_timespan)): web::Path<(String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/query/origins")]
async fn get_metrics_query_origins(
    web::Path((zone_name, metrics_timespan)): web::Path<(String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/answer/codes")]
async fn get_metrics_answer_codes(
    web::Path((zone_name, metrics_timespan)): web::Path<(String, String)>,
) -> HttpResponse {
    // TODO
    HttpResponse::InternalServerError().finish()
}
