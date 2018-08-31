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

use ethereum_types::{H256, U256};

use super::Worker;

static JOB_ID: AtomicUsize = AtomicUsize::new(0);

pub fn work(hash: &H256, target: &U256, mut worker: Box<Worker>, jobs: usize) -> Option<Vec<Vec<u8>>> {
    let id = JOB_ID.fetch_add(1, Ordering::SeqCst);
    info!("Starting a new Job {} with hash {}, target: {}", id, hash, target);
    for nonce in 0..=u64::max_value() {
        worker.init(hash, nonce, target);
        while !worker.is_finished() {
            if JOB_ID.load(Ordering::SeqCst) > id + jobs {
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
