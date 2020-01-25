extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg};
use kvs::server::KvServer;
use kvs::{KvStore, Result, SharedQueueThreadPool, SledEngine, ThreadPool};
use slog::Drain;
use slog::Logger;
use std::net::ToSocketAddrs;
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
    let pool = SharedQueueThreadPool::new(4)?;

    info!(logger, "Parsed configuration"; "engine" => engine, "addr" => addr);

    if !compatible_engine(engine, store_path) {
        eprintln!("Server started with incompatible engine.");
        error!(logger, "{} engine incompatible with existing store", engine);
        std::process::exit(1)
    }
    info!(logger, "Loading store from {}", store_path);

    match engine {
        "sled" => {
            let server = KvServer::new(SledEngine::open(store_path)?, pool, logger);
            let _ = server.run(addr);
        }
        _ => {
            let server = KvServer::new(KvStore::open(store_path)?, pool, logger);
            let _ = server.run(addr);
        }
    };

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

        return false;
    }
}
