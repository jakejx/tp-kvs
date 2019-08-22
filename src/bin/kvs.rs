extern crate clap;

use clap::{App, Arg, SubCommand};
use kvs::{KvStore, Result};
use std::error::Error;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Junxuan")
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").required(true).index(1))
                .arg(Arg::with_name("value").required(true).index(2)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("key").required(true).index(1)))
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("key").required(true).index(1)))
        .subcommand(SubCommand::with_name("version"))
        .get_matches();

    let mut kvs = KvStore::open(std::path::Path::new("."))?;

    match matches.subcommand() {
        ("get", Some(m)) => {
            let key = m.value_of("key").unwrap();
            let res = kvs.get(key.to_string());
            match res {
                Ok(value) => {
                    if let Some(v) = value {
                        println!("{}", v);
                        Ok(())
                    } else {
                        println!("Key not found");
                        std::process::exit(0);
                    }
                }
                Err(err) => {
                    println!("{}", err.description());
                    std::process::exit(1);
                }
            }
        }
        ("set", Some(m)) => {
            let key = m.value_of("key").unwrap();
            let value = m.value_of("value").unwrap();
            if let Err(err) = kvs.set(key.to_string(), value.to_string()) {
                println!("{}", err);
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        ("rm", Some(m)) => {
            let key = m.value_of("key").unwrap().to_string();
            if let Err(err) = kvs.remove(key) {
                println!("{}", err.description());
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        _ => std::process::exit(1),
    }
}
