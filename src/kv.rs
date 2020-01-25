use std::sync::Arc;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::errors::{KvError, Result};
use crate::kv_engine::KvsEngine;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set(String, String),
    Rm(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

/// A key-value store
#[derive(Debug, Clone)]
pub struct KvStore {
    store_path: PathBuf,
    index: Arc<RwLock<BTreeMap<String, CommandPos>>>,
    writer: Arc<RwLock<KvStoreWriter>>,
    readers: KvStoreReader,
}

#[derive(Debug)]
struct KvStoreWriter {
    writer: File,
    current_gen: u64,
    compact_space: u64,
    safe_gen: u64,
}

#[derive(Debug)]
struct KvStoreReader {
    store_path: PathBuf,
    readers: Arc<RwLock<HashMap<u64, File>>>,
}

impl Clone for KvStoreReader {
    fn clone(&self) -> Self {
        let mut readers = HashMap::new();
        for gen in self.readers.read().unwrap().keys().cloned() {
            let reader = File::open(format_log_path(&self.store_path, gen)).unwrap();
            readers.insert(gen, reader);
        }

        KvStoreReader {
            store_path: self.store_path.clone(),
            readers: Arc::new(RwLock::new(readers)),
        }
    }
}

impl KvStore {
    /// Open a KvStore
    pub fn open(store_path: impl Into<PathBuf>) -> Result<Self> {
        let store_path = store_path.into();
        std::fs::create_dir_all(&store_path)?;

        let mut compact_space = 0;
        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();
        // find files that end with .log in the log folder
        let mut log_files: Vec<u64> = store_path
            .read_dir()?
            .flat_map(|f| -> Result<_> { Ok(f?.path()) })
            .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
            .flat_map(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .map(|name| name.trim_end_matches(".log"))
                    .map(str::parse::<u64>)
            })
            .flatten()
            .collect();
        // sort the log files
        log_files.sort_unstable();

        // for each generation, load log into index
        // create and store a reader for each log file
        for &gen in &log_files {
            let mut reader = File::open(format_log_path(&store_path, gen))?;
            let free = KvStore::load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
            compact_space += free;
        }

        // find the latest generation, start at 1 if none
        let current_gen = log_files.last().map_or(1, |gen| gen + 1);
        let (reader, writer) = KvStore::new_log(&store_path, current_gen)?;
        readers.insert(current_gen, reader);

        let readers = KvStoreReader {
            store_path,
            readers: Arc::new(RwLock::new(readers)),
        };

        let writer = KvStoreWriter {
            writer: writer,
            current_gen,
            compact_space,
            safe_gen: *log_files.first().unwrap_or(&0),
        };

        let store = KvStore {
            store_path: readers.store_path.clone(),
            index: Arc::new(RwLock::new(index)),
            writer: Arc::new(RwLock::new(writer)),
            readers
        };

        Ok(store)
    }

    /// Load the KvStore
    fn load(
        gen: u64,
        mut reader: &mut File,
        index: &mut BTreeMap<String, CommandPos>,
    ) -> Result<u64> {
        let mut free_space = 0;
        let mut pos = reader.seek(SeekFrom::Start(0))?;
        let mut stream = serde_json::Deserializer::from_reader(&mut reader).into_iter::<Command>();

        // read each json-encoded log instruction and apply to the in-memory store
        // the store only holds the key name and the location to find the value
        while let Some(command) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            match command? {
                Command::Set(key, _) => {
                    let command_pos = CommandPos {
                        gen,
                        pos,
                        len: new_pos - pos,
                    };
                    if let Some(old_cmd) = index.insert(key, command_pos) {
                        free_space += old_cmd.len
                    }
                }
                Command::Rm(key) => {
                    if let Some(old_cmd) = index.remove(&key) {
                        free_space += old_cmd.len
                    }
                }
            }
            pos = new_pos;
        }

        Ok(free_space)
    }

    /// Creates a new log file and returns a reader & writer for that log file
    fn new_log(store: &Path, gen: u64) -> Result<(File, File)> {
        let log_path = format_log_path(&store, gen);
        let writer = OpenOptions::new()
            .append(true)
            .read(true)
            .create(true)
            .open(&log_path)?;
        let reader = File::open(&log_path)?;

        Ok((reader, writer))
    }

    /// Read the value from the log files given a `CommandPos`
    fn read_log(&self, cmd_pos: &CommandPos) -> Result<Command> {
        let gen = cmd_pos.gen;
        let len = cmd_pos.len;
        // obtain the correct reader for the generation
        let reader_map = &mut self.readers.readers.write().unwrap();
        let mut reader = reader_map.get(&gen);
        if reader.is_none() {
            let new_reader = File::open(format_log_path(&self.readers.store_path, gen)).unwrap();
            reader_map.insert(gen, new_reader);
            reader = reader_map.get(&gen);
        }
        let mut reader = reader.unwrap();
        // move reader to the start position
        reader.seek(SeekFrom::Start(cmd_pos.pos))?;
        let handle = reader.take(len);

        let cmd = serde_json::from_reader(handle)?;
        Ok(cmd)
    }

    fn write_log(mut writer: &File, cmd: &Command, gen: u64) -> Result<CommandPos> {
        let serialized = serde_json::to_string(cmd)?;
        // obtain the last position in the log file
        let pos = writer.seek(SeekFrom::End(0))?;
        let len = writer.write(serialized.as_bytes())? as u64;
        Ok(CommandPos { gen, pos, len })
    }

    fn compact(&self) -> Result<()> {
        println!("Compaction called");
        // get lock on index
        let mut index = self.index.write().unwrap();
        let mut kv_writer = self.writer.write().unwrap();
        let current_gen = kv_writer.current_gen + 1;
        let (reader, mut writer) = KvStore::new_log(&self.store_path, current_gen)?;
        // compact entries into a new generation
        let mut new_pos = 0;

        // iterate through the entries inside the index
        for (_, cmd_pos) in index.iter_mut() {
            // read the value of the key from the correct reader
            let reader = self.readers.readers.write().unwrap();
            let mut old_reader = reader
                .get(&cmd_pos.gen)
                .ok_or(KvError::InternalError)?;
            old_reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let mut entry_reader = old_reader.take(cmd_pos.len);

            // copy the value to the latest generation
            let len = std::io::copy(&mut entry_reader, &mut writer)?;
            *cmd_pos = CommandPos {
                gen: current_gen,
                pos: new_pos,
                len,
            };
            new_pos += len;
        }

        // delete old generations TODO: optimise this portion
        for gen in 1..current_gen {
            let log_path = format_log_path(&self.store_path, gen);
            let _ = std::fs::remove_file(log_path);
        }

        // recreate readers hashmap
        // this must be done separately as it is not possible to mutate the index
        // while iterating over it
        let mut new_readers = HashMap::new();
        new_readers.insert(current_gen, reader);

        let mut readers = self.readers.readers.write().unwrap();
        *readers = new_readers;
        kv_writer.current_gen = current_gen;
        kv_writer.compact_space = 0;
        kv_writer.safe_gen = current_gen;
        kv_writer.writer = writer;

        Ok(())
    }
}

