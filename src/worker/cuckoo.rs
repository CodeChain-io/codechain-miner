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

use byteorder::{ByteOrder, LittleEndian};
use crypto::blake2b::Blake2b;
use crypto::digest::Digest;
use cuckoo::Cuckoo;
use ethereum_types::{H256, U256};
use rlp;

use super::Worker;

pub fn blake256<T: AsRef<[u8]>>(s: T) -> H256 {
    let input = s.as_ref();
    let mut result = H256::default();
    let mut hasher = Blake2b::new(32);
    hasher.input(input);
    hasher.result(&mut *result);
    result
}

#[derive(Clone)]
pub struct CuckooConfig {
    pub max_vertex: usize,
    pub max_edge: usize,
    pub cycle_length: usize,
}

pub struct CuckooWorker {
    message: Vec<u8>,
    nonce: u64,
    target: U256,
    is_executed: bool,
    solver: Cuckoo,
}

impl CuckooWorker {
    pub fn new(config: &CuckooConfig) -> Self {
        Self {
            message: Vec::new(),
            nonce: 0,
            target: U256::zero(),
            is_executed: false,
            solver: Cuckoo::new(config.max_vertex, config.max_edge, config.cycle_length),
        }
    }
}

impl Worker for CuckooWorker {
    fn init(&mut self, message: &[u8], nonce: u64, target: &U256) {
        self.message = message.to_vec();
        // FIXME: check message length
        LittleEndian::write_u64(&mut self.message, nonce);

        self.nonce = nonce;
        self.target = *target;

        self.is_executed = false;
    }

    fn proceed(&mut self) -> Option<Vec<Vec<u8>>> {
        if self.is_finished() {
            return None
        }
        self.is_executed = true;
        if let Some(proof) = self.solver.solve(&self.message) {
            let hash = blake256(rlp::encode_list(&proof));
            let current_score = U256::from(hash);
            if current_score <= self.target {
                info!("Solution found.\n  nonce: {}\n  proof: {:?}", self.nonce, proof);
                let nonce_bytes = ::rlp::encode(&self.nonce).to_vec();
                let proof_bytes = ::rlp::encode_list(&proof).to_vec();

                return Some(vec![nonce_bytes, proof_bytes])
            }
            trace!("Retry.\n score : {:#0128x}\n target: {:#0128x}", current_score, self.target);
        }
        None
    }

    fn is_finished(&self) -> bool {
        self.is_executed
    }
}
