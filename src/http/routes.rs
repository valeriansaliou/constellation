// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::response::Failure;
use rocket::http::Status;
use rocket_contrib::Json;

use super::record_guard::RecordGuard;
use dns::record::RecordType;
use store::store::StoreRecord;

use APP_STORE;

#[derive(Deserialize)]
pub struct RecordData {
    ttl: u32,
    value: String,
}

#[derive(Serialize)]
pub struct RecordGetResponse {
    #[serde(rename = "type")]
    _type: RecordType,

    name: String,
    ttl: u32,
    value: String,
}

#[head("/record/<record_name>/<record_type>")]
fn head_record(
    _auth: RecordGuard,
    record_name: String,
    record_type: RecordType,
) -> Result<(), Failure> {
    APP_STORE.check(&record_name, record_type).or(Err(Failure(
        Status::NotFound,
    )))
}

#[get("/record/<record_name>/<record_type>")]
fn get_record(
    _auth: RecordGuard,
    record_name: String,
    record_type: RecordType,
) -> Result<Json<RecordGetResponse>, Failure> {
    APP_STORE.get(&record_name, record_type).map(|record| {
        Json(RecordGetResponse {
            _type: record.kind,
            name: record.name,
            ttl: record.ttl,
            value: record.value,
        })
    }).or(Err(Failure(
        Status::NotFound,
    )))
}

#[put("/record/<record_name>/<record_type>", data = "<data>", format = "application/json")]
fn put_record(
    _auth: RecordGuard,
    record_name: String,
    record_type: RecordType,
    data: Json<RecordData>,
) -> Result<(), Failure> {
    APP_STORE.set(StoreRecord {
        kind: record_type,
        name: record_name,
        ttl: data.ttl,
        value: data.value.to_owned(),
    }).or(Err(Failure(
        Status::InternalServerError,
    )))
}

#[delete("/record/<record_name>/<record_type>")]
fn delete_record(
    _auth: RecordGuard,
    record_name: String,
    record_type: RecordType,
) -> Result<(), Failure> {
    APP_STORE.remove(&record_name, record_type).or(Err(Failure(
        Status::InternalServerError,
    )))
}
