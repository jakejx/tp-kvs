use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::errors::{KvError, Result};

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set(String, String),
    Rm(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandPos {
    pos: u64,
    len: u64,
}

/// A key-value store
pub struct KvStore {
    store: BTreeMap<String, CommandPos>,
    writer: File,
    reader: File,
}

impl KvStore {
    /// Load the KvStore
    fn load(&mut self) -> Result<()> {
        let mut pos = self.reader.seek(SeekFrom::Start(0))?;
        let mut stream =
            serde_json::Deserializer::from_reader(&mut self.reader).into_iter::<Command>();
        while let Some(command) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            match command? {
                Command::Set(key, _) => {
                    let command_pos = CommandPos {
                        pos,
                        len: new_pos - pos,
                    };
                    self.store.insert(key, command_pos)
                }
                Command::Rm(key) => self.store.remove(&key),
            };
            pos = new_pos;
        }

        Ok(())
    }

    /// Open a KvStore
    pub fn open(store: &Path) -> Result<Self> {
        let log_path = store.join("log");
        let writer = OpenOptions::new()
            .append(true)
            .read(true)
            .create(true)
            .open(&log_path)?;
        let read_file = OpenOptions::new().read(true).open(&log_path)?;

        let mut store = KvStore {
            store: BTreeMap::new(),
            writer,
            reader: read_file,
        };

        store.load()?;
        Ok(store)
    }

    /// Sets the value of a key. If the key already exists, it will overwrite the current value.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set(key.to_string(), value.to_string());
        let serialized = serde_json::to_string(&cmd)?;
        let pos = self.writer.seek(SeekFrom::End(0))?;
        let len = self.writer.write(serialized.as_bytes())? as u64;
        self.store.insert(key, CommandPos {
            pos,
            len
        });

        Ok(())
    }

    /// Retrieves the value associated with the key.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let command_pos = self.store.get(&key);
        if let Some(command) = command_pos {
            self.reader.seek(SeekFrom::Start(command.pos))?;
            let mut deserializer =
                serde_json::Deserializer::from_reader(&mut self.reader).into_iter::<Command>();
            if let Some(c) = deserializer.next() {
                match c? {
                    Command::Set(_, value) => Ok(Some(value)),
                    Command::Rm(_) => Err(KvError::InternalError),
                }
            } else {
                Err(KvError::InternalError)
            }
        } else {
            Ok(None)
        }
    }

    /// Removes the key and its value in the key-value store.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if let None = self.store.remove(&key) {
            return Err(KvError::KeyNotFound);
        }
        let cmd = Command::Rm(key.to_string());
        let serialized = serde_json::to_string(&cmd)?;
        self.writer.write(serialized.as_bytes())?;

        Ok(())
    }
}
