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

extern crate byteorder;
extern crate clap;
extern crate crypto;
extern crate cuckoo;
extern crate env_logger;
extern crate ethereum_types;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate rlp;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod message;
mod worker;

use std::str::FromStr;

use clap::{App, AppSettings, Arg, SubCommand};
use ethereum_types::{clean_0x, H256, U256};
use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use self::message::Job;
use self::worker::{spawn_worker, BlakeWorker, CuckooConfig, CuckooWorker, Worker, WorkerConfig};

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn new_worker(config: &WorkerConfig) -> Box<Worker> {
    match config {
        WorkerConfig::Blake => Box::new(BlakeWorker::new()) as Box<Worker>,
        WorkerConfig::Cuckoo(config) => Box::new(CuckooWorker::new(config)) as Box<Worker>,
    }
}

fn get_work(req: Request<Body>, config: &WorkerConfig, submit_port: u16) -> BoxFut {
    let worker = new_worker(config);
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
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

// returns (listen_port, submit_port, worker_config)
fn get_options() -> Result<(u16, u16, WorkerConfig), String> {
    let matches = App::new("codechain-miner")
        .setting(AppSettings::SubcommandRequired)
        .subcommands(vec![
            SubCommand::with_name("cuckoo").args(&[
                Arg::with_name("max vertex").short("n").takes_value(true).required(true),
                Arg::with_name("max edge").short("m").takes_value(true).required(true),
                Arg::with_name("cycle length").short("l").takes_value(true).required(true),
            ]),
            SubCommand::with_name("blake"),
        ])
        .args(&[
            Arg::with_name("listening port").short("p").global(true).takes_value(true).default_value("3333"),
            Arg::with_name("submitting port").short("s").global(true).takes_value(true).default_value("8080"),
        ])
        .get_matches();

    let listen_port: u16 = matches.value_of("listening port").unwrap().parse().map_err(|_| "Invalid listening port")?;
    let submit_port: u16 = matches.value_of("submitting port").unwrap().parse().map_err(|_| "Invalid submitting port")?;

    let worker_config = match matches.subcommand() {
        ("cuckoo", Some(submatch)) => {
            let max_vertex = submatch.value_of("max vertex").unwrap().parse().map_err(|_| "Invalid max vertex")?;
            let max_edge = submatch.value_of("max edge").unwrap().parse().map_err(|_| "Invalid max edge")?;
            let cycle_length = submatch.value_of("cycle length").unwrap().parse().map_err(|_| "Invalid cycle length")?;
            WorkerConfig::Cuckoo(CuckooConfig {
                max_vertex,
                max_edge,
                cycle_length,
            })
        }
        ("blake", _) => WorkerConfig::Blake,
        _ => return Err("Invalid subcommand".into()),
    };

    Ok((listen_port, submit_port, worker_config))
}

fn main() -> Result<(), String> {
    env_logger::init();
    let (listen_port, submit_port, worker_config) = get_options()?;
    let addr = ([127, 0, 0, 1], listen_port).into();

    let server = Server::bind(&addr)
        .serve(move || {
            let config = worker_config.clone();
            service_fn(move |req| get_work(req, &config, submit_port))
        })
        .map_err(|e| error!("server error: {}", e));
    info!("Server started, listening on {:?}", addr);
    info!("It will submit to 127.0.0.1:{}", submit_port);

    hyper::rt::run(server);

    Ok(())
}
