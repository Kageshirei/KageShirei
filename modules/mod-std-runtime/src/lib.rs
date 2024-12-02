//! # Standard Runtime: A Custom Runtime and Thread Pool for Multithreading
//!
//! This crate provides a runtime and thread pool designed to work in standard environments,
//! leveraging Rust's standard library for task execution and management. It is ideal for
//! applications requiring custom concurrency primitives and thread pooling.
//!
//! ## Features
//! - **Custom Runtime (`StdRuntime`)**:
//!   - Implements the `Runtime` trait for managing tasks and asynchronous operations.
//!   - Supports task scheduling, spawning, and `block_on` for running asynchronous tasks.
//! - **Thread Pool (`ThreadPool`)**:
//!   - Manages a pool of worker threads for concurrent task execution.
//!   - Uses an MPSC (multiple-producer, single-consumer) channel for job distribution.
//!
//! ## Modules
//! - [`std_runtime`]: Provides the `StdRuntime` for task management.
//! - [`std_threadpool`]: Implements the thread pool for handling multiple worker threads.
//!
//! ## Examples
//!
//! ### Using the Custom Runtime
//! ```rust ignore
//! use std::sync::Arc;
//!
//! use mod_std_runtime::StdRuntime;
//!
//! // Create a runtime with 4 worker threads
//! let runtime = Arc::new(StdRuntime::new(4));
//!
//! // Define an asynchronous task
//! async fn async_task() -> u32 {
//!     42 // Return the answer to life, the universe, and everything.
//! }
//!
//! // Run the async task and block until completion
//! let result = runtime.block_on(async_task());
//! assert_eq!(result, 42);
//!
//! // Shut down the runtime
//! Arc::try_unwrap(runtime).unwrap().shutdown();
//! ```
//!
//! ### Using the Thread Pool
//! ```rust ignore
//! use mod_std_runtime::std_threadpool::ThreadPool;
//!
//! // Create a thread pool with 4 worker threads
//! let pool = ThreadPool::new(4);
//!
//! // Submit tasks to the thread pool
//! for i in 0 .. 10 {
//!     pool.execute(move || {
//!         println!("Task {} is running", i);
//!     });
//! }
//!
//! // Shut down the thread pool
//! pool.shutdown();
//! ```
//!
//! ## Safety and Usage
//! This crate leverages the standard library and avoids unsafe code in most of its implementation.
//! However, it interacts with multithreading primitives, so the following precautions should be
//! observed:
//! - Ensure proper synchronization when sharing data between threads to avoid data races.
//! - Avoid deadlocks by carefully managing locks and resource contention.
//!
//! ## Testing
//! The crate includes comprehensive tests to validate the runtime and thread pool functionalities.
pub mod std_runtime;
pub mod std_threadpool;

pub use std_runtime::StdRuntime;

#[cfg(test)]
mod tests {
    use std::{
        sync::{mpsc, Arc},
        thread,
        time::Duration,
    };

    use kageshirei_communication_protocol::{
        communication::{AgentCommands, SimpleAgentCommand, TaskOutput},
        Metadata,
    };
    use kageshirei_runtime::Runtime as _; // Import the Runtime trait

    use crate::std_runtime::StdRuntime;

    #[test]
    fn custom_runtime_test() {
        // Create a new CustomRuntime with a thread pool of 16 workers.
        let runtime = Arc::new(StdRuntime::new(4));

        // Create a channel to receive results from tasks.
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>();

        // Spawn a separate thread to handle and print the results.
        let result_handler = thread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                println!("Result {}: {:?}", i, result.output.unwrap());
                i += 1;
            }
        });

        // Spawn 100 tasks using the CustomRuntime.
        for i in 0 .. 100 {
            // Generate metadata for each task
            let metadata = Metadata {
                request_id: format!("req-{}", i),
                command_id: format!("cmd-{}", i),
                agent_id:   "agent-1234".to_owned(),
                path:       None,
            };

            let command = if i % 2 == 0 {
                SimpleAgentCommand {
                    op: AgentCommands::INVALID,
                    metadata,
                }
            }
            else {
                SimpleAgentCommand {
                    op: AgentCommands::Checkin,
                    metadata,
                }
            };

            let result_tx = result_tx.clone();

            let runtime_clone = Arc::clone(&runtime);
            runtime_clone.spawn(move || {
                let result = match command.op {
                    AgentCommands::Terminate => task_type_a(),
                    AgentCommands::Checkin => task_type_a(),
                    AgentCommands::INVALID => task_type_b(),
                };
                result_tx.send(result).unwrap();
            });
        }

        // Close the sender channel to signal that no more results will be sent.
        drop(result_tx);

        // Wait for the result handler to finish processing all results.
        result_handler.join().unwrap();

        // Shutdown the runtime to ensure all threads are properly terminated.
        Arc::try_unwrap(runtime).unwrap().shutdown();
    }

    #[test]
    fn test_block_on() {
        // Create a new CustomRuntime with a thread pool of 4 workers.
        let runtime = Arc::new(StdRuntime::new(4));

        // A simple asynchronous task that returns a value after some work.
        async fn async_task() -> u32 {
            42 // Return the answer to life, the universe, and everything.
        }

        // Use block_on to run the async_task and get the result.
        let result = runtime.block_on(async_task());

        // Assert that the result is as expected.
        assert_eq!(result, 42);

        // Shutdown the runtime to ensure all threads are properly terminated.
        Arc::try_unwrap(runtime).unwrap().shutdown();
    }

    // Simulated asynchronous task that takes 2 seconds to complete.
    pub fn task_type_a() -> TaskOutput {
        let mut output = TaskOutput::new();
        output.output = Some("Result from task type A".to_owned());
        output
    }

    // Simulated asynchronous task that takes 3 seconds to complete.
    pub fn task_type_b() -> TaskOutput {
        thread::sleep(Duration::from_secs(3)); // Simulate some work
        let mut output = TaskOutput::new();
        output.output = Some("Result from task type B".to_owned());
        output
    }
}
