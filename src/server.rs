//! Kv Server
use crate::errors::{Result};
use crate::kv_engine::KvsEngine;
use crate::thread_pool::ThreadPool;
use crate::{KvRequest, KvResponse};
use slog::Logger;
use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::fmt::Display;

/// The Kv Server
pub struct KvServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    pool: P,
    logger: Logger,
}

impl<E: KvsEngine, P: ThreadPool> KvServer<E, P> {
    /// Create new server
    pub fn new(engine: E, pool: P, logger: Logger) -> Self {
        KvServer {
            engine,
            pool,
            logger,
        }
    }

    /// Start the server and listen on given address
    pub fn run<A: Display +  ToSocketAddrs>(self, addr: A) -> Result<()> {
        debug!(&self.logger, "Listening on {}", &addr);
        let listener = TcpListener::bind(addr)?;
        for client in listener.incoming() {
            match client {
                Ok(stream) => {
                    let engine = self.engine.clone();
                    self.pool.spawn(move || {
                        let _ = handle_client(engine, stream.try_clone().unwrap());
                    });
                }
                Err(_) => {
                    println!("Failed to process stream");
                }
            }
        }

        Ok(())
    }

}

fn handle_client<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let writer = BufWriter::new(&stream);
    let mut request_stream =
        serde_json::Deserializer::from_reader(reader).into_iter::<KvRequest>();
    let req = request_stream.next();

    let res = req.map(|body| match body {
        Ok(r) => match r {
            KvRequest::Get(k) => engine
                .get(k)
                .map(KvResponse::Success)
                .unwrap_or_else(|e| KvResponse::Error(e.description().to_string())),
            KvRequest::Set(k, v) => engine
                .set(k, v)
                .map(|_| KvResponse::Success(None))
                .unwrap_or_else(|e| KvResponse::Error(e.description().to_string())),
            KvRequest::Rm(k) => engine
                .remove(k)
                .map(|_| KvResponse::Success(None))
                .unwrap_or_else(|e| KvResponse::Error(e.description().to_string())),
        },
        Err(e) => KvResponse::Error(e.description().to_string()),
    }).unwrap_or_else(|| KvResponse::Error("Unable to parse request".to_string()));

    serde_json::to_writer(writer, &res)?;

    Ok(())
}
