use std::fmt::Display;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;

pub mod client;

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Device {
    #[serde(deserialize_with = "from_str", serialize_with = "to_str")]
    pub id:   u64,
    #[serde(rename = "device_name")]
    pub name: String,
    #[serde(rename = "device_type")]
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dimensions {
    #[serde(rename = "customDimensions")]
    pub dimensions: Vec<Dimension>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Dimension {
    pub id:           u64,
    pub name:         String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub kind:         String,
    #[serde(default, rename = "is_bulk")]
    pub bulk:         bool,
    #[serde(default)]
    pub internal:     bool,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn to_str<T: Display, S: Serializer>(v: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&v.to_string())
}
