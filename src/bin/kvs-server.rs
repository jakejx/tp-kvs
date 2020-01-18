extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg};
use kvs::{KvError, KvRequest, KvResponse, KvStore, KvsEngine, Result};
use slog::Drain;
use slog::Logger;
use std::error::Error;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;

fn valid_engine(engine: String) -> std::result::Result<(), String> {
    if engine == "kvs" || engine == "sled" {
        return Ok(());
    }
    Err(String::from(
        "The server only supports kvs or sled as an engine",
    ))
}

fn valid_ip(ip: String) -> std::result::Result<(), String> {
    match ip.to_socket_addrs() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Invalid address provided")),
    }
}

fn init_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

fn main() -> Result<()> {
    let logger = init_logger();
    info!(logger, "Kvs server started"; "version" => env!("CARGO_PKG_VERSION"));

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Junxuan")
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .value_name("ADDR")
                .default_value("127.0.0.1:4000")
                .validator(valid_ip),
        )
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .value_name("ENGINE")
                .default_value("kvs")
                .validator(valid_engine),
        )
        .get_matches();

    let engine = matches.value_of("engine").unwrap();
    let addr = matches.value_of("addr").unwrap();
    let store_path = "./log";

    info!(logger, "Parsed configuration"; "engine" => engine, "addr" => addr);

    if !compatible_engine(engine, store_path) {
        eprintln!("Server started with incompatible engine.");
        error!(logger, "{} engine incompatible with existing store", engine);
        std::process::exit(1)
    }

    let listener = TcpListener::bind(addr)?;
    debug!(logger, "Listening on {}", addr);

    let mut store: Box<dyn KvsEngine> = Box::new(KvStore::open(store_path)?);

    info!(logger, "Loaded store from {}", store_path);

    for client in listener.incoming() {
        let _ = handle_client(client?, &logger, &mut store);
    }

    Ok(())
}

fn compatible_engine(engine: &str, path: &str) -> bool {
    let store_path = Path::new(path);
    if !store_path.exists() {
        return true;
    } else {
        let db_path = store_path.join("db");
        if db_path.exists() && engine == "sled" {
            return true;
        }

        if !db_path.exists() && engine == "kvs" {
            return true;
        }
    }
    return false;
}

fn handle_client(client: TcpStream, logger: &Logger, store: &mut Box<dyn KvsEngine>) -> Result<()> {
    debug!(logger, "Client connected");
    let reader = BufReader::new(&client);
    let writer = BufWriter::new(&client);

    let mut request_stream = serde_json::Deserializer::from_reader(reader).into_iter::<KvRequest>();

    let req = request_stream.next().ok_or(KvError::MalformedRequest);
    match req {
        Ok(req) => match req {
            Ok(cmd) => match cmd {
                KvRequest::Get(k) => {
                    info!(logger, "Get key: {}", k);
                    handle_get(store, writer, k)
                }
                KvRequest::Set(k, v) => {
                    info!(logger, "Set key: {} to value: {}", k, v);
                    handle_set(store, writer, k, v)
                }
                KvRequest::Rm(k) => {
                    info!(logger, "Delete key: {}", k);
                    handle_rm(store, writer, k)
                }
            },
            Err(_) => {
                let res = KvResponse::Error("Malformed request".to_string());
                serde_json::to_writer(writer, &res)?;
                Ok(())
            }
        },
        Err(_) => {
            let res = KvResponse::Error("Malformed request".to_string());
            serde_json::to_writer(writer, &res)?;
            Ok(())
        }
    }
}

fn handle_set<W: Write>(
    store: &mut Box<dyn KvsEngine>,
    writer: W,
    key: String,
    value: String,
) -> Result<()> {
    let value = store.set(key, value);
    match value {
        Ok(_) => {
            let res = KvResponse::Success(None);
            let _ = serde_json::to_writer(writer, &res)?;
            Ok(())
        }
        Err(err) => {
            let err_res = KvResponse::Error(err.description().to_string());
            let _ = serde_json::to_writer(writer, &err_res)?;
            Err(err)
        }
    }
}

fn handle_get<W: Write>(store: &mut Box<dyn KvsEngine>, writer: W, key: String) -> Result<()> {
    let value = store.get(key);
    match value {
        Ok(val) => {
            let res = KvResponse::Success(val);
            let _ = serde_json::to_writer(writer, &res)?;
            Ok(())
        }
        Err(err) => {
            let err_res = KvResponse::Error(err.description().to_string());
            let _ = serde_json::to_writer(writer, &err_res)?;
            Err(err)
        }
    }
}

fn handle_rm<W: Write>(store: &mut Box<dyn KvsEngine>, writer: W, key: String) -> Result<()> {
    let value = store.remove(key);
    match value {
        Ok(_) => {
            let res = KvResponse::Success(None);
            let _ = serde_json::to_writer(writer, &res)?;
            Ok(())
        }
        Err(err) => {
            let err_res = KvResponse::Error(err.description().to_string());
            let _ = serde_json::to_writer(writer, &err_res)?;
            Err(err)
        }
    }
}
