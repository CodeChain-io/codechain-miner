use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::spawn;

use ethereum_types::{H256, U256};
use hyper::{Body, Client, Method, Request};
use hyper::header::HeaderValue;
use hyper::rt::{self, Future};

use super::Worker;

const JOB_ID: AtomicUsize = AtomicUsize::new(0);

pub fn spawn_worker(hash: H256, target: U256, worker: Box<Worker>, submit_port: u16) {
    spawn(move || {
        let id = JOB_ID.fetch_add(1, Ordering::SeqCst);
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
            if JOB_ID.load(Ordering::SeqCst) != id {
                return None
            }
            match worker.proceed() {
                Some(solution) => return Some(solution),
                None => {},
            }
        }
    }
    None
}

pub fn submit(hash: H256, solution: Vec<Vec<u8>>, port: u16) {
    let seal: Vec<_> = solution.iter().map(|bytes| {
        bytes.iter().fold(String::from("0x"), |acc, x| format!("{}{:02x}", acc, x))
    }).collect();

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

    rt::run(Client::new().request(req).map(|_| {}).map_err(|err| { eprintln!("Error {}", err); }));
}
