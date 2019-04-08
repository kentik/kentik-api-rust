mod server;

use std::time::Duration;
use kentik_api::{Client, Error};
use kentik_api::core::*;
use server::Server;

#[test]
fn auth_required() {
    let server = server::start("127.0.0.1:0", None, None);

    let (mut email, token) = server.auth();
    email += "-invalid";
    let endpoint = server.url("");

    let client = Client::new(&email, &token, &endpoint, None).unwrap();
    let result = client.get_device_by_name("invalid");
    assert_eq!(Err(Error::Auth), result);
}

#[test]
fn status_error() {
    let (client, _server) = pair();
    let result = client.get_device_by_name("404");
    assert_eq!(Err(Error::Status(404)), result);
}

#[test]
fn app_error() {
    let (client, _server) = pair();
    let result  = client.get_device_by_name("invalid");
    let message = "invalid device name".to_string();
    assert_eq!(Err(Error::App(message, 404)), result);
}

#[test]
fn retry_request() {
    let (client, server) = pair();
    let timeout = Duration::from_secs(1);

    let result = client.get_device_by_name("503");

    for _ in 0..3 {
        let path = server.request(timeout).unwrap().path;
        assert_eq!("/api/internal/device/503", path)
    }

    assert_eq!(Err(Error::Status(503)), result);
}

#[test]
fn no_retry_on_4xx() {
    let (client, server) = pair();
    let timeout = Duration::from_millis(100);

    let result = client.get_device_by_name("403");

    assert!(server.request(timeout).is_ok());
    assert!(server.request(timeout).is_err());

    assert_eq!(Err(Error::Status(403)), result);
}

#[test]
fn get_device_by_name() {
    let (client, _server) = pair();

    let device = Device {
        id:   1,
        name: "test".to_owned(),
        kind: "router".to_owned(),
    };

    let result = client.get_device_by_name("test");
    assert_eq!(Ok(device), result);
}

fn pair() -> (Client, Server) {
    let server = server::start("127.0.0.1:0", None, None);
    let (email, token) = server.auth();
    let endpoint = server.url("");
    let client = Client::new(&email, &token, &endpoint, None).unwrap();
    (client, server)
}
