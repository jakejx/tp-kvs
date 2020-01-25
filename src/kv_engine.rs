use crate::errors::Result;

/// Trait for engines that are compatible with the KV Store
pub trait KvsEngine: Clone + Send + 'static {
    /// Get a particular key from the store
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Set the value of a key. If the key already exists, it will overwrite the value.
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Removes the key from the store. If the key does not exist, a KeyNotFound error will be returned.
    fn remove(&self, key: String) -> Result<()>;
}
