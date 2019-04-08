use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::*;
use futures::future::{ok, Future};
use log::{debug, error};
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::runtime::Runtime;
use super::Response;
use crate::{AsyncClient, Error};
use Error::{App, Empty};
use RecvTimeoutError::*;

pub struct Client {
    sender: Sender<Response>,
    thread: JoinHandle<Result<(), Error>>,
}

impl Client {
    pub fn new(c: AsyncClient) -> Self {
        let (tx, rx) = bounded(100_000);
        let task     = || poll(rx, c);
        Self {
            sender: tx,
            thread: thread::spawn(task),
        }
    }

    pub fn send(&self, r: Response, d: Duration) -> Result<(), Error> {
        Ok(self.sender.send_timeout(r, d)?)
    }

    pub fn stop(self) -> Result<(), Error> {
        drop(self.sender);
        self.thread.join()?
    }
}

fn poll(rx: Receiver<Response>, c: AsyncClient) -> Result<(), Error> {
    let mut vec = Vec::with_capacity(1024);
    let mut rt  = Runtime::new()?;
    let timeout = Duration::from_secs(1);
    let ticker  = tick(timeout);

    loop {
        let mut s = Serializer::new_named(&mut vec);

        match rx.recv_timeout(timeout) {
            Ok(r)             => r.serialize(&mut s)?,
            Err(Timeout)      => (),
            Err(Disconnected) => break,
        };

        let flush = ticker.try_recv().is_ok() && !vec.is_empty();

        if flush || vec.len() >= 1_000_000 {
            rt.spawn(c.post("/dns", vec).then(|r| {
                match r {
                    Ok(()) | Err(Empty) => debug!("submitted batch"),
                    Err(App(e, _))      => error!("DNS API error {}", e),
                    Err(e)              => error!("request error {}", e),
                }
                ok(())
            }));
            vec = Vec::with_capacity(1024);
        }
    }

    Ok(())
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Other(err.to_string())
    }
}
