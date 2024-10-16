#![no_std]

pub mod nostd_nt_runtime;
pub mod nostd_nt_threadpool;

pub use nostd_nt_runtime::NoStdNtRuntime;

extern crate alloc;

#[cfg(test)]
mod tests {
    use alloc::{format, string::ToString, sync::Arc};

    use kageshirei_communication_protocol::{
        communication::{
            agent_commands::AgentCommands,
            simple_agent_command::SimpleAgentCommand,
            task_output::TaskOutput,
        },
        metadata::Metadata,
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
        let (result_tx, result_rx) = nostd_mpsc::channel();

        // Spawn a separate thread to handle and print the results as they are received.
        // This thread will listen on the receiver end of the channel and print each result.
        let result_handler = nostd_thread::NoStdThread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                libc_println!("Result {}: {:?}", i, result);
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
                    AgentCommands::Terminate => task_type_a(command.metadata),
                    AgentCommands::Checkin => task_type_a(command.metadata),
                    AgentCommands::INVALID => task_type_b(command.metadata),
                };
                result_tx.send(result).unwrap();
            });
        }

        // Close the sender channel to signal that no more results will be sent.
        // This is necessary to allow the result handler thread to exit its loop and finish.
        drop(result_tx);

        // Wait for the result handler thread to finish processing all results.
        // This ensures that all tasks have completed and their results have been printed.
        result_handler.join().unwrap();

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

    pub fn task_type_a(metadata: Metadata) -> TaskOutput {
        let mut output = TaskOutput::new();
        output.with_metadata(metadata);
        output.output = Some("Result from task type A".to_string());
        output
    }

    pub fn task_type_b(metadata: Metadata) -> TaskOutput {
        delay(3);
        let mut output = TaskOutput::new();
        output.with_metadata(metadata);
        output.output = Some("Result from task type B".to_string());
        output
    }
}
