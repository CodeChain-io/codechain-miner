mod blake;
mod cuckoo;
mod work;

use ethereum_types::U256;

pub use self::blake::BlakeWorker;
pub use self::cuckoo::CuckooWorker;
pub use self::work::{work, spawn_worker, submit};

pub trait Worker: Send {
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256);
    fn proceed(&mut self) -> Option<Vec<Vec<u8>>>;
    fn is_finished(&self) -> bool;
}
