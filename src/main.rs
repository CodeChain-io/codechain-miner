extern crate hyper;

use hyper::{Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn;

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work(_req: Request<Body>) -> BoxFut {
    unimplemented!()
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
