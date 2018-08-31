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

extern crate bytes;
extern crate ethereum_types;
#[macro_use]
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate tokio_executor;

mod rpc;
mod worker;

use std::sync::Arc;

use rpc::{HttpRunner, RpcRunner, StratumRunner};

pub use rpc::{HttpConfig, RpcConfig, StratumConfig};
pub use worker::Worker;

pub fn run<C: 'static + Config>(config: C) {
    let rpc_runner = match config.rpc_config() {
        RpcConfig::Http(config) => Box::new(HttpRunner::new(&config)) as Box<RpcRunner>,
        RpcConfig::Stratum(config) => Box::new(StratumRunner::new(&config)) as Box<RpcRunner>,
    };
    let jobs = config.jobs();
    let recruiter = Arc::new(move || config.worker());

    rpc_runner.run(recruiter, jobs);
}

pub trait Config: Send + Sync {
    fn rpc_config(&self) -> RpcConfig;
    fn jobs(&self) -> usize;
    fn worker(&self) -> Box<Worker>;
}
