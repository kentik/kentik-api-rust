mod server;

use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use serde::Deserialize;
use rmp_serde::decode::Deserializer;
use kentik_api::dns::*;
use server::Server;

#[test]
fn send_dns_batch() {
    let (client, server) = pair();
    let start = Instant::now();

    let records = (0..=9).map(|n| {
        let name = format!("foo{}.com", n);
        let host = format!("10.0.0.{}", n).parse::<Ipv4Addr>().unwrap();
        let ip   = format!("10.1.0.{}", n).parse::<Ipv4Addr>().unwrap();

        Response {
            question: Question {
                name: name,
                host: host.octets().to_vec(),
            },
            answers: vec![Answer{
                name:  String::new(),
                cname: String::new(),
                ip:    ip.octets().to_vec(),
                ttl:   n,
            }],
        }
    }).collect::<Vec<_>>();

    client.send(records.clone(), Duration::from_millis(1)).unwrap();

    let interval = Duration::from_secs(1);
    let request = server.request(interval * 2).unwrap();
    let body = request.body();
    let mut de = Deserializer::from_slice(&body);

    for record in records {
        assert_eq!(record, Response::deserialize(&mut de).unwrap());
    }

    assert!(request.ts.duration_since(start) >= interval);

    assert!(server.request(interval * 2).is_err());
}

fn pair() -> (Client, Server) {
    let server = server::start("127.0.0.1:0", None, None);
    let (email, token) = server.auth();
    let endpoint = server.url("");
    let client = kentik_api::AsyncClient::new(&email, &token, &endpoint, None).unwrap();
    let client = Client::new(client);
    (client, server)
}
