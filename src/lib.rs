//!
//! kvs, a key-value story library
//!
//! This library provides an in-memory key-value store

#![deny(missing_docs)]

mod kv;
mod errors;
mod reader_with_pos;

pub use crate::kv::KvStore;
pub use crate::errors::Result;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
