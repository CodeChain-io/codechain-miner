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

mod blake;
mod cuckoo;
mod work;

use ethereum_types::U256;

pub use self::blake::BlakeWorker;
pub use self::cuckoo::{CuckooConfig, CuckooWorker};
pub use self::work::{work, spawn_worker, submit};

#[derive(Clone)]
pub enum WorkerConfig {
    Blake,
    Cuckoo(CuckooConfig),
}

pub trait Worker: Send {
    fn init(&mut self,  message: &[u8], nonce: u64, target: &U256);
    fn proceed(&mut self) -> Option<Vec<Vec<u8>>>;
    fn is_finished(&self) -> bool;
}
