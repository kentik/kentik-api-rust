#![allow(dead_code, non_snake_case)]

use std::marker::PhantomData;
use std::mem::replace;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::{Instant, Duration};
use actix_rt::System;
use actix_service::{NewService, Service, Transform};
use actix_web::{*, dev, web::*};
use actix_web::http::{Method, HeaderValue};
use actix_web::dev::{Payload, ServiceRequest, ServiceResponse};
use actix_web::error::PayloadError;
use bytes_old::Bytes;
use crossbeam_channel::*;
use futures_old::{Async, Future, Poll, Stream};
use futures_old::future::{ok, FutureResult};
use rand::prelude::*;
use serde::{Serialize, Deserialize};
use kentik_api::core::{Device, Dimension};

pub struct Server {
    address:  SocketAddr,
    server:   dev::Server,
    email:    String,
    token:    String,
    requests: Receiver<Request>,
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path:   String,
    pub body:   Receiver<Bytes>,
    pub ts:     Instant,
}

pub fn start<T: ToSocketAddrs + Send + 'static>(addrs: T, email: Option<&str>, token: Option<&str>) -> Server {
    let (tx0, rx0) = bounded(1);
    let (tx1, rx1) = unbounded();

    let email = email.map(str::to_owned).unwrap_or_else(random);
    let token = token.map(str::to_owned).unwrap_or_else(random);

    let auth  = Auth {
        email: email.clone(),
        token: token.clone(),
    };

    thread::spawn(move || {
        let system = System::new("test-server");
        let server = HttpServer::new(move || {
            App::new()
                .chain(Inspector::new(tx1.clone()))
                .wrap(auth.clone())
                .service(get_device)
                .service(resource("/api/internal/customdimension").route(post().to(add_custom_dimension)))
                .service(resource("/dns").route(post().to_async(dns_batch)))
        }).bind(addrs).unwrap();
        let address = server.addrs()[0];
        let server  = server.start();
        tx0.send((address, server)).unwrap();
        system.run().unwrap();
    });

    let (address, server) = rx0.recv().unwrap();

    Server {
        address:  address,
        server:   server,
        email:    email,
        token:    token,
        requests: rx1,
    }
}

impl Server {
    pub fn auth(&self) -> (String, String) {
        (self.email.clone(), self.token.clone())
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    pub fn request(&self, timeout: Duration) -> Result<Request, RecvTimeoutError> {
        self.requests.recv_timeout(timeout)
    }

    pub fn stop(&self) {
        self.server.stop(false);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Request {
    pub fn body(&self) -> Vec<u8> {
        let mut body = Vec::new();
        for bytes in &self.body {
            body.extend_from_slice(&bytes);
        }
        body
    }
}

#[derive(Clone)]
struct Auth {
    email: String,
    token: String,
}

impl<S, P, B> Transform<S> for Auth where
    S: Service<Request = ServiceRequest<P>, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Request   = ServiceRequest<P>;
    type Response  = ServiceResponse<B>;
    type Error     = S::Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future    = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            email:   self.email.parse().unwrap(),
            token:   self.token.parse().unwrap(),
            service: service,
        })
    }
}

struct AuthMiddleware<S> {
    email:   HeaderValue,
    token:   HeaderValue,
    service: S,
}

impl<S, P, B> Service for AuthMiddleware<S> where
    S: Service<Request = ServiceRequest<P>, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Request  = ServiceRequest<P>;
    type Response = ServiceResponse<B>;
    type Error    = Error;
    type Future   = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest<P>) -> Self::Future {
        if Some(&self.email) != req.headers().get("X-CH-Auth-Email") {
            return Box::new(ok(req.error_response(HttpResponse::Unauthorized().finish())));
        }

        if Some(&self.token) != req.headers().get("X-CH-Auth-API-Token") {
            return Box::new(ok(req.error_response(HttpResponse::Unauthorized().finish())));
        }

        Box::new(self.service.call(req))
    }
}

pub struct Inspector<P> {
    requests: Sender<Request>,
    marker:   PhantomData<P>,
}

impl<P> Inspector<P> where P: Stream<Item = Bytes, Error = PayloadError> {
    pub fn new(tx: Sender<Request>) -> Self {
        Self {
            requests: tx,
            marker:   PhantomData,
        }
    }
}

impl<P> NewService for Inspector<P> where P: Stream<Item = Bytes, Error = PayloadError> {
    type Request   = ServiceRequest<P>;
    type Response  = ServiceRequest<Body<Payload<P>>>;
    type Error     = Error;
    type InitError = ();
    type Service   = Inspector<P>;
    type Future    = FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self, _: &()) -> Self::Future {
        ok(Inspector{
            requests: self.requests.clone(),
            marker:   PhantomData,
        })
    }
}

impl<P> Service for Inspector<P> where P: Stream<Item = Bytes, Error = PayloadError> {
    type Request  = ServiceRequest<P>;
    type Response = ServiceRequest<Body<Payload<P>>>;
    type Error    = Error;
    type Future   = FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: ServiceRequest<P>) -> Self::Future {
        let (req, payload) = req.into_parts();
        let (tx, rx) = bounded(1);
        self.requests.send(Request {
            method: req.method().clone(),
            path:   req.path().to_string(),
            body:   rx,
            ts:     Instant::now(),
        }).unwrap();
        let payload = Body::new(payload, tx);
        ok(ServiceRequest::from_parts(req, Payload::Stream(payload)))
    }
}

pub struct Body<S> {
    stream: S,
    chunks: Sender<Bytes>,
}

impl<S> Body<S> where S: Stream<Item = Bytes, Error = PayloadError> {
    pub fn new(stream: S, tx: Sender<Bytes>) -> Self {
        Self {
            stream: stream,
            chunks: tx,
        }
    }
}

impl<S> Stream for Body<S> where S: Stream<Item = Bytes, Error = PayloadError> {
    type Item  = Bytes;
    type Error = PayloadError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let poll = self.stream.poll()?;
        if let Async::Ready(ready) = &poll {
            match ready {
                Some(bytes) => { self.chunks.try_send(bytes.clone()).ok(); },
                None        => { replace(&mut self.chunks, bounded(1).0);  },
            };
        }
        Ok(poll)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum DeviceWrapper {
    Device{device: Device},
    Error{error: String},
}

#[get("/api/internal/device/{name}")]
fn get_device(name: Path<String>) -> impl Responder {
    match name.as_str() {
        "403" => return HttpResponse::Forbidden().finish(),
        "404" => return HttpResponse::NotFound().finish(),
        "503" => return HttpResponse::ServiceUnavailable().finish(),
        _     => (),
    };

    if name.as_str() == "invalid" {
        return HttpResponse::NotFound().json(DeviceWrapper::Error{
            error: "invalid device name".to_string(),
        });
    }

    HttpResponse::Ok().json(DeviceWrapper::Device {
        device: Device {
            id:   1,
            name: name.to_string(),
            kind: "router".to_string(),
        },
    })
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum DimensionWrapper {
    Dimension{customDimension: Dimension},
    Error{error: String},
}

fn add_custom_dimension(json: Json<Dimension>) -> Json<DimensionWrapper> {
    Json(DimensionWrapper::Dimension{
        customDimension: json.into_inner(),
    })
}

fn dns_batch(p: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    p.concat2().from_err().and_then(|_body| {
        Ok(HttpResponse::Ok().finish())
    })
}

fn random() -> String {
    let mut rng  = rand::thread_rng();
    let mut data = [0u8; 8];
    rng.fill(&mut data);
    base64::encode(&data)
}
