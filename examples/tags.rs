use std::env;
use std::error::Error;
use std::time::Duration;
use env_logger;
use kentik_api::tag::*;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let email    = env::var("EMAIL").expect("env var EMAIL");
    let token    = env::var("TOKEN").expect("env var TOKEN");
    let endpoint = env::var("ENDPOINT").unwrap_or_else(|_| {
        "https://api.our1.kentik.com".to_string()
    });

    let client = kentik_api::Client::new(&email, &token, &endpoint, None)?;
    let client = Client::new(client);

    let users = vec![
        ("alice", "10.0.0.16"),
        ("bob",   "10.0.0.32"),
        ("eve",   "10.0.0.48"),
    ];

    let upserts = users.iter().map(|(name, ip)| {
        let name = name.to_string();
        let addr = Some((ip.to_string(),));
        Upsert::Small(Small{
            value:    name,
            criteria: (Rule{addr, ..Default::default()},)
        })
    }).collect::<Vec<_>>();

    client.send("c_will_test_00", Request {
        replace_all: false,
        complete:    true,
        ttl_minutes: 0,
        upserts:     upserts,
        deletes:     vec![],
    }, Duration::from_secs(1))?;

    client.stop()?;

    Ok(())
}
