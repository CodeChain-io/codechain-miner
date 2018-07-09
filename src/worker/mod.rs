mod cuckoo;

use ethereum_types::U256;

pub use self::cuckoo::CuckooWorker;

pub trait Worker {
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256);
    fn proceed(&mut self) -> Option<Vec<Vec<u8>>>;
    fn is_finished(&self) -> bool;
}
