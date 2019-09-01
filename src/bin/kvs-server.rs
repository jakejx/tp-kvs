extern crate clap;

use clap::{App, Arg, SubCommand};
use kvs::{KvStore, Result};
use std::error::Error;

fn valid_engine(input: String) -> std::result::Result<(), String> {
    if (input == "kvs" || input == "sled") {
        return Ok(());
    }
    Err(String::from(
        "The server only supports kvs or sled as an engine",
    ))
}

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Junxuan")
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .value_name("ADDR")
                .default_value("127.0.0.1:4000"),
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

    Ok(())
}
