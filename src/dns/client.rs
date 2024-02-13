use std::mem::swap;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::*;
use log::{debug, error};
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::runtime::Runtime;
use super::Response;
use crate::{AsyncClient, Error};
use Error::{App, Empty};
use RecvTimeoutError::*;

pub struct Client {
    sender: Sender<Vec<Response>>,
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

    pub fn send(&self, rs: Vec<Response>, d: Duration) -> Result<(), Error> {
        Ok(self.sender.send_timeout(rs, d)?)
    }

    pub fn stop(self) -> Result<(), Error> {
        drop(self.sender);
        self.thread.join()?
    }
}

fn poll(rx: Receiver<Vec<Response>>, c: AsyncClient) -> Result<(), Error> {
    let mut buf = Vec::with_capacity(1024);
    let rt      = Runtime::new()?;
    let timeout = Duration::from_secs(1);
    let ticker  = tick(timeout);

    loop {
        let mut encode = |rs: Vec<Response>| -> Result<(), Error> {
            let mut s = Serializer::new(&mut buf).with_struct_map();
            rs.into_iter().map(|r: Response| -> Result<(), Error> {
                Ok(r.serialize(&mut s)?)
            }).collect::<Result<_, _>>()
        };

        match rx.recv_timeout(timeout) {
            Ok(rs)            => encode(rs)?,
            Err(Timeout)      => (),
            Err(Disconnected) => break,
        };

        let flush = ticker.try_recv().is_ok() && !buf.is_empty();
        let bytes = buf.len();

        if flush || bytes >= 1_000_000 {
            let mut vec = Vec::with_capacity(bytes);
            swap(&mut buf, &mut vec);

            let client = c.clone();
            rt.spawn(async move {
                match client.post("/dns", vec).await {
                    Ok(()) | Err(Empty) => debug!("submitted batch"),
                    Err(App(e, _))      => error!("DNS API error {}", e),
                    Err(e)              => error!("request error {}", e),
                }
            });
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
