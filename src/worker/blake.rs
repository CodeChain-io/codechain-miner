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

pub struct BlakeWorker {
    message: Vec<u8>,
    nonce: u64,
    target: U256,
    is_executed: bool,
}

impl BlakeWorker {
    pub fn new() -> Self {
        Self {
            message: Vec::new(),
            nonce: 0,
            target: U256::zero(),
            is_executed: false,
        }
    }
}

impl Worker for BlakeWorker {
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
        let hash = blake256(&self.message);
        let current_score = U256::from(hash);
        if current_score <= self.target {
            info!("Solution found.\n  nonce: {}", self.nonce);
            let nonce_bytes = rlp::encode(&self.nonce).to_vec();

            return Some(vec![nonce_bytes])
        }
        trace!("Retry.\n score : {:#0128x}\n target: {:#0128x}", current_score, self.target);
        None
    }

    fn is_finished(&self) -> bool {
        self.is_executed
    }
}
