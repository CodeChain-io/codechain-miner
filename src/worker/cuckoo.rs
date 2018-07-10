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
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256) {
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
            if U256::from(hash) <= self.target {
                info!("Solution found: {:?}", proof);
                let nonce_bytes = ::rlp::encode(&self.nonce).to_vec();
                let proof_bytes = ::rlp::encode_list(&proof).to_vec();

                return Some(vec![nonce_bytes, proof_bytes])
            }
        }
        None
    }

    fn is_finished(&self) -> bool {
        self.is_executed
    }
}
