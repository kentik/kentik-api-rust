use serde::{Serialize, Deserialize};

pub mod client;

pub use client::Client;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "Question")]
    pub question: Question,
    #[serde(rename = "Answers")]
    pub answers:  Vec<Answer>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Question {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Host", with = "serde_bytes")]
    pub host: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Answer {
    #[serde(rename = "Name")]
    pub name:  String,
    #[serde(rename = "CNAME")]
    pub cname: String,
    #[serde(rename = "IP", with = "serde_bytes")]
    pub ip:    Vec<u8>,
    #[serde(rename = "TTL")]
    pub ttl:   u32,
}