fn format_log_path(path: &Path, gen: u64) -> PathBuf {
    let fname = format!("{}.log", gen);
    path.join(fname)
}

impl KvsEngine for KvStore {
    /// Retrieves the value associated with the key.
    fn get(&self, key: String) -> Result<Option<String>> {
        let index = self.index.read().unwrap();
        let command_pos = index.get(&key);
        if let Some(command) = command_pos {
            let cmd = self.read_log(&command)?;
            match cmd {
                Command::Set(_, value) => Ok(Some(value)),
                Command::Rm(_) => Err(KvError::InternalError),
            }
        } else {
            Ok(None)
        }
    }

    /// Sets the value of a key. If the key already exists, it will overwrite the current value.
    fn set(&self, key: String, value: String) -> Result<()> {
        {
            let mut writer = self.writer.write().unwrap();
            let cmd = Command::Set(key.to_string(), value.to_string());
            let cmd_pos = KvStore::write_log(&writer.writer, &cmd, writer.current_gen)?;
            if let Some(old_cmd) = self.index.write().unwrap().insert(key, cmd_pos) {
                writer.compact_space += old_cmd.len;
            }
        }

        if self.writer.read().unwrap().compact_space > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Removes the key and its value in the key-value store.
    fn remove(&self, key: String) -> Result<()> {
        {
            let mut index = self.index.write().unwrap();
            let mut writer = self.writer.write().unwrap();
            if let Some(old_cmd) = index.remove(&key) {
                let cmd = Command::Rm(key.to_string());
                KvStore::write_log(&writer.writer, &cmd, writer.current_gen)?;
                writer.compact_space += old_cmd.len;
            } else {
                return Err(KvError::KeyNotFound);
            }
        }

        if self.writer.read().unwrap().compact_space > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }
}
