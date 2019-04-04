use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::*;
use log::{debug, error};
use rmp_serde::Serializer;
use serde::Serialize;
use super::Response;
use crate::{Client as ApiClient, Error};
use Error::App;
use RecvTimeoutError::*;

pub struct Client {
    sender: Sender<Response>,
    thread: JoinHandle<Result<(), Error>>,
}

impl Client {
    pub fn new(c: ApiClient) -> Self {
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

fn poll(rx: Receiver<Response>, c: ApiClient) -> Result<(), Error> {
    let mut vec = Vec::with_capacity(1024);
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
            match c.post_raw(c.endpoint(), &vec) {
                Ok(())          => debug!("submitted batch"),
                Err(App(e, _))  => error!("DNS API error {}", e),
                Err(e)          => error!("request error {}", e),
            }
            vec.clear();
        }
    }

    Ok(())
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Error::Other(err.to_string())
    }
}
