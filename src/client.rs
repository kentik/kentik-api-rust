use std::fmt::Display;
use std::str::FromStr;
use backoff::{ExponentialBackoff, Operation};
use log::debug;
use reqwest::{Client as HttpClient, StatusCode, Proxy};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use serde::de::{self, DeserializeOwned, Deserializer};
use serde::ser::Serializer;
use crate::Error;
use crate::tag::{Request, Response};

const MAX_RETRIES: u64 = 3;

pub struct Client {
    client:   HttpClient,
    endpoint: String,
}

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

impl Client {
    pub fn new(email: &str, token: &str, endpoint: &str, proxy: Option<&str>) -> Result<Self, Error> {
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

    pub fn get_device_by_name(&self, name: &str) -> Result<Device, Error> {
        let url = format!("{}/api/internal/device/{}", self.endpoint, name);

        #[derive(Serialize, Deserialize, Debug)]
        struct Wrapper {
            device: Device,
        }

        Ok(self.get::<Wrapper>(&url)?.device)
    }

    pub fn get_custom_dimensions(&self) -> Result<Dimensions, Error> {
        let url = format!("{}/api/internal/customdimensions", self.endpoint);
        self.get(&url)
    }

    pub fn add_custom_dimension(&self, d: &Dimension) -> Result<Dimension, Error> {
        let url = format!("{}/api/internal/customdimension", self.endpoint);

        #[derive(Serialize, Deserialize, Debug)]
        struct Wrapper {
            #[serde(rename = "customDimension")]
            dimension: Dimension,
        }

        Ok(self.post::<_, Wrapper>(&url, d)?.dimension)
    }

    pub fn update_populators(&self, column: &str, r: &Request) -> Result<Response, Error> {
        let url = format!("{}/api/internal/batch/customdimensions/{}/populators", self.endpoint, column);
        self.post(&url, r)
    }

    fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        retry(|n| send(self.client.get(url)).map_err(|err| {
            debug!("GET {} #{} failed: {}", url, n, err);
            err
        }))
    }

    fn post<T: Serialize, U: DeserializeOwned>(&self, url: &str, body: &T) -> Result<U, Error> {
        retry(|n| send(self.client.post(url).json(body)).map_err(|err| {
            debug!("POST {} #{} failed: {}", url, n, err);
            err
        }))
    }
}

fn retry<T>(mut op: impl FnMut(u64) -> Result<T, Error>) -> Result<T, Error> {
    let mut backoff = ExponentialBackoff::default();
    let mut attempt = 0;

    let mut task = || op(attempt).map_err(|err| {
        attempt += 1;

        if attempt >= MAX_RETRIES {
            return backoff::Error::Permanent(err);
        }

        match err {
            Error::Auth              => backoff::Error::Permanent(err),
            Error::App(_, 300...499) => backoff::Error::Permanent(err),
            Error::Status(300...499) => backoff::Error::Permanent(err),
            _                        => backoff::Error::Transient(err),
        }
    });

    Ok(task.retry(&mut backoff)?)
}

fn send<T: DeserializeOwned>(r: reqwest::RequestBuilder) -> Result<T, Error> {
    let mut r  = r.send()?;
    let status = r.status();

    let mut error = || {
        #[derive(Deserialize)]
        struct Wrapper {
            error: String,
        }

        match r.json::<Wrapper>() {
            Ok(w)  => Error::App(w.error, status.into()),
            Err(_) => Error::Status(status.into()),
        }
    };

    match status {
        _ if status.is_success() => Ok(r.json()?),
        StatusCode::UNAUTHORIZED => Err(Error::Auth),
        _                        => Err(error()),
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

fn to_str<T: Display, S: Serializer>(v: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&v.to_string())
}

impl From<backoff::Error<Error>> for Error {
    fn from(err: backoff::Error<Error>) -> Self {
        match err {
            backoff::Error::Permanent(e) => e,
            backoff::Error::Transient(e) => e,
        }
    }
}
