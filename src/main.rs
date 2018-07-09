extern crate ethereum_types;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;

mod worker;

use futures::future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;

use self::worker::{CuckooWorker, Worker};

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work(req: Request<Body>, algorithm: &str) -> BoxFut {
    let worker = match algorithm {
        "cuckoo" => CuckooWorker::new(),
        _ => unreachable!(),
    };
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            Box::new(req.into_body().concat2().map(|_| {
                // FIXME: Spawn worker with received work
                *response.status_mut() = StatusCode::OK;
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
