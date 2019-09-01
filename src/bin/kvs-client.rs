extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg, SubCommand};
use serde::Serialize;
use kvs::{KvStore, Result, KvRequest};
use slog::Drain;
use std::io::{Write, BufWriter, BufReader};
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
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").required(true).index(1))
                .arg(Arg::with_name("value").required(true).index(2)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("key").required(true).index(1)))
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("key").required(true).index(1)))
        .subcommand(SubCommand::with_name("version"))
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    info!(logger, "Parsed configuration"; "addr" => addr);

    let mut connection = TcpStream::connect(addr)?;
    debug!(logger, "Connected to {}", addr);

    send_request(connection, KvRequest::Get("test".to_string()))?;

    // match matches.subcommand() {
    //     ("get", Some(m)) => {
    //         let key = m.value_of("key").unwrap();
    //         let req = KvRequest::Get(key.to_string());
    //         let res = kvs.get(key.to_string());
    //         match res {
    //             Ok(value) => {
    //                 if let Some(v) = value {
    //                     println!("{}", v);
    //                     Ok(())
    //                 } else {
    //                     println!("Key not found");
    //                     std::process::exit(0);
    //                 }
    //             }
    //             Err(err) => {
    //                 println!("{}", err.description());
    //                 std::process::exit(1);
    //             }
    //         }
    //     }
    //     ("set", Some(m)) => {
    //         let key = m.value_of("key").unwrap();
    //         let value = m.value_of("value").unwrap();
    //         if let Err(err) = kvs.set(key.to_string(), value.to_string()) {
    //             println!("{}", err);
    //             std::process::exit(1);
    //         } else {
    //             Ok(())
    //         }
    //     }
    //     ("rm", Some(m)) => {
    //         let key = m.value_of("key").unwrap().to_string();
    //         if let Err(err) = kvs.remove(key) {
    //             println!("{}", err.description());
    //             std::process::exit(1);
    //         } else {
    //             Ok(())
    //         }
    //     }
    //     _ => std::process::exit(1),
    // }

    Ok(())
}

fn send_request(connection: TcpStream, request: KvRequest) -> Result<()> {
    let mut writer = BufWriter::new(&connection);
    let mut reader = BufReader::new(&connection);

    serde_json::to_writer(writer, &request)?;

    let response_stream = serde_json::from_reader(reader).into_iter::<KvResponse>();

    Ok(())
}
