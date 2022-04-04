use std::fmt::{self, Display};
use reqwest;

pub mod client;
pub mod r#async;
pub mod core;
pub mod dns;
pub mod tag;

pub use client::Client;
pub use r#async::{Client as AsyncClient};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Error {
    Auth,
    App(String, u16),
    Status(u16),
    Empty,
    Timeout,
    Other(String),
}

impl Error {
    fn into_backoff(self) -> backoff::Error<Self> {
        match self {
            Error::Auth | Error::Empty => backoff::Error::Permanent(self),
            Error::App(_, 300..=499)   => backoff::Error::Permanent(self),
            Error::Status(300..=499)   => backoff::Error::Permanent(self),
            _                          => backoff::Error::Transient(self),
        }
    }
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

impl From<backoff::Error<Error>> for Error {
    fn from(err: backoff::Error<Error>) -> Self {
        match err {
            backoff::Error::Permanent(e) => e,
            backoff::Error::Transient(e) => e,
        }
    }
}
