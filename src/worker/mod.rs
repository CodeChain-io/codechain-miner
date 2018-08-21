// Copyright 2018 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod work;

use std::str::FromStr;

use ethereum_types::{clean_0x, H256, U256};
use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use serde_json;

use self::work::spawn_worker;

use super::message::Job;

pub fn run<C: 'static + Config>(config: C) {
    let listen_port = config.listening_port();
    let submit_port = config.submitting_port();
    let addr = ([127, 0, 0, 1], listen_port).into();

    let server = Server::bind(&addr)
        .serve(move || service_fn(move |req| get_work(config, req)))
        .map_err(|e| error!("server error: {}", e));
    info!("Server started, listening on {:?}", addr);
    info!("It will submit to 127.0.0.1:{}", submit_port);

    hyper::rt::run(server);
}


type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work<C: Config>(config: C, req: Request<Body>) -> BoxFut {
    let worker = config.worker();
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let submit_port = config.submitting_port();
            Box::new(req.into_body().concat2().map(move |chunk| {
                match serde_json::from_slice::<Job>(&chunk.into_bytes()) {
                    Ok(rpc) => {
                        // FIXME: don't unwrap while parsing incoming job
                        let hash = H256::from_str(clean_0x(&rpc.result.0)).unwrap();
                        let target = U256::from_str(clean_0x(&rpc.result.1)).unwrap();
                        spawn_worker(hash, target, worker, submit_port);
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

pub trait Config: Copy + Send {
    fn listening_port(&self) -> u16;
    fn submitting_port(&self) -> u16;
    fn worker(&self) -> Box<Worker>;
}

pub trait Worker: Send {
    fn init(&mut self, message: &[u8], nonce: u64, target: &U256);
    fn proceed(&mut self) -> Option<Vec<Vec<u8>>>;
    fn is_finished(&self) -> bool;
}
