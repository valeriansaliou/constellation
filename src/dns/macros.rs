// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_export]
macro_rules! serde_string_impls {
    ($Target:ident) => {
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
                        match $Target::from_str(value) {
                            Some(inner) => Ok(inner),
                            None => Err(E::invalid_type(Unexpected::Enum, &self)),
                        }
                    }
                }

                deserializer.deserialize_str(FieldVisitor)
            }
        }
    }
}
