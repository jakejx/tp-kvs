use std::io::{self};

use crate::thread_pool::ThreadPool;

/// A shared queue thread pool
pub struct RayonThreadPool {
    
}

impl ThreadPool for RayonThreadPool {
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
