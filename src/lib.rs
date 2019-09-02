//!
//! kvs, a key-value story library
//!
//! This library provides an in-memory key-value store

#![deny(missing_docs)]

mod errors;
mod kv;
mod kv_protocol;

pub use crate::errors::{KvError, Result};
pub use crate::kv::KvStore;
pub use crate::kv_protocol::{KvRequest, KvResponse};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
