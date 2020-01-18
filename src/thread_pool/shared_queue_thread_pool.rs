use std::io::{self};

use crate::thread_pool::ThreadPool;

/// A shared queue thread pool
pub struct SharedQueueThreadPool {
    
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(num_threads: usize) -> io::Result<Self> {
        unimplemented!();
    }

    /// Spawn a task on a running thread
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static {
        unimplemented!();
    }
}
