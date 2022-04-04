use backoff::{ExponentialBackoff, Operation};
use log::debug;
use reqwest::{StatusCode, Proxy};
use reqwest::blocking::{Client as HttpClient, RequestBuilder, Response};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::Error;

pub struct Client {
    client:   HttpClient,
    endpoint: String,
    retries:  u64,
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
            retries:  3,
        })
    }

    pub fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        retry(|n| send(self.client.get(url)).map_err(|err| {
            debug!("GET {} #{} failed: {}", url, n, err);
            err
        }), self.retries)
    }

    pub fn post<T: Serialize, U: DeserializeOwned>(&self, url: &str, body: &T) -> Result<U, Error> {
        retry(|n| send(self.client.post(url).json(body)).map_err(|err| {
            debug!("POST {} #{} failed: {}", url, n, err);
            err
        }), self.retries)
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

fn retry<T>(mut op: impl FnMut(u64) -> Result<T, Error>, retries: u64) -> Result<T, Error> {
    let mut backoff = ExponentialBackoff::default();
    let mut attempt = 0;

    let mut task = || op(attempt).map_err(|err| {
        attempt += 1;

        if attempt >= retries {
            return backoff::Error::Permanent(err);
        }

        err.into_backoff()
    });

    Ok(task.retry(&mut backoff)?)
}

fn send<T: DeserializeOwned>(r: RequestBuilder) -> Result<T, Error> {
    let response = r.send()?;
    let status   = response.status();

    let error = |response: Response| {
        #[derive(Deserialize)]
        struct Wrapper {
            error: String,
        }

        match response.json::<Wrapper>() {
            Ok(w)  => Error::App(w.error, status.into()),
            Err(_) => Error::Status(status.into()),
        }
    };

    match status {
        _ if status.is_success() => Ok(response.json()?),
        StatusCode::UNAUTHORIZED => Err(Error::Auth),
        _                        => Err(error(response)),
    }
}
