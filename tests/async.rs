mod server;

use std::time::Duration;
use serde::{Serialize, Deserialize};
use tokio::runtime::Builder;
use kentik_api::{AsyncClient, Error};
use kentik_api::core::Device;
use server::Server;

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
struct Wrapper {
    device: Device,
}

#[test]
fn async_status_error() {
    let (client, _server) = pair();

    let rt = Builder::new_current_thread().enable_all().enable_all().build().unwrap();
    let result = rt.block_on(client.get::<Wrapper>("/api/internal/device/404"));

    assert_eq!(Err(Error::Status(404)), result);
}

#[test]
fn async_app_error() {
    let (client, _server) = pair();

    let rt  = Builder::new_current_thread().enable_all().build().unwrap();
    let result  = rt.block_on(client.get::<Wrapper>("/api/internal/device/invalid"));
    let message = "invalid device name".to_string();

    assert_eq!(Err(Error::App(message, 404)), result);
}

#[test]
fn async_retry_request() {
    let (client, server) = pair();
    let timeout = Duration::from_secs(1);

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let result = rt.block_on(client.get::<Wrapper>("/api/internal/device/503"));

    assert_eq!("/api/internal/device/503", server.request(timeout).unwrap().path);
    assert_eq!("/api/internal/device/503", server.request(timeout).unwrap().path);
    assert_eq!("/api/internal/device/503", server.request(timeout).unwrap().path);

    assert_eq!(Err(Error::Status(503)), result);
}

#[test]
fn async_no_retry_on_4xx() {
    let (client, server) = pair();

    let timeout = Duration::from_millis(100);
    let rt  = Builder::new_current_thread().enable_all().build().unwrap();
    let result  = rt.block_on(client.get::<Wrapper>("/api/internal/device/403"));

    assert!(server.request(timeout).is_ok());
    assert!(server.request(timeout).is_err());

    assert_eq!(Err(Error::Status(403)), result);
}

#[test]
fn async_get_device_by_name() {
    let (client, _server) = pair();

    let device = Device {
        id:   1,
        name: "test".to_owned(),
        kind: "router".to_owned(),
    };

    let path = format!("/api/internal/device/{}", device.name);

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let result = rt.block_on(async move {
        Result::<_, Error>::Ok(client.get::<Wrapper>(&path).await?.device)
    });

    assert_eq!(Ok(device), result);
}

fn pair() -> (AsyncClient, Server) {
    let server = server::start("127.0.0.1:0", None, None);
    let (email, token) = server.auth();
    let endpoint = server.url("");
    let client = AsyncClient::new(&email, &token, &endpoint, None).unwrap();
    (client, server)
}
