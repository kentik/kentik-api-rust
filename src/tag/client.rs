use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::*;
use log::{debug, error};
use super::{Request, Response};
use crate::{Client as ApiClient, Error};
use Error::*;

pub struct Client {
    sender: Sender<(String, Request)>,
    thread: JoinHandle<Result<(), Error>>,
}

impl Client {
    pub fn new(c: ApiClient) -> Self {
        let (tx, rx) = bounded(1024);
        let task     = || poll(rx, c);
        Self {
            sender: tx,
            thread: thread::spawn(task),
        }
    }

    pub fn send(&self, c: &str, r: Request, d: Duration) -> Result<(), Error> {
        Ok(self.sender.send_timeout((c.to_owned(), r), d)?)
    }

    pub fn stop(self) -> Result<(), Error> {
        drop(self.sender);
        self.thread.join()?
    }
}

fn poll(rx: Receiver<(String, Request)>, c: ApiClient) -> Result<(), Error> {
    while let Ok((column, request)) = rx.recv() {
        match c.update_populators(&column, &request) {
            Ok(Response{guid, ..}) => debug!("submitted: {}", guid),
            Err(App(e, _))         => error!("tag API error {}", e),
            Err(e)                 => error!("request error {}", e),
        }
    }

    Ok(())
}

impl<T> From<SendTimeoutError<T>> for Error {
    fn from(_: SendTimeoutError<T>) -> Self {
        Error::Timeout
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<Box<dyn std::any::Any + Send>> for Error {
    fn from(err: Box<dyn std::any::Any + Send>) -> Self {
        Error::Other(format!("{:#?}", err))
    }
}
