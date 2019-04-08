use std::time::Duration;
use crossbeam_channel::RecvTimeoutError::*;
use env_logger;
use log::info;

#[path="../tests/server/mod.rs"]
mod server;

fn main() {
    env_logger::init();

    let server  = server::start("127.0.0.1:8999", Some("test@example.com"), Some("token"));
    let timeout = Duration::from_secs(60);

    info!("server address {}", server.url(""));

    loop {
        match server.request(timeout) {
            Ok(request)       => info!("{:#?}", request),
            Err(Timeout)      => (),
            Err(Disconnected) => break,
        }
    }
}
