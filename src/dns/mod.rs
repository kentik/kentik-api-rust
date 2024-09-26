use std::net::IpAddr;
use smallvec::SmallVec;
use serde::{Serialize, Serializer};

pub use client::Client;

#[derive(Clone, Eq, PartialEq, Serialize, Debug)]
pub struct Response {
    #[serde(rename = "Question")]
    pub question: Question,
    #[serde(rename = "Answers")]
    pub answers:  SmallVec<[Answer; 1]>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Debug)]
pub struct Question {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Host")]
    pub host: Address,
    #[serde(rename = "Port")]
    pub port: u16,
}

#[derive(Clone, Eq, PartialEq, Serialize, Debug)]
pub struct Answer {
    #[serde(rename = "Name")]
    pub name:  String,
    #[serde(rename = "CNAME")]
    pub cname: String,
    #[serde(rename = "IP")]
    pub ip:    Address,
    #[serde(rename = "TTL")]
    pub ttl:   u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Address(pub IpAddr);

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            IpAddr::V4(ip) => serializer.serialize_bytes(&ip.octets()[..]),
            IpAddr::V6(ip) => serializer.serialize_bytes(&ip.octets()[..]),
        }
    }
}

pub mod client;

