use serde::{Deserialize, Serialize};

pub type Key = String;
pub type Value = String;

/// Commands that can be sent from the client to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum KvRequest {
    /// Get the value of key
    Get(Key),
    /// Set the value of key, where the first value refers to the key and the second is the value
    Set(Key, Value),
    /// Remove the value of key
    Rm(Key),
}

/// Response from the kv server
#[derive(Debug, Serialize, Deserialize)]
pub enum KvResponse {
    /// A successful operation
    Success(Option<String>),
    /// An error on the server side
    Error(String)
}
