use bytes::Bytes;
use futures::{Future, future::{err, Either::*}};
use reqwest::{self, Proxy, StatusCode};
use reqwest::r#async::{Client as HttpClient, RequestBuilder, Response};
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use crate::Error;
use super::retry::retry;

#[derive(Clone)]
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
        }

        Ok(Self {
            client:   client.build()?,
            endpoint: endpoint.to_owned(),
            retries:  3,
        })
    }

    pub fn get<T: DeserializeOwned>(&self, url: &str) -> impl Future<Item = T, Error = Error> {
        let url    = format!("{}{}", self.endpoint, url);
        let client = self.client.clone();
        retry(move || send(client.get(&url)), self.retries)
    }

    pub fn post<T: DeserializeOwned>(&self, url: &str, body: Vec<u8>) -> impl Future<Item = T, Error = Error> {
        let url    = format!("{}{}", self.endpoint, url);
        let client = self.client.clone();
        let body   = Bytes::from(body);
        retry(move || send(client.post(&url).body(body.clone())), self.retries)
    }
}

fn send<T: DeserializeOwned>(r: RequestBuilder) -> impl Future<Item = T, Error = Error> {
    const OK:           StatusCode = StatusCode::OK;
    const UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED;
    r.send().from_err::<Error>().and_then(|mut r| {
        let status = r.status();

        let error  = |mut r: Response| {
            #[derive(Deserialize)]
            struct Wrapper {
                error: String,
            }

            r.json::<Wrapper>().then(move |r| err(match r {
                Ok(w)  => Error::App(w.error, status.into()),
                Err(_) => Error::Status(status.into()),
            }))
        };

        let result = r.headers().get(CONTENT_TYPE).map(|v| {
            (v.as_bytes(), status.is_success())
        }).ok_or(status);

        match result {
            Ok((b"application/json", true)) => A(A(r.json().from_err::<Error>())),
            Ok((_,                   true)) => B(err(Error::Empty)),
            Err(OK)                         => B(err(Error::Empty)),
            Err(UNAUTHORIZED)               => B(err(Error::Auth)),
            _                               => A(B(error(r))),
        }
    })
}
