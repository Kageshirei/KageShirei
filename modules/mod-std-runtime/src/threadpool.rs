use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

/// The `ThreadPool` struct manages a pool of worker threads that execute jobs.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>, // Vector of workers (threads) in the pool.
    sender: Option<Arc<Mutex<mpsc::Sender<Job>>>>, // Sender channel to dispatch jobs to the workers.
}

/// Type alias for a job, which is a boxed closure that takes no arguments, returns nothing, and must be `Send` and `'static`.
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
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0); // Ensure the size of the pool is greater than 0.

        // Create a channel for sending jobs to workers. `sender` is used to send jobs,
        // and `receiver` is used by workers to receive jobs.
        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender)); // Wrap the sender in Arc<Mutex<>>.
        let receiver = Arc::new(Mutex::new(receiver)); // Arc and Mutex protect the receiver so it can be safely shared among multiple threads.

        let mut workers = Vec::with_capacity(size); // Create a vector with the capacity to hold all workers.
        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver))); // Create and push each worker to the workers vector.
        }

        // Return a new ThreadPool with the specified workers and sender channel.
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Method to execute a job on the thread pool. The job is sent to the worker threads via the sender channel.
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
            sender.lock().unwrap().send(job).unwrap(); // Send the job to the workers via the channel.
        }
    }

    /// Gracefully shuts down the thread pool by dropping the sender and joining all worker threads.
    pub fn shutdown(self) {
        drop(self.sender); // Dropping the sender closes the channel, signaling no more jobs will be sent.
        for worker in self.workers {
            worker.join(); // Wait for each worker thread to finish executing its current job.
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
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            // Lock the receiver to safely receive a job. If the channel is closed, break the loop and stop the worker.
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    job(); // Execute the received job.
                }
                Err(_) => {
                    break; // Exit the loop if the channel is closed (no more jobs to process).
                }
            }
        });

        Worker {
            handle: Some(handle), // Store the thread handle for later joining.
        }
    }

    /// Joins the worker thread, blocking until the thread completes its execution.
    fn join(self) {
        if let Some(handle) = self.handle {
            handle.join().unwrap(); // Join the thread and ensure it has completed its work.
        }
    }
}

// Simulated task that takes 1 second to complete.
pub fn task_type_a() -> String {
    thread::sleep(Duration::from_secs(1)); // Simulate work with a sleep of 1 second.
    "Result from task type A".to_string() // Return a result string.
}

// Simulated task that takes 3 seconds to complete.
pub fn task_type_b() -> String {
    thread::sleep(Duration::from_secs(3)); // Simulate work with a sleep of 3 seconds.
    "Result from task type B".to_string() // Return a result string.
}
