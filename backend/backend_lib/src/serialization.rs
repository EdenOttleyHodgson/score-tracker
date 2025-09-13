use itertools::Itertools;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize, de::Visitor};

use crate::state::room::RoomCode;

impl JsonSchema for RoomCode {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RoomCode".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "type": "string",
            "min_length": "8",
            "max_length": "8",
        })
    }
}

impl Serialize for RoomCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&String::from(self))
    }
}

impl<'de> Deserialize<'de> for RoomCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct("RoomCode", RoomCodeVisitor)
    }
}
struct RoomCodeVisitor;
impl Visitor<'_> for RoomCodeVisitor {
    type Value = RoomCode;
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let err = E::invalid_value(
            serde::de::Unexpected::Str(v),
            &"An alphanumeric string of length 8",
        );
        let mut chars = v.chars();
        if chars.all(char::is_alphanumeric) {
            let array: [char; 8] = chars.collect_array().ok_or(err)?;
            Ok(RoomCode::from(array))
        } else {
            Err(err)
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("An alpha numeric string of length 8")
    }
}
