use crate::nostd_channel::{channel, Receiver, Sender};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use mod_win32::nt_time::delay;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
use rs2_communication_protocol::metadata::Metadata;
use spin::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>,        // Vector of workers handling tasks
    sender: Option<Sender<Job>>, // Sender channel to dispatch jobs to the workers
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new `ThreadPool` with a specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads in the pool.
    ///
    /// # Returns
    ///
    /// * A new `ThreadPool` instance with the specified number of workers.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Create a custom channel for sending jobs to workers.
        let (sender, receiver) = channel::<Job>();
        let mut workers = Vec::with_capacity(size);

        // Create and start worker threads.
        for _ in 0..size {
            workers.push(Worker::new(receiver.clone()));
        }

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
            let job = Box::new(f);
            sender.send(job).unwrap(); // Send the job to the workers via the channel.
        }
    }

    pub fn run_worker(&mut self) {
        for worker in &mut self.workers {
            worker.join(); // Use a mutable reference to call join.
        }
    }

    /// Gracefully shuts down the thread pool by dropping the sender and joining all worker threads.
    pub fn shutdown(&mut self) {
        drop(self.sender.take()); // Drop the sender to signal no more jobs will be sent.

        // Wait for each worker thread to finish executing its current job.
        for worker in &mut self.workers {
            worker.join(); // Use a mutable reference to call join.
        }
    }
}

struct Worker {
    receiver: Receiver<Job>,
}

impl Worker {
    fn new(receiver: Receiver<Job>) -> Worker {
        Worker { receiver }
    }

    fn join(&mut self) {
        while let Some(job) = self.receiver.recv() {
            job();
        }
    }
}

// struct Worker {
//     handle: Option<WorkerHandle>,
// }

// impl Worker {
//     /// Creates a new worker thread that listens for jobs from the receiver channel.
//     ///
//     /// # Arguments
//     ///
//     /// * `receiver` - A `Receiver<Job>` from which the worker receives jobs.
//     ///
//     /// # Returns
//     ///
//     /// * A `Worker` instance wrapping the worker handle.
//     fn new(receiver: Receiver<Job>) -> Worker {
//         let handle = WorkerHandle::start(receiver);
//         Worker {
//             handle: Some(handle),
//         }
//     }

//     /// Joins the worker thread, blocking until the thread completes its execution.
//     fn join(&mut self) {
//         if let Some(handle) = self.handle.take() {
//             handle.join();
//         }
//     }
// }

// struct WorkerHandle {
//     receiver: Receiver<Job>,
// }

// impl WorkerHandle {
//     fn start(receiver: Receiver<Job>) -> Self {
//         let receiver = receiver.clone();
//         let handle = WorkerHandle { receiver };
//         handle
//     }

//     fn join(self) {
//         while let Some(job) = self.receiver.recv() {
//             job();
//         }
//     }
// }

// Simulated asynchronous task that takes 2 seconds to complete.
pub fn task_type_a(metadata: Metadata) -> TaskOutput {
    delay(1); // Simulate some work
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type A".to_string());
    output
}

// Simulated asynchronous task that takes 3 seconds to complete.
pub fn task_type_b(metadata: Metadata) -> TaskOutput {
    delay(3); // Simulate some work
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type B".to_string());
    output
}
