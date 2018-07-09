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
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256) {
        self.message = message.to_vec();
        // FIXME: check message length
        LittleEndian::write_u64(&mut self.message, nonce);

        self.nonce = nonce;
        self.target = *target;
    }

    fn proceed(&mut self) -> Option<Vec<Vec<u8>>> {
        if self.is_finished() {
            return None
        }

        let hash = blake256(&self.message);
        if U256::from(hash) <= self.target {
            let nonce_bytes = rlp::encode(&self.nonce).to_vec();

            return Some(vec![nonce_bytes])
        }
        None
    }

    fn is_finished(&self) -> bool {
        self.is_executed
    }
}
