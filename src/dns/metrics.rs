// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::de::{Error as DeserializeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::{fmt, str};

use super::code::CodeName;
use super::record::RecordType;
use super::zone::ZoneName;
use crate::geo::country::CountryCode;
use crate::APP_CONF;

lazy_static! {
    pub static ref METRICS_STORE: MetricsStore = MetricsStoreBuilder::new();
}

const METRICS_TICK_INTERVAL: Duration = Duration::from_secs(60);
const METRICS_BACKLOG_MINUTES: usize = 15;

const METRICS_TIMESPAN_ONE_MINUTE: &'static str = "1m";
const METRICS_TIMESPAN_FIVE_MINUTES: &'static str = "5m";
const METRICS_TIMESPAN_FIFTEEN_MINUTES: &'static str = "15m";

serde_string_impls!(MetricsTimespan, from_str);

pub type MetricsStoreCountType = u32;

pub struct DNSMetricsTickBuilder;
pub struct DNSMetricsTick;

pub type MetricsStoreQueryTypeType = HashMap<Option<RecordType>, MetricsStoreCountType>;
pub type MetricsStoreQueryOriginType = HashMap<Option<CountryCode>, MetricsStoreCountType>;
pub type MetricsStoreAnswerCodeType = HashMap<Option<CodeName>, MetricsStoreCountType>;

struct MetricsStoreBuilder;
struct MetricsStoreZoneBuilder;

pub struct MetricsStore {
    pub zones: RwLock<HashMap<ZoneName, MetricsStoreZone>>,
}

#[derive(Default)]
pub struct MetricsStoreZone {
    pub query_type: [MetricsStoreQueryTypeType; METRICS_BACKLOG_MINUTES + 1],
    pub query_origin: [MetricsStoreQueryOriginType; METRICS_BACKLOG_MINUTES + 1],
    pub answer_code: [MetricsStoreAnswerCodeType; METRICS_BACKLOG_MINUTES + 1],
}

pub enum MetricsType {
    QueryType,
    QueryOrigin,
    AnswerCode,
}

