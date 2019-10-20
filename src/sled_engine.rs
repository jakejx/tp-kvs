use crate::errors::KvError;
use crate::errors::Result;
use crate::KvsEngine;
use sled::{Db, IVec};
use std::path::Path;

/// A key-value store using the Sled engine
pub struct SledEngine {
    store: Db,
}

impl SledEngine {
    /// Open a new store
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let store = Db::open(path)?;

        Ok(SledEngine { store })
    }
}

impl KvsEngine for SledEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let val = self.store.get(key)?;
        let conv = val
            .map(|x| x.to_vec())
            .and_then(|x| String::from_utf8(x).ok());

        Ok(conv)
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.store
            .insert(IVec::from(key.as_bytes()), IVec::from(value.as_bytes()))?;
        let _ = self.store.flush();

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let val = self.store.remove(IVec::from(key.as_bytes()))?;
        let _ = self.store.flush();

        val.map(|_| ()).ok_or(KvError::KeyNotFound)
    }
}
