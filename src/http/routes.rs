// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::http::Status;
use rocket_contrib::json::Json;
use std::collections::HashMap;

use super::record_guard::RecordGuard;
use crate::dns::metrics::{MetricsStoreCountType, MetricsTimespan, MetricsType, METRICS_STORE};
use crate::dns::record::{RecordBlackhole, RecordName, RecordRegions, RecordType, RecordValues};
use crate::dns::zone::ZoneName;
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

type MetricsGenericGetResponse = HashMap<String, MetricsStoreCountType>;

#[head("/zone/<zone_name>/record/<record_name>/<record_type>")]
pub fn head_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
) -> Result<(), Status> {
    APP_STORE
        .check(&zone_name, &record_name, &record_type)
        .or(Err(Status::NotFound))
}

#[get("/zone/<zone_name>/record/<record_name>/<record_type>")]
pub fn get_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
) -> Result<Json<RecordGetResponse>, Status> {
    APP_STORE
        .get(
            &zone_name,
            &record_name,
            &record_type,
            StoreAccessOrigin::Internal,
        )
        .map(|record| {
            Json(RecordGetResponse {
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
        .or(Err(Status::NotFound))
}

#[put(
    "/zone/<zone_name>/record/<record_name>/<record_type>",
    data = "<data>",
    format = "application/json"
)]
pub fn put_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
    data: Json<RecordData>,
) -> Result<(), Status> {
    APP_STORE
        .set(
            &zone_name,
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
        .or(Err(Status::ServiceUnavailable))
}

#[delete("/zone/<zone_name>/record/<record_name>/<record_type>")]
pub fn delete_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
) -> Result<(), Status> {
    APP_STORE
        .remove(&zone_name, &record_name, &record_type)
        .or(Err(Status::ServiceUnavailable))
}

#[get("/zone/<zone_name>/metrics/<metrics_timespan>/query/types")]
pub fn get_metrics_query_types(
    _auth: RecordGuard,
    zone_name: ZoneName,
    metrics_timespan: MetricsTimespan,
) -> Result<Json<MetricsGenericGetResponse>, Status> {
    METRICS_STORE
        .aggregate(&zone_name, MetricsType::QueryType, metrics_timespan)
        .ok_or(Status::NotFound)
        .map(|aggregated| Json(aggregated))
}

#[get("/zone/<zone_name>/metrics/<metrics_timespan>/query/origins")]
pub fn get_metrics_query_origins(
    _auth: RecordGuard,
    zone_name: ZoneName,
    metrics_timespan: MetricsTimespan,
) -> Result<Json<MetricsGenericGetResponse>, Status> {
    METRICS_STORE
        .aggregate(&zone_name, MetricsType::QueryOrigin, metrics_timespan)
        .ok_or(Status::NotFound)
        .map(|aggregated| Json(aggregated))
}

#[get("/zone/<zone_name>/metrics/<metrics_timespan>/answer/codes")]
pub fn get_metrics_answer_codes(
    _auth: RecordGuard,
    zone_name: ZoneName,
    metrics_timespan: MetricsTimespan,
) -> Result<Json<MetricsGenericGetResponse>, Status> {
    METRICS_STORE
        .aggregate(&zone_name, MetricsType::AnswerCode, metrics_timespan)
        .ok_or(Status::NotFound)
        .map(|aggregated| Json(aggregated))
}
