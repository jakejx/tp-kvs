use std::io::{self};
use std::sync::mpsc::{self, channel};
use std::thread::{self, JoinHandle};
use std::cell::Cell;

use crate::thread_pool::ThreadPool;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A naive thread pool
pub struct NaiveThreadPool {
    channels: Vec<mpsc::Sender<Job>>,
    threads: Vec<JoinHandle<()>>,
    current_thread: Cell<usize>,
}

impl ThreadPool for NaiveThreadPool {
    fn new(num_threads: usize) -> io::Result<Self> {
        let mut channels: Vec<_> = Vec::with_capacity(num_threads);
        let mut threads: Vec<_> = Vec::with_capacity(num_threads);
        for i in 0..num_threads {
            let (tx, rx) = channel::<Job>();
            channels.push(tx);

            // add mechanism for thread to notify scheduler of completion
            let builder = thread::Builder::new();
            let handle = builder.spawn(move || {
                for job in rx {
                    println!("Worker #{} got a job", i);
                    job();
                }
            })?;
            threads.push(handle);
        }

        Ok(NaiveThreadPool {
            channels,
            threads,
            current_thread: Cell::new(0),
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let tx = self.channels.get(self.current_thread.get()).unwrap();
        tx.send(Box::new(job));

        self.current_thread.set(( self.current_thread.get() + 1 ) % self.threads.len() );
    }
}
