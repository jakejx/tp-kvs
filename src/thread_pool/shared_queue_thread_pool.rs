use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{self};
use std::sync::mpsc::{self, channel};
use std::thread::{self, ThreadId};
use std::panic::{ self, AssertUnwindSafe };

use crate::thread_pool::ThreadPool;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum ThreadPoolMessage {
    RunJob(Job),
    JobComplete(ThreadId),
    Shutdown,
}

struct PublisherThread {
    threads: HashMap<ThreadId, mpsc::Sender<ThreadPoolMessage>>,
    queue: VecDeque<ThreadId>,
    ids: Vec<ThreadId>,
}

/// A shared queue thread pool
pub struct SharedQueueThreadPool {
    sender: mpsc::Sender<ThreadPoolMessage>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(num_threads: usize) -> io::Result<Self> {
        let mut threads = HashMap::new();
        let mut queue = VecDeque::new();
        let mut ids = Vec::new();

        let (dispatch_tx, dispatch_rx) = channel::<ThreadPoolMessage>();

        // spawn worker threads
        for _ in 0..num_threads {
            let (tx, rx) = channel::<ThreadPoolMessage>();
            let sender_clone = dispatch_tx.clone();
            let handle = thread::spawn(move || {
                for job in rx {
                    match job {
                        ThreadPoolMessage::RunJob(job) => {
                            let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                                job();
                            }));
                            let _ = sender_clone
                                .send(ThreadPoolMessage::JobComplete(thread::current().id()));
                        }
                        ThreadPoolMessage::Shutdown => {
                            break;
                        }
                        _ => {}
                    }
                }
            });

            let thread_id = handle.thread().id();
            threads.insert(thread_id, tx);
            queue.push_back(thread_id);
            ids.push(thread_id);
        }

        let retry_sender = dispatch_tx.clone();
        // spawn dispatcher thread
        thread::spawn(move || {
            let mut publisher = PublisherThread { threads, queue, ids };

            for message in dispatch_rx {
                match message {
                    m @ ThreadPoolMessage::RunJob(_) => {
                        if publisher.queue.len() == 0 {
                            let _ = retry_sender.send(m);
                        } else {
                            let thread_id = publisher.queue.pop_front().unwrap();
                            let job_sender = &publisher.threads[&thread_id];
                            let _ = job_sender.send(m);
                        }
                    }
                    ThreadPoolMessage::JobComplete(id) => publisher.queue.push_back(id),
                    // shutdown all worker threads
                    ThreadPoolMessage::Shutdown => {
                        for id in &publisher.ids {
                            let sender = &publisher.threads[&id];
                            let _ = sender.send(ThreadPoolMessage::Shutdown);
                        }
                    },
                }
            }
        });

        Ok(SharedQueueThreadPool {
            sender: dispatch_tx,
        })
    }

    /// Spawn a task on a running thread
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.sender.send(ThreadPoolMessage::RunJob(Box::new(job)));
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        self.sender.send(ThreadPoolMessage::Shutdown);
    }
}
