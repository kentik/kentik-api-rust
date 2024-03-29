use std::env;
use std::error::Error;
use env_logger;
use kentik_api::client::*;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let email    = env::var("EMAIL").expect("env var EMAIL");
    let token    = env::var("TOKEN").expect("env var TOKEN");
    let endpoint = env::var("ENDPOINT").unwrap_or_else(|_| {
        "https://api.our1.kentik.com".to_string()
    });

    let client = Client::new(&email, &token, &endpoint, None)?;
    let device = client.get_device_by_name("istio_test")?;
    println!("{:#?}", device);

    Ok(())
}
