// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_export]
macro_rules! serde_string_impls {
    ($Target:ident, $from_method:ident) => {
        impl Serialize for $Target {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self.to_str())
            }
        }

        impl<'d> Deserialize<'d> for $Target {
            fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<$Target, D::Error> {
                struct FieldVisitor;

                impl<'d> Visitor<'d> for FieldVisitor {
                    type Value = $Target;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a valid value")
                    }

                    fn visit_str<E: DeserializeError>(self, value: &str) -> Result<Self::Value, E> {
                        match $Target::$from_method(value) {
                            Some(inner) => Ok(inner),
                            None => Err(E::invalid_type(Unexpected::Enum, &self)),
                        }
                    }
                }

                deserializer.deserialize_str(FieldVisitor)
            }
        }
    };
}

macro_rules! gen_record_type_impls {
    ($($TypeEnum:ident -> $TypeString:expr),+,) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub enum RecordType {
            $($TypeEnum,)+
        }

        impl RecordType {
            pub fn from_str(value: &str) -> Option<RecordType> {
                match value {
                    $(
                        $TypeString => Some(RecordType::$TypeEnum),
                    )+
                    _ => None,
                }
            }

            pub fn from_hickory(record_type: &HickoryRecordType) -> Option<RecordType> {
                match record_type {
                    $(
                        &HickoryRecordType::$TypeEnum => Some(RecordType::$TypeEnum),
                    )+
                    _ => None,
                }
            }

            pub fn to_str(&self) -> &'static str {
                match *self {
                    $(
                        RecordType::$TypeEnum => $TypeString,
                    )+
                }
            }

            pub fn to_hickory(&self) -> Result<HickoryRecordType, ()> {
                match *self {
                    $(
                        RecordType::$TypeEnum => Ok(HickoryRecordType::$TypeEnum),
                    )+
                }
            }

            pub fn list_choices() -> Vec<RecordType> {
                return vec![
                    $(RecordType::$TypeEnum,)+
                ];
            }
        }
    }
}

macro_rules! gen_metrics_tick_perform_item {
    ($Store:ident, $Backlog:ident) => {
        // Move all minutes up in the list (sliding window)
        for index in (0..$Backlog).rev() {
            $Store[index + 1] = $Store[index].clone();
        }

        // Start a new 'minute' (this begins a new timespan storage for the current minute)
        $Store[0] = HashMap::new();
    };
}

macro_rules! gen_metrics_stack_item {
    ($Counters:ident, $Key:ident) => {
        if let Some(count) = $Counters.get_mut($Key) {
            *count += 1;
        } else {
            $Counters.insert($Key.to_owned(), 1);
        }
    };
}

macro_rules! gen_metrics_aggregate_item {
    ($Store:ident, $Limit:ident, $Backlog:ident) => {{
        let mut aggregated_map = HashMap::new();

        for index in 1..(min($Limit as usize, $Backlog) + 1) {
            let point_map = &$Store[index];

            for (key, count) in point_map.iter() {
                let final_key = if let Some(key_inner) = key {
                    key_inner.to_str().to_lowercase()
                } else {
                    "other".to_string()
                };

                // Increment existing count, or insert
                if let Some(existing_count) = aggregated_map.get_mut(&final_key) {
                    *existing_count = *existing_count + *count
                } else {
                    aggregated_map.insert(final_key, *count);
                }
            }
        }

        aggregated_map
    }};
}
