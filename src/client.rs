use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use reqwest::{Client as HttpClient, RequestBuilder, Proxy};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use serde::de::{self, Deserializer};
use serde_derive::{Deserialize, Serialize};
use crate::tag::{Request, Response};

pub struct Client {
    client:   HttpClient,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    #[serde(deserialize_with = "from_str")]
    pub id:     u64,
    #[serde(rename = "device_name")]
    pub name:   String,
    #[serde(rename = "device_type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeviceWrapper {
    Device(Device),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dimensions {
    #[serde(rename = "customDimensions")]
    pub dimensions: Vec<Dimension>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Dimension {
    pub id:           u64,
    pub name:         String,
    pub display_name: String,
    pub r#type:       String,
    #[serde(default, rename = "is_bulk")]
    pub bulk:         bool,
    #[serde(default)]
    pub internal:     bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DimensionWrapper {
    #[serde(rename = "customDimension")]
    Dimension(Dimension),
    Error(String),
}

impl Client {
    pub fn new(email: &str, token: &str, endpoint: &str, proxy: Option<&str>) -> Result<Self, Box<Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("X-CH-Auth-Email",     email.parse()?);
        headers.insert("X-CH-Auth-API-Token", token.parse()?);

        let mut client = HttpClient::builder().default_headers(headers);

        if let Some(url) = proxy {
            client = client.proxy(Proxy::all(url)?);
            // client = client.danger_accept_invalid_certs(true);
        }

        Ok(Self {
            client:   client.build()?,
            endpoint: endpoint.to_owned(),
        })
    }

    pub fn get_device_by_name(&self, name: &str) -> Result<DeviceWrapper, Box<Error>> {
        let url = format!("{}/api/internal/device/{}", self.endpoint, name);
        Ok(self.get(&url).send()?.json()?)
    }

    pub fn get_custom_dimensions(&self) -> Result<Dimensions, Box<Error>> {
        let url = format!("{}/api/internal/customdimensions", self.endpoint);
        Ok(self.get(&url).send()?.json()?)
    }

    pub fn add_custom_dimension(&self, d: &Dimension) -> Result<DimensionWrapper, Box<Error>> {
        let url = format!("{}/api/internal/customdimension", self.endpoint);
        Ok(self.post(&url, d).send()?.json()?)
    }

    pub fn update_populators(&self, column: &str, r: &Request) -> Result<Response, Box<Error>> {
        let url = format!("{}/api/internal/batch/customdimensions/{}/populators", self.endpoint, column);
        Ok(self.post(&url, r).send()?.json()?)
    }

    fn get(&self, url: &str) -> RequestBuilder {
        self.client.get(url)
    }

    fn post<T: Serialize>(&self, url: &str, body: &T) -> RequestBuilder {
        self.client.post(url).json(body)
    }
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
