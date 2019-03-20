#![feature(test)]

use std::fmt::{self, Display};
use reqwest;

pub mod client;
pub mod tag;

pub use client::Client;

#[derive(Debug)]
pub enum Error {
    Auth,
    API(String, u16),
    Status(u16),
    Timeout,
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {
}

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        match err.is_timeout() {
            true  => Error::Timeout,
            false => Error::Other(err.to_string())
        }
    }
}
