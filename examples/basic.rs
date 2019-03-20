use std::env;
use std::error::Error;
use env_logger;
use rkapi::client::*;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let email    = env::var("EMAIL").expect("env var EMAIL");
    let token    = env::var("TOKEN").expect("env var TOKEN");
    let endpoint = env::var("ENDPOINT").unwrap_or_else(|_| {
        "https://api.our1.kentik.com".to_string()
    });

    let client = Client::new(&email, &token, &endpoint, None)?;

    let r = client.add_custom_dimension(&Dimension{
        name:         "c_will_test_00".to_owned(),
        display_name: "A test column".to_owned(),
        r#type:       "string".to_owned(),
        ..Default::default()
    })?;
    println!("{:#?}", r);

    let r = client.get_custom_dimensions()?;
    println!("{:#?}", r);

    Ok(())
}
