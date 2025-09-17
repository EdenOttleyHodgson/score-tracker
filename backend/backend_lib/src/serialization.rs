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
        deserializer.deserialize_any(RoomCodeVisitor)
    }
}
struct RoomCodeVisitor;
impl<'de> Visitor<'de> for RoomCodeVisitor {
    type Value = RoomCode;
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        println!("{v}");
        let err = E::invalid_value(
            serde::de::Unexpected::Str(v),
            &"An alphanumeric string of length 8",
        );
        if v.chars().all(char::is_alphanumeric) {
            let array: [char; 8] = v.chars().collect_array().ok_or(err)?;
            println!("3{v}");
            Ok(RoomCode::from(array))
        } else {
            println!("4{v}");
            Err(err)
        }
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        panic!("granggl")
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("An alpha numeric string of length 8")
    }
}
#[cfg(test)]
mod tests {
    use crate::state::room::RoomCode;
    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct Dummy {
        dummy: RoomCode,
    }

    #[test]
    fn json_serialize() {
        let expected = Dummy {
            dummy: RoomCode::from(['A', 'A', 'A', 'A', 'A', 'A', 'A', 'A']),
        };
        let input = "{\"dummy\": \"AAAAAAAA\"}";
        let output: Dummy = serde_json::from_str(input).unwrap();
        assert_eq!(expected, output)
    }
}