pub enum MetricsValue<'a> {
    QueryType(&'a Option<RecordType>),
    QueryOrigin(&'a Option<CountryCode>),
    AnswerCode(&'a Option<CodeName>),
}

impl MetricsStoreBuilder {
    pub fn new() -> MetricsStore {
        MetricsStore {
            zones: RwLock::new(HashMap::new()),
        }
    }
}

impl MetricsStoreZoneBuilder {
    pub fn new() -> MetricsStoreZone {
        MetricsStoreZone::default()
    }
}

pub enum MetricsTimespan {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
}

impl DNSMetricsTickBuilder {
    pub fn new() -> DNSMetricsTick {
        DNSMetricsTick {}
    }
}

impl DNSMetricsTick {
    pub fn run(&self) {
        info!("metrics ticker is now active");

        loop {
            // Hold for next tick run
            thread::sleep(METRICS_TICK_INTERVAL);

            debug!("running a metrics tick...");

            Self::perform();

            info!("ran metrics tick");
        }
    }

    fn perform() {
        // Perform a sliding window on all store maps
        let mut zones_write = METRICS_STORE.zones.write().unwrap();

        for (_, zone_store) in zones_write.iter_mut() {
            let (store_query_type, store_query_origin, store_answer_code) = (
                &mut zone_store.query_type,
                &mut zone_store.query_origin,
                &mut zone_store.answer_code,
            );

            gen_metrics_tick_perform_item!(store_query_type, METRICS_BACKLOG_MINUTES);
            gen_metrics_tick_perform_item!(store_query_origin, METRICS_BACKLOG_MINUTES);
            gen_metrics_tick_perform_item!(store_answer_code, METRICS_BACKLOG_MINUTES);
        }
    }
}

impl MetricsStore {
    pub fn stack(&self, zone_name: &ZoneName, metrics_value: MetricsValue) {
        let mut zones_write = self.zones.write().unwrap();

        // Initialize store if not found?
        if zones_write.contains_key(zone_name) == false {
            zones_write.insert(zone_name.to_owned(), MetricsStoreZoneBuilder::new());
        }

        // Acquire store for zone
        if let Some(zone_store) = zones_write.get_mut(zone_name) {
            match metrics_value {
                MetricsValue::QueryType(record_type) => {
                    self.stack_query_type(zone_store, record_type)
                }
                MetricsValue::QueryOrigin(origin_country) => {
                    self.stack_query_origin(zone_store, origin_country)
                }
                MetricsValue::AnswerCode(code) => self.stack_answer_code(zone_store, code),
            }
        }
    }

    pub fn aggregate(
        &self,
        zone_name: &ZoneName,
        metrics_type: MetricsType,
        metrics_timespan: MetricsTimespan,
    ) -> Option<HashMap<String, MetricsStoreCountType>> {
        if APP_CONF.dns.zone_exists(zone_name.to_str()) == true {
            let aggregated_map: HashMap<String, MetricsStoreCountType>;

            let zone_read = METRICS_STORE.zones.read().unwrap();

            if let Some(zone_store) = zone_read.get(&zone_name) {
                let aggregate_limit = metrics_timespan.as_minutes();

                match metrics_type {
                    MetricsType::QueryType => {
                        aggregated_map = self.aggregate_query_type(zone_store, aggregate_limit);
                    }
                    MetricsType::QueryOrigin => {
                        aggregated_map = self.aggregate_query_origin(zone_store, aggregate_limit);
                    }
                    MetricsType::AnswerCode => {
                        aggregated_map = self.aggregate_answer_code(zone_store, aggregate_limit);
                    }
                }
            } else {
                aggregated_map = HashMap::new();
            }

            Some(aggregated_map)
        } else {
            None
        }
    }

    fn stack_query_type(&self, store: &mut MetricsStoreZone, record_type: &Option<RecordType>) {
        debug!(
            "stacking query type metric for record type: {:?}",
            record_type
        );

        let query_type_counters = &mut store.query_type[0];

        gen_metrics_stack_item!(query_type_counters, record_type);
    }

    fn stack_query_origin(
        &self,
        store: &mut MetricsStoreZone,
        origin_country: &Option<CountryCode>,
    ) {
        debug!(
            "stacking query origin metric for origin country: {:?}",
            origin_country
        );

        let query_origin_counters = &mut store.query_origin[0];

        gen_metrics_stack_item!(query_origin_counters, origin_country);
    }

    fn stack_answer_code(&self, store: &mut MetricsStoreZone, code: &Option<CodeName>) {
        debug!("stacking answer code metric for response code: {:?}", code);

        let answer_code_counters = &mut store.answer_code[0];

        gen_metrics_stack_item!(answer_code_counters, code);
    }

    fn aggregate_query_type(
        &self,
        store: &MetricsStoreZone,
        aggregate_limit: u8,
    ) -> HashMap<String, MetricsStoreCountType> {
        let store_target = &store.query_type;

        gen_metrics_aggregate_item!(store_target, aggregate_limit, METRICS_BACKLOG_MINUTES)
    }

    fn aggregate_query_origin(
        &self,
        store: &MetricsStoreZone,
        aggregate_limit: u8,
    ) -> HashMap<String, MetricsStoreCountType> {
        let store_target = &store.query_origin;

        gen_metrics_aggregate_item!(store_target, aggregate_limit, METRICS_BACKLOG_MINUTES)
    }

    fn aggregate_answer_code(
        &self,
        store: &MetricsStoreZone,
        aggregate_limit: u8,
    ) -> HashMap<String, MetricsStoreCountType> {
        let store_target = &store.answer_code;

        gen_metrics_aggregate_item!(store_target, aggregate_limit, METRICS_BACKLOG_MINUTES)
    }
}

impl MetricsTimespan {
    pub fn to_str(&self) -> &str {
        match self {
            MetricsTimespan::OneMinute => METRICS_TIMESPAN_ONE_MINUTE,
            MetricsTimespan::FiveMinutes => METRICS_TIMESPAN_FIVE_MINUTES,
            MetricsTimespan::FifteenMinutes => METRICS_TIMESPAN_FIFTEEN_MINUTES,
        }
    }

    pub fn from_str(value: &str) -> Option<MetricsTimespan> {
        match value {
            METRICS_TIMESPAN_ONE_MINUTE => Some(MetricsTimespan::OneMinute),
            METRICS_TIMESPAN_FIVE_MINUTES => Some(MetricsTimespan::FiveMinutes),
            METRICS_TIMESPAN_FIFTEEN_MINUTES => Some(MetricsTimespan::FifteenMinutes),
            _ => None,
        }
    }

    pub fn as_minutes(&self) -> u8 {
        match self {
            MetricsTimespan::OneMinute => 1,
            MetricsTimespan::FiveMinutes => 5,
            MetricsTimespan::FifteenMinutes => 15,
        }
    }
}
