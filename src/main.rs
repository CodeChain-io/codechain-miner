extern crate byteorder;
extern crate ethereum_types;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod message;
mod worker;

use std::str::FromStr;

use ethereum_types::{clean_0x, H256, U256};
use futures::future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;

use self::message::Job;
use self::worker::{CuckooWorker, spawn_worker};

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work(req: Request<Body>, algorithm: &str) -> BoxFut {
    let worker = match algorithm {
        "cuckoo" => CuckooWorker::new(),
        _ => unreachable!(),
    };
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            Box::new(req.into_body().concat2().map(|chunk| {
                match serde_json::from_slice::<Job>(&chunk.into_bytes()) {
                    Ok(rpc) => {
                        // FIXME: don't unwrap while parsing incoming job
                        let hash = H256::from_str(clean_0x(&rpc.result.0)).unwrap();
                        let target = U256::from_str(clean_0x(&rpc.result.1)).unwrap();
                        spawn_worker(hash, target, Box::new(worker));
                        *response.status_mut() = StatusCode::OK;
                    }
                    Err(_) => {
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                    }
                }
                response
            }))
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(response))
        }
    }
}

fn main() {
    env_logger::init();
    // FIXME: Get notification address from command line option
    const RECEIVE_PORT: u16 = 3333;
    let addr = ([127, 0, 0, 1], RECEIVE_PORT).into();

    // FIXME: Get algorithm type from command line option
    let algorithm = String::from("cuckoo");

    let server = Server::bind(&addr)
        .serve(move || {
            let a = algorithm.clone();
            service_fn(move |req| get_work(req, &a))
        })
        .map_err(|e| error!("server error: {}", e));

    hyper::rt::run(server);
}
