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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::spawn;

use ethereum_types::{H256, U256};
use futures::Future;
use hyper::header::HeaderValue;
use hyper::rt::run;
use hyper::{Body, Client, Method, Request};
use rustc_hex::ToHex;

use super::Worker;

static JOB_ID: AtomicUsize = AtomicUsize::new(0);

pub fn spawn_worker(hash: H256, target: U256, worker: Box<Worker>, submit_port: u16) {
    spawn(move || {
        let id = JOB_ID.fetch_add(1, Ordering::SeqCst);
        info!("Starting a new job {}", id);
        if let Some(solution) = work(id, &hash, &target, worker) {
            submit(hash, solution, submit_port);
        }
    });
}

pub fn work(id: usize, hash: &H256, target: &U256, mut worker: Box<Worker>) -> Option<Vec<Vec<u8>>> {
    info!("Job start with hash {}, target: {}", hash, target);
    for nonce in 0..=u64::max_value() {
        worker.init(hash, nonce, target);
        while !worker.is_finished() {
            if JOB_ID.load(Ordering::SeqCst) > id + 1 {
                info!("A new job submitted. Stopping the job {}", id);
                return None
            }
            match worker.proceed() {
                Some(solution) => {
                    info!("Nonce: {}", nonce);
                    return Some(solution)
                }
                None => {}
            }
        }
    }
    info!("Could not find the solution for hash {}", hash);
    None
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
