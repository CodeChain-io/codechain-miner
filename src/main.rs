extern crate futures;
extern crate hyper;

use futures::future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work(req: Request<Body>) -> BoxFut {
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
    // FIXME: Get notification address from command line option
    const RECEIVE_PORT: u16 = 3333;
    let addr = ([127, 0, 0, 1], RECEIVE_PORT).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(get_work))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
