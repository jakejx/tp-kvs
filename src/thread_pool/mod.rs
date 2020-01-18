//! The thread_pool module for KVS

use std::io::{self};

mod naive_thread_pool;
mod shared_queue_thread_pool;
mod rayon_thread_pool;

pub use self::naive_thread_pool::NaiveThreadPool;
pub use self::shared_queue_thread_pool::SharedQueueThreadPool;
pub use self::rayon_thread_pool::RayonThreadPool;

/// ThreadPool trait
pub trait ThreadPool {
  /// Creates a new threadpool
  fn new(num_threads: usize) -> io::Result<Self>
  where
    Self: Sized;

  /// Spawn a task on a running thread
  fn spawn<F>(&self, job: F)
  where
    F: FnOnce() + Send + 'static;
}
