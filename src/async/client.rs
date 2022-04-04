use bytes::Bytes;
use reqwest::{self, Proxy, StatusCode};
use reqwest::{Client as HttpClient, RequestBuilder, Response};
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use crate::Error;
use super::retry::retry;

#[derive(Clone)]
pub struct Client {
    client:   HttpClient,
    endpoint: String,
    retries:  usize,
}

impl Client {
    pub fn new(email: &str, token: &str, endpoint: &str, proxy: Option<&str>) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert("X-CH-Auth-Email",     email.parse()?);
        headers.insert("X-CH-Auth-API-Token", token.parse()?);

        let mut client = HttpClient::builder().default_headers(headers);

        if let Some(url) = proxy {
            client = client.proxy(Proxy::all(url)?);
        }

        Ok(Self {
            client:   client.build()?,
            endpoint: endpoint.to_owned(),
            retries:  3,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let url    = format!("{}{}", self.endpoint, url);
        let client = self.client.clone();
        match retry(move || send(client.get(&url)), self.retries).await {
            Ok((value, _)) => Ok(value),
            Err((err, _))  => Err(err),
        }
    }

    pub async fn post<T: DeserializeOwned>(&self, url: &str, body: Vec<u8>) -> Result<T, Error> {
        let url    = format!("{}{}", self.endpoint, url);
        let client = self.client.clone();
        let body   = Bytes::from(body);
        match retry(move || send(client.post(&url).body(body.clone())), self.retries).await {
            Ok((value, _)) => Ok(value),
            Err((err, _))  => Err(err),
        }
    }
}

async fn send<T: DeserializeOwned>(r: RequestBuilder) -> Result<T, Error> {
    const OK:           StatusCode = StatusCode::OK;
    const UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED;

    let response = r.send().await?;
    let status   = response.status();

    let error = |response: Response| async {
        #[derive(Deserialize)]
        struct Wrapper {
            error: String,
        }

        match response.json::<Wrapper>().await {
            Ok(w)  => Error::App(w.error, status.into()),
            Err(_) => Error::Status(status.into()),
        }
    };

    let result = response.headers().get(CONTENT_TYPE).map(|v| {
        (v.as_bytes(), status.is_success())
    }).ok_or(status);

    match result {
        Ok((b"application/json", true)) => Ok(response.json().await?),
        Ok((_,                   true)) => Err(Error::Empty),
        Err(OK)                         => Err(Error::Empty),
        Err(UNAUTHORIZED)               => Err(Error::Auth),
        _                               => Err(error(response).await),
    }
}
