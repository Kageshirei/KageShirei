use alloc::{boxed::Box, sync::Arc, vec::Vec};

use mod_nostd::{nostd_mpsc, nostd_thread};
use nostd_mpsc::{Receiver, Sender};
use spin::Mutex;

/// The `NoStdThreadPool` struct manages a pool of worker threads that execute jobs.
/// It provides a simple implementation of a thread pool that uses a custom MPSC
/// (multiple-producer, single-consumer) channel for job distribution among workers.
pub struct NoStdThreadPool {
    workers: Vec<Worker>,                     // Vector holding the worker threads in the pool.
    sender: Option<Arc<Mutex<Sender<Job>>>>, // Channel sender used to dispatch jobs to the workers.
}

/// Type alias for a job, which is represented as a boxed closure. The closure takes no arguments,
/// returns nothing, and must implement `Send` and `'static`, allowing it to be safely sent across threads.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl NoStdThreadPool {
    /// Creates a new `NoStdThreadPool` with a specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads to spawn in the pool.
    ///
    /// # Returns
    ///
    /// * A new `NoStdThreadPool` instance with the specified number of workers.
    pub fn new(size: usize) -> NoStdThreadPool {
        assert!(size > 0); // Ensure that the size of the pool is greater than 0.

        // Create a custom MPSC channel for sending jobs to workers.
        // `sender` is used to send jobs, and `receiver` is used by workers to receive jobs.
        let (sender, receiver) = nostd_mpsc::channel();
        let sender = Arc::new(Mutex::new(sender)); // Wrap the sender in `Arc<Mutex<>>` for safe sharing across threads.
        let receiver = Arc::new(Mutex::new(receiver)); // Wrap the receiver similarly for thread-safe access.

        // Create a vector to hold the worker threads, with an initial capacity equal to the pool size.
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            // Create each worker and push it into the vector.
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        // Return a new `NoStdThreadPool` instance containing the worker threads and the sender channel.
        NoStdThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Executes a job by sending it to one of the worker threads via the sender channel.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure representing the job to be executed. The closure must implement `FnOnce`, `Send`, and
    ///   `'static` to be safely executed across threads.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let job = Box::new(f); // Box the job (closure) to make it a heap-allocated trait object.
            sender.lock().send(job).unwrap(); // Send the job to the workers via the channel.
        }
    }

    /// Gracefully shuts down the thread pool by dropping the sender and joining all worker threads.
    /// This ensures that all worker threads have completed their jobs before the pool is terminated.
    pub fn shutdown(&mut self) {
        // Drop the sender to close the channel and signal that no more jobs will be sent.
        drop(self.sender.take());

        // Wait for each worker thread to finish executing its current job.
        for worker in &mut self.workers {
            worker.join(); // Join each worker thread to ensure it has completed its work.
        }
    }
}

/// The `Worker` struct represents a single thread in the thread pool.
/// Each worker continuously listens for jobs on the receiver channel and executes them when available.
#[derive(Debug)]
struct Worker {
    handle: Option<nostd_thread::NoStdThread>, // Handle to the thread, allowing it to be joined later.
}

impl Worker {
    /// Creates a new worker thread that listens for jobs from the receiver channel.
    ///
    /// # Arguments
    ///
    /// * `receiver` - An `Arc<Mutex<Receiver<Job>>>` from which the worker receives jobs.
    ///
    /// # Returns
    ///
    /// * A `Worker` instance wrapping the thread handle.
    fn new(receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handle = nostd_thread::NoStdThread::spawn(move || {
            loop {
                // Lock the receiver to safely receive a job. If the channel is closed, break the loop and stop the
                // worker.
                let job = receiver.lock().recv();

                match job {
                    Some(job) => {
                        job(); // Execute the received job.
                    },
                    None => {
                        break; // Exit the loop if the channel is closed (no more jobs to process).
                    },
                }
            }
        });

        Worker {
            handle: Some(handle), // Store the thread handle for later joining.
        }
    }

    /// Joins the worker thread, blocking until the thread completes its execution.
    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap(); // Join the thread and ensure it has completed its work.
        }
    }
}
