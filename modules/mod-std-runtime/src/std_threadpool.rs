use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// The `ThreadPool` struct manages a pool of worker threads that execute jobs.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,                           // Vector of workers (threads) in the pool.
    sender:  Option<Arc<Mutex<mpsc::Sender<Job>>>>, // Sender channel to dispatch jobs to the workers.
}

/// Type alias for a job, which is a boxed closure that takes no arguments, returns nothing, and
/// must be `Send` and `'static`.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new `ThreadPool` with a specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads to spawn in the pool.
    ///
    /// # Returns
    ///
    /// * A new `ThreadPool` instance with the specified number of workers.
    pub fn new(size: usize) -> Self {
        assert!(size > 0); // Ensure the size of the pool is greater than 0.

        // Create a channel for sending jobs to workers. `sender` is used to send jobs,
        // and `receiver` is used by workers to receive jobs.
        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender)); // Wrap the sender in Arc<Mutex<>>.
        let receiver = Arc::new(Mutex::new(receiver)); // Arc and Mutex protect the receiver so it can be safely shared among multiple threads.

        let mut workers = Vec::with_capacity(size); // Create a vector with the capacity to hold all workers.
        for _ in 0 .. size {
            workers.push(Worker::new(Arc::clone(&receiver))); // Create and push each worker to the
                                                              // workers vector.
        }

        // Return a new ThreadPool with the specified workers and sender channel.
        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Method to execute a job on the thread pool. The job is sent to the worker threads via the
    /// sender channel.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure representing the job to be executed.
    ///
    /// The closure must be `Send`, `FnOnce`, and `'static` to be safely executed across threads.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let job = Box::new(f); // Box the job (closure) to make it a heap-allocated trait object.
            sender.lock().unwrap().send(job).unwrap(); // Send the job to the workers via the
                                                       // channel.
        }
    }

    /// Gracefully shuts down the thread pool by dropping the sender and joining all worker threads.
    pub fn shutdown(&mut self) {
        // Drop the sender to close the channel and signal no more jobs will be sent.
        drop(self.sender.take());

        // Wait for each worker thread to finish executing its current job.
        for worker in &mut self.workers {
            worker.join(); // Use a mutable reference to call join.
        }
    }
}

/// The `Worker` struct represents a single thread in the thread pool.
#[derive(Debug)]
struct Worker {
    handle: Option<thread::JoinHandle<()>>, // Handle to the thread, allowing it to be joined later.
}

impl Worker {
    /// Creates a new worker thread that listens for jobs from the receiver channel.
    ///
    /// # Arguments
    ///
    /// * `receiver` - An `Arc<Mutex<mpsc::Receiver<Job>>>` from which the worker receives jobs.
    ///
    /// # Returns
    ///
    /// * A `Worker` instance wrapping the thread handle.
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                // Lock the receiver to safely receive a job. If the channel is closed, break the loop and stop the
                // worker.
                let job = receiver.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        job(); // Execute the received job.
                    },
                    Err(_) => {
                        break; // Exit the loop if the channel is closed (no more jobs to process).
                    },
                }
            }
        });

        Self {
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
