extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg};
use kvs::{KvStore, Result, KvRequest, KvError};
use slog::Drain;
use std::net::{ToSocketAddrs, TcpListener, TcpStream};
use slog::Logger;
use std::io::{BufReader, BufWriter, BufRead};

fn valid_engine(engine: String) -> std::result::Result<(), String> {
    if (engine == "kvs" || engine == "sled") {
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

    // let mut kvs = KvStore::open(std::path::Path::new("."))?;
    let engine = matches.value_of("engine").unwrap();
    let addr = matches.value_of("addr").unwrap();

    info!(logger, "Parsed configuration"; "engine" => engine, "addr" => addr);

    let listener = TcpListener::bind(addr)?;
    debug!(logger, "Listening on {}", addr);

    for client in listener.incoming() {
        handle_client(client?, &logger)?;
    }

    Ok(())
}

fn handle_client(client: TcpStream, logger: &Logger) -> Result<()> {
    debug!(logger, "Client connected");
    let reader = BufReader::new(&client);
    let mut writer = BufWriter::new(&client);

    let mut request_stream = serde_json::Deserializer::from_reader(reader).into_iter::<KvRequest>();

    let req = request_stream.next().ok_or(KvError::MalformedRequest);
    dbg!(&req?);

    Ok(())
}
