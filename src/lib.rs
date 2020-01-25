//!
//! kvs, a key-value story library
//!
//! This library provides an in-memory key-value store

#![deny(missing_docs)]

#[macro_use]
extern crate slog;

mod errors;
mod kv;
mod kv_engine;
mod sled_engine;
mod kv_protocol;
pub mod server;
pub mod thread_pool;

pub use crate::errors::{KvError, Result};
pub use crate::kv::KvStore;
pub use crate::sled_engine::SledEngine;
pub use crate::kv_engine::KvsEngine;
pub use crate::kv_protocol::{KvRequest, KvResponse};
pub use crate::thread_pool::{NaiveThreadPool, ThreadPool, SharedQueueThreadPool};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
