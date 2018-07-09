use ethereum_types::U256;

use super::Worker;

pub struct CuckooWorker {
}

impl CuckooWorker {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Worker for CuckooWorker {
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256) {
        unimplemented!()
    }

    fn proceed(&mut self) -> Option<Vec<Vec<u8>>> {
        if self.is_finished() {
            return None
        }
        unimplemented!()
    }

    fn is_finished(&self) -> bool {
        unimplemented!()
    }
}
