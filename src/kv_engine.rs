use crate::errors::{KvError, Result};

/// Trait for engines that are compatible with the KV Store
pub trait KvsEngine {
    /// Get a particular key from the store
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Set the value of a key. If the key already exists, it will overwrite the value.
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Removes the key from the store. If the key does not exist, a KeyNotFound error will be returned.
    fn remove(&mut self, key: String) -> Result<()>;
}
