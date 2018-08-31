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

use std::sync::Arc;
use std::thread::spawn;

use ethereum_types::{clean_0x, H256};
use futures::future;
use hyper::header::HeaderValue;
use hyper::rt::{run, Future, Stream};
use hyper::service::service_fn;
use hyper::{self, Body, Client, Method, Request, Response, Server, StatusCode};
use rustc_hex::ToHex;
use serde_json;

use super::super::worker::{work, Worker};
use super::RpcRunner;

#[derive(Deserialize)]
pub struct Job {
    /// (hash, target).
    pub result: (String, String),
}

#[derive(Clone)]
pub struct Config {
    pub listen_port: u16,
    pub submitting_port: u16,
}

pub struct Runner {
    listen_port: u16,
    submitting_port: u16,
}

impl Runner {
    pub fn new(config: &Config) -> Self {
        Self {
            listen_port: config.listen_port,
            submitting_port: config.submitting_port,
        }
    }
}

impl RpcRunner for Runner {
    fn run(&self, recruiter: Arc<Fn() -> Box<Worker> + Send + Sync>, jobs: usize) {
        let submit_port = self.submitting_port;
        let addr = ([127, 0, 0, 1], self.listen_port).into();
        let server = Server::bind(&addr)
            .serve(move || {
                let recruiter = Arc::clone(&recruiter);
                service_fn(move |req| {
                    let worker = recruiter();
                    get_work(worker, jobs, req, submit_port)
                })
            })
            .map_err(|e| error!("server error: {}", e));
        info!("Server started, listening on {:?}", addr);
        info!("It will submit to 127.0.0.1:{}", submit_port);
        info!("The maximum number of parallel jobs is {}", jobs);

        hyper::rt::run(server);
    }
}

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn get_work(worker: Box<Worker>, jobs: usize, req: Request<Body>, submit_port: u16) -> BoxFut {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            Box::new(req.into_body().concat2().map(move |chunk| {
                match serde_json::from_slice::<Job>(&chunk.into_bytes()) {
                    Ok(rpc) => {
                        // FIXME: don't unwrap while parsing incoming job
                        let hash = clean_0x(&rpc.result.0).parse().unwrap();
                        let target = clean_0x(&rpc.result.1).parse().unwrap();
                        spawn(move || {
                            if let Some(solution) = work(&hash, &target, worker, jobs) {
                                submit(hash, solution, submit_port);
                            }
                        });
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

pub fn submit(hash: H256, solution: Vec<Vec<u8>>, port: u16) {
    let seal: Vec<_> = solution.iter().map(|bytes| format!("0x{}", bytes.to_hex())).collect();

    let json = json!({
        "jsonrpc": "2.0",
        "method": "miner_submitWork",
        "params": [
            format!("0x{:x}", hash),
            seal,
        ],
        "id": null
    });
    let mut req = Request::new(Body::from(json.to_string()));
    *req.method_mut() = Method::POST;
    *req.uri_mut() = format!("http://127.0.0.1:{}", port).parse().unwrap();
    req.headers_mut().insert("content-type", HeaderValue::from_str("application/json").unwrap());

    info!("Job finished with hash {}, seal {:?}", hash, seal);
    run(Client::new().request(req).map(|_| {}).map_err(|err| {
        eprintln!("Error {}", err);
    }));
}
