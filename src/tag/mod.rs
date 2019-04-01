use serde_derive::{Deserialize, Serialize};

pub mod client;
pub mod change;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub replace_all: bool,
    pub complete:    bool,
    pub ttl_minutes: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub upserts:     Vec<Upsert>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deletes:     Vec<Delete>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Upsert {
    Small(Small),
    Large(Large),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Small {
    pub value:    String,
    pub criteria: (Rule,)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Large {
    pub value:    String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<Rules>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Rules {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction:       Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub port:            Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub protocol:        Vec<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub asn:             Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vlans:           Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lasthop_as_name: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nexthop_asn:     Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nexthop_as_name: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bgp_aspath:      Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bgp_community:   Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_flags:       Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub addr:            Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mac:             Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub country:         Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub site:            Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub device_type:     Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interface_name:  Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub device_name:     Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_hop:        Vec<String>,
}
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Rule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction:       Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port:            Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol:        Option<[u64; 1]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn:             Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlans:           Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lasthop_as_name: Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nexthop_asn:     Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nexthop_as_name: Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_aspath:      Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_community:   Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_flags:       Option<(u16,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addr:            Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac:             Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country:         Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site:            Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type:     Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_name:  Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name:     Option<(String,)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_hop:        Option<(String,)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Delete {
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    message: String,
    guid:    String,
}
