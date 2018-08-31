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

use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ethereum_types::{clean_0x, H256, U256};
use futures::{future, Future};
use rustc_hex::ToHex;
use serde_json::Value as JsonValue;

use super::super::super::worker::{work, Worker};
use super::super::RpcRunner;
use super::client::Client;
use super::{dispatch_fn, Result};

#[derive(Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Deserialize)]
pub struct Rpc {
    pub id: usize,
    pub method: Option<String>,
    pub error: Option<RpcError>,
}

#[derive(Clone)]
pub struct Config {
    pub id: String,
    pub pwd: String,
    pub port: u16,
}

pub struct Runner {
    id: String,
    pwd: String,
    port: u16,
}

impl Runner {
    pub fn new(config: &Config) -> Self {
        Self {
            id: config.id.clone(),
            pwd: config.pwd.clone(),
            port: config.port,
        }
    }
}

impl RpcRunner for Runner {
    fn run(&self, recruiter: Arc<Fn() -> Box<Worker> + Send + Sync>, jobs: usize) {
        let job_id = Arc::new(AtomicUsize::new(1));
        let addr = ([127, 0, 0, 1], self.port).into();
        let client = Client::bind(&addr, self.id.to_owned(), self.pwd.to_owned())
            .serve(move || {
                let job_id = job_id.clone();
                let recruiter = Arc::clone(&recruiter);
                dispatch_fn(move |req| -> Result {
                    let vec = ::serde_json::to_vec(&req).unwrap();
                    let rpc: Rpc = ::serde_json::from_slice(&vec).unwrap();
                    if rpc.method.is_some() {
                        match rpc.method.unwrap().as_ref() {
                            "mining.notify" => {
                                let worker = recruiter();
                                let id = job_id.fetch_add(1, Ordering::SeqCst);
                                return Box::new(future::ok(get_work(worker, jobs, id, req)))
                            }
                            _ => warn!("Unsupported method"),
                        }
                    }

                    if rpc.error.is_some() {
                        let error = rpc.error.unwrap();
                        warn!("{} {}", error.code, error.message);
                    }
                    Box::new(future::ok(None))
                })
            })
            .map_err(|e| error!("stratum client error: {}", e));

        ::tokio::run(client);
    }
}

fn get_work(worker: Box<Worker>, jobs: usize, job_id: usize, req: JsonValue) -> Option<JsonValue> {
    let params = req["params"].clone();
    if params.is_array() && params.as_array().unwrap().len() == 2 {
        let hash = H256::from_str(clean_0x(&params[0].as_str().unwrap())).unwrap();
        let target = U256::from_str(clean_0x(&params[1].as_str().unwrap())).unwrap();
        if let Some(solution) = work(&hash, &target, worker, jobs) {
            return Some(submit(job_id, hash, solution))
        }
    };

    None
}

pub fn submit(job_id: usize, hash: H256, solution: Vec<Vec<u8>>) -> JsonValue {
    let seal: Vec<_> = solution.iter().map(|bytes| format!("0x{}", bytes.to_hex())).collect();
    json!({
        "jsonrpc": "2.0",
        "id": job_id,
        "method": "mining.submit",
        "params": [
            format!("0x{:x}", hash),
            seal,
        ],
    })
}
