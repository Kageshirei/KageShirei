/// The `mod-std-runtime` module implements the `Runtime` trait using a custom thread pool.
/// This module provides a thread pool-based runtime adapter that conforms to the generic `Runtime` interface.
pub mod std_runtime;
pub mod threadpool;

pub use std_runtime::CustomRuntime;

#[cfg(test)]
mod tests {
    use crate::std_runtime::CustomRuntime;
    use rs2_runtime::Runtime; // Import the Runtime trait
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    fn custom_runtime_test() {
        // Create a new CustomRuntime with a thread pool of 4 workers.
        let runtime = Arc::new(CustomRuntime::new(16));

        // Create a channel to receive results from tasks.
        let (result_tx, result_rx) = mpsc::channel();

        // Spawn a separate thread to handle and print the results.
        let result_handler = thread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                println!("Result {}: {}", i, result);
                i += 1;
            }
        });

        // Spawn 10 tasks using the CustomRuntime.
        for i in 0..100 {
            let task_name = if i % 2 == 0 {
                "Long Task".to_string()
            } else {
                format!("Test Task {}", i)
            };

            let result_tx = result_tx.clone();

            let runtime_clone = Arc::clone(&runtime);
            runtime_clone.spawn(move || {
                let result = if task_name == "Long Task" {
                    super::threadpool::task_type_b()
                } else {
                    super::threadpool::task_type_a()
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
        let runtime = Arc::new(CustomRuntime::new(4));

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
}
