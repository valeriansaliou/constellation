// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_web::{delete, get, head, put, web, HttpResponse};

use crate::dns::metrics::{MetricsTimespan, MetricsType, METRICS_STORE};
use crate::dns::record::{RecordBlackhole, RecordName, RecordRegions, RecordType, RecordValues};
use crate::dns::zone::ZoneNameExists;
use crate::store::store::{StoreAccessOrigin, StoreRecord};
use crate::APP_STORE;

#[derive(Deserialize)]
pub struct RecordData {
    ttl: Option<u32>,
    flatten: Option<bool>,
    blackhole: Option<RecordBlackhole>,
    regions: Option<RecordRegions>,
    rescue: Option<RecordValues>,
    values: RecordValues,
}

#[derive(Serialize)]
pub struct RecordGetResponse {
    #[serde(rename = "type")]
    _type: RecordType,

    name: RecordName,
    ttl: Option<u32>,
    flatten: Option<bool>,
    blackhole: Option<RecordBlackhole>,
    regions: Option<RecordRegions>,
    rescue: Option<RecordValues>,
    values: RecordValues,
}

#[head("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn head_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(
        ZoneNameExists,
        RecordName,
        RecordType,
    )>,
) -> HttpResponse {
    APP_STORE
        .get(
            &zone_name.into_inner(),
            &record_name,
            &record_type,
            StoreAccessOrigin::Internal,
        )
        .map(|_| HttpResponse::Ok().finish())
        .unwrap_or(HttpResponse::NotFound().finish())
}

#[get("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn get_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(
        ZoneNameExists,
        RecordName,
        RecordType,
    )>,
) -> HttpResponse {
    APP_STORE
        .get(
            &zone_name.into_inner(),
            &record_name,
            &record_type,
            StoreAccessOrigin::Internal,
        )
        .map(|record| {
            HttpResponse::Ok().json(RecordGetResponse {
                _type: record.kind,
                name: record.name,
                ttl: record.ttl,
                flatten: record.flatten,
                blackhole: record.blackhole,
                regions: record.regions,
                rescue: record.rescue,
                values: record.values,
            })
        })
        .unwrap_or(HttpResponse::NotFound().finish())
}

#[put("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn put_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(
        ZoneNameExists,
        RecordName,
        RecordType,
    )>,

    data: web::Json<RecordData>,
) -> HttpResponse {
    APP_STORE
        .set(
            &zone_name.into_inner(),
            StoreRecord {
                kind: record_type,
                name: record_name,
                ttl: data.ttl,
                flatten: data.flatten,
                blackhole: data.blackhole.to_owned(),
                regions: data.regions.to_owned(),
                rescue: data.rescue.to_owned(),
                values: data.values.to_owned(),
            },
        )
        .map(|_| HttpResponse::Ok().finish())
        .unwrap_or(HttpResponse::ServiceUnavailable().finish())
}

#[delete("/zone/{zone_name}/record/{record_name}/{record_type}")]
async fn delete_zone_record(
    web::Path((zone_name, record_name, record_type)): web::Path<(
        ZoneNameExists,
        RecordName,
        RecordType,
    )>,
) -> HttpResponse {
    APP_STORE
        .remove(&zone_name.into_inner(), &record_name, &record_type)
        .map(|_| HttpResponse::Ok().finish())
        .unwrap_or(HttpResponse::ServiceUnavailable().finish())
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/query/types")]
async fn get_metrics_query_types(
    web::Path((zone_name, metrics_timespan)): web::Path<(ZoneNameExists, MetricsTimespan)>,
) -> HttpResponse {
    METRICS_STORE
        .aggregate(
            &zone_name.into_inner(),
            MetricsType::QueryType,
            metrics_timespan,
        )
        .map(|aggregated| HttpResponse::Ok().json(aggregated))
        .unwrap_or(HttpResponse::NotFound().finish())
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/query/origins")]
async fn get_metrics_query_origins(
    web::Path((zone_name, metrics_timespan)): web::Path<(ZoneNameExists, MetricsTimespan)>,
) -> HttpResponse {
    METRICS_STORE
        .aggregate(
            &zone_name.into_inner(),
            MetricsType::QueryOrigin,
            metrics_timespan,
        )
        .map(|aggregated| HttpResponse::Ok().json(aggregated))
        .unwrap_or(HttpResponse::NotFound().finish())
}

#[get("/zone/{zone_name}/metrics/{metrics_timespan}/answer/codes")]
async fn get_metrics_answer_codes(
    web::Path((zone_name, metrics_timespan)): web::Path<(ZoneNameExists, MetricsTimespan)>,
) -> HttpResponse {
    METRICS_STORE
        .aggregate(
            &zone_name.into_inner(),
            MetricsType::AnswerCode,
            metrics_timespan,
        )
        .map(|aggregated| HttpResponse::Ok().json(aggregated))
        .unwrap_or(HttpResponse::NotFound().finish())
}
