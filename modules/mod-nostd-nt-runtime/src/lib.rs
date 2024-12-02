#![no_std]
//! # No-Std NT Runtime: Lightweight Runtime and Thread Pool for `no_std`
//!
//! This crate provides a custom runtime implementation and a thread pool designed for `no_std`
//! environments. It leverages low-level Windows NT APIs and lightweight abstractions to enable
//! asynchronous task execution and multi-threading in constrained environments.
//!
//! ## Features
//! - **Custom Runtime (`NoStdNtRuntime`)**:
//!   - Implements the `Runtime` trait, providing support for task scheduling, spawning, and
//!     blocking on asynchronous tasks.
//!   - Allows integration with `futures` for asynchronous programming.
//! - **Thread Pool (`NoStdThreadPool`)**:
//!   - Manages a pool of worker threads to execute tasks concurrently.
//!   - Provides a lightweight implementation suitable for environments where standard Rust
//!     threading is unavailable.
//! - **Message Passing (`nostd_mpsc`)**:
//!   - Facilitates inter-thread communication using a multiple-producer, single-consumer (MPSC)
//!     channel.
//!
//! ## Modules
//! - [`nostd_nt_runtime`]: Custom runtime for task scheduling and execution.
//! - [`nostd_nt_threadpool`]: Thread pool implementation for managing worker threads.
//!
//! ## Examples
//!
//! ### Using the Custom Runtime
//! ```rust ignore
//! use std::sync::Arc;
//!
//! use mod_nostd_nt_runtime::NoStdNtRuntime;
//!
//! // Create a runtime with 4 worker threads
//! let runtime = Arc::new(NoStdNtRuntime::new(4));
//!
//! // Define an asynchronous task
//! async fn async_task() -> u32 { 42 }
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
//! use mod_nostd_nt_runtime::nostd_nt_threadpool::NoStdThreadPool;
//!
//! // Create a thread pool with 4 worker threads
//! let pool = NoStdThreadPool::new(4);
//!
//! // Submit tasks to the thread pool
//! for i in 0 .. 10 {
//!     pool.execute(move || {
//!         // Perform some work
//!         println!("Task {} is running", i);
//!     });
//! }
//!
//! // Shut down the thread pool
//! pool.shutdown();
//! ```
//!
//! ## Safety
//! This crate interacts directly with low-level Windows NT APIs and performs unsafe operations such
//! as:
//! - Raw pointer manipulations
//! - Direct system calls to manage threads and synchronization
//!
//! ### Caller Responsibilities
//! - Ensure correct usage of pointers and system resources.
//! - Avoid data races and deadlocks by properly managing concurrency.
//! - Use the provided APIs in contexts where `no_std` is strictly required to minimize complexity.
//!
//! ## Testing
//! The crate includes tests for both runtime and thread pool functionalities.
//! ```

pub mod nostd_nt_runtime;
pub mod nostd_nt_threadpool;

pub use nostd_nt_runtime::NoStdNtRuntime;

extern crate alloc;

#[cfg(test)]
mod tests {
    use alloc::{format, string::ToString, sync::Arc};

    use kageshirei_communication_protocol::{
        communication::{AgentCommands, SimpleAgentCommand, TaskOutput},
        Metadata,
    };
    use kageshirei_runtime::Runtime;
    use libc_print::libc_println;
    use mod_nostd::{nostd_mpsc, nostd_thread};
    use mod_win32::nt_time::delay;

    use crate::nostd_nt_runtime::NoStdNtRuntime; // Import the Runtime trait

    #[test]
    fn custom_runtime_test() {
        // Create a new NoStdNtRuntime with a thread pool of 4 workers.
        // This runtime will be used to manage and execute the tasks in the test.
        let runtime = Arc::new(NoStdNtRuntime::new(4));

        // Create a channel to receive results from tasks.
        // This channel will be used to send results from the tasks back to the main thread.
        let (result_tx, result_rx) = nostd_mpsc::channel::<TaskOutput>();

        // Spawn a separate thread to handle and print the results as they are received.
        // This thread will listen on the receiver end of the channel and print each result.
        let result_handler = nostd_thread::NoStdThread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                libc_println!("Result {}: {:?}", i, result.output.unwrap());
                i += 1;
            }
        });

        // Spawn 10 tasks using the NoStdNtRuntime.
        // Each task will either execute `task_type_a` or `task_type_b` depending on the loop index.
        for i in 0 .. 100 {
            // Generate metadata for each task, which includes a unique request ID and command ID.
            let metadata = Metadata {
                request_id: format!("req-{}", i),
                command_id: format!("cmd-{}", i),
                agent_id:   "agent-1234".to_string(),
                path:       None,
            };

            // Based on the index, alternate between two different commands.
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

            // Clone the sender and runtime to be moved into the task closure.
            let result_tx = result_tx.clone();
            let runtime_clone = Arc::clone(&runtime);

            // Spawn the task using the runtime. The task will execute either `task_type_a` or `task_type_b`
            // and send the result back through the channel.
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
        // This is necessary to allow the result handler thread to exit its loop and finish.
        drop(result_tx);

        // Wait for the result handler thread to finish processing all results.
        // This ensures that all tasks have completed and their results have been printed.
        result_handler.unwrap().join().unwrap();

        // Shutdown the runtime to ensure all threads are properly terminated.
        // This releases any resources held by the runtime and ensures clean exit.
        Arc::try_unwrap(runtime).unwrap().shutdown();
    }

    #[test]
    fn test_block_on() {
        // Create a new NoStdNtRuntime with a thread pool of 4 workers.
        let runtime = Arc::new(NoStdNtRuntime::new(4));

        // Define a simple asynchronous task that returns a value after some work.
        // This task will be used to test the `block_on` functionality of the runtime.
        async fn async_task() -> u32 {
            42 // Return the answer to life, the universe, and everything.
        }

        // Use the runtime's `block_on` method to run the async task and get the result.
        // This method will block the current thread until the async task completes.
        let result = runtime.block_on(async_task());

        // Assert that the result is as expected (42).
        // This verifies that the async task was executed correctly.
        assert_eq!(result, 42);

        // Shutdown the runtime to ensure all threads are properly terminated.
        // This is necessary to clean up resources and prevent leaks.
        Arc::try_unwrap(runtime).unwrap().shutdown();
    }

    pub fn task_type_a() -> TaskOutput {
        let mut output = TaskOutput::new();
        output.output = Some("Result from task type A".to_string());
        output
    }

    pub fn task_type_b() -> TaskOutput {
        delay(3);
        let mut output = TaskOutput::new();
        output.output = Some("Result from task type B".to_string());
        output
    }
}
