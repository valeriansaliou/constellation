// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::http::Status;
use rocket_contrib::json::Json;

use super::record_guard::RecordGuard;
use dns::zone::ZoneName;
use dns::record::{RecordType, RecordName, RecordRegions, RecordValues};
use store::store::StoreRecord;

use APP_STORE;

#[derive(Deserialize)]
pub struct RecordData {
    ttl: Option<u32>,
    regions: Option<RecordRegions>,
    values: RecordValues,
}

#[derive(Serialize)]
pub struct RecordGetResponse {
    #[serde(rename = "type")]
    _type: RecordType,

    name: RecordName,
    ttl: Option<u32>,
    regions: Option<RecordRegions>,
    values: RecordValues,
}

#[head("/zone/<zone_name>/record/<record_name>/<record_type>")]
pub fn head_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
) -> Result<(), Status> {
    APP_STORE.check(&zone_name, &record_name, &record_type).or(
        Err(
            Status::NotFound,
        ),
    )
}

#[get("/zone/<zone_name>/record/<record_name>/<record_type>")]
pub fn get_zone_record(
    _auth: RecordGuard,
    zone_name: ZoneName,
    record_name: RecordName,
    record_type: RecordType,
) -> Result<Json<RecordGetResponse>, Status> {
    APP_STORE
        .get(&zone_name, &record_name, &record_type)
        .map(|record| {
            Json(RecordGetResponse {
                _type: record.kind,
                name: record.name,
                ttl: record.ttl,
                regions: record.regions,
                values: record.values,
            })
        })
        .or(Err(Status::NotFound))
}

#[put("/zone/<zone_name>/record/<record_name>/<record_type>", data = "<data>",
      format = "application/json")]
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
                regions: data.regions.to_owned(),
                values: data.values.to_owned(),
            },
        )
        .or(Err(Status::InternalServerError))
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
        .or(Err(Status::InternalServerError))
}
