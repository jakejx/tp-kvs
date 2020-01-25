extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg, SubCommand};
use kvs::{KvError, KvRequest, KvResponse, Result};
use slog::Drain;
use std::io::{self, BufReader, BufWriter};
use std::net::{TcpStream, ToSocketAddrs};

fn init_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

fn valid_ip(ip: String) -> std::result::Result<(), String> {
    match ip.to_socket_addrs() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Invalid address provided")),
    }
}

fn main() -> Result<()> {
    let logger = init_logger();
    info!(logger, "Kvs client started"; "version" => env!("CARGO_PKG_VERSION"));

    let address_arg = Arg::with_name("addr")
        .long("addr")
        .value_name("ADDR")
        .default_value("127.0.0.1:4000")
        .validator(valid_ip);
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Junxuan")
        .subcommand(
            SubCommand::with_name("set")
                .arg(&address_arg)
                .arg(Arg::with_name("key").required(true).index(1))
                .arg(Arg::with_name("value").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("get")
                .arg(&address_arg)
                .arg(Arg::with_name("key").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .arg(&address_arg)
                .arg(Arg::with_name("key").required(true).index(1)),
        )
        .subcommand(SubCommand::with_name("version"))
        .get_matches();

    match matches.subcommand() {
        ("get", Some(m)) => {
            let key = m.value_of("key").unwrap().to_string();
            let addr = m.value_of("addr").unwrap();
            let connection = new_connection(addr, &logger)?;
            info!(logger, "Get key: {}", key);
            let res = handle_get(connection, key)?;
            match res {
                KvResponse::Success(value) => {
                    if let Some(v) = value {
                        println!("{}", v);
                        Ok(())
                    } else {
                        println!("Key not found");
                        std::process::exit(0);
                    }
                }
                KvResponse::Error(err) => {
                    println!("{}", err);
                    std::process::exit(1);
                }
            }
        }
        ("set", Some(m)) => {
            let key = m.value_of("key").unwrap().to_string();
            let value = m.value_of("value").unwrap().to_string();
            let addr = m.value_of("addr").unwrap();
            let connection = new_connection(addr, &logger)?;
            info!(logger, "Set key: {} to value: {}", key, value);
            if let KvResponse::Error(err) = handle_set(connection, key, value)? {
                println!("{}", err);
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        ("rm", Some(m)) => {
            let key = m.value_of("key").unwrap().to_string();
            let addr = m.value_of("addr").unwrap();
            let connection = new_connection(addr, &logger)?;
            info!(logger, "Remove key: {}", key);
            if let KvResponse::Error(err) = handle_rm(connection, key)? {
                eprintln!("{}", err);
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        _ => std::process::exit(1),
    }
}

fn new_connection(addr: &str, logger: &slog::Logger) -> io::Result<TcpStream> {
    info!(logger, "Parsed configuration"; "addr" => addr);

    let connection = TcpStream::connect(addr)?;
    debug!(logger, "Connected to {}", addr);
    Ok(connection)
}

fn handle_get(connection: TcpStream, key: String) -> Result<KvResponse> {
    let req = KvRequest::Get(key);
    send_request(connection, req)
}

fn handle_set(connection: TcpStream, key: String, value: String) -> Result<KvResponse> {
    let req = KvRequest::Set(key, value);
    send_request(connection, req)
}

fn handle_rm(connection: TcpStream, key: String) -> Result<KvResponse> {
    let req = KvRequest::Rm(key);
    send_request(connection, req)
}

fn send_request(connection: TcpStream, request: KvRequest) -> Result<KvResponse> {
    let writer = BufWriter::new(&connection);
    let reader = BufReader::new(&connection);

    serde_json::to_writer(writer, &request)?;

    let mut response_stream =
        serde_json::Deserializer::from_reader(reader).into_iter::<KvResponse>();
    let res = response_stream.next().ok_or(KvError::InternalError)??;

    Ok(res)
}
