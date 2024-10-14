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
        communication_structs::{
            agent_commands::AgentCommands,
            simple_agent_command::SimpleAgentCommand,
            task_output::TaskOutput,
        },
        metadata::Metadata,
    };
    use kageshirei_runtime::Runtime; // Import the Runtime trait

    use crate::std_runtime::StdRuntime;

    #[test]
    fn custom_runtime_test() {
        // Create a new CustomRuntime with a thread pool of 16 workers.
        let runtime = Arc::new(StdRuntime::new(4));

        // Create a channel to receive results from tasks.
        let (result_tx, result_rx) = mpsc::channel();

        // Spawn a separate thread to handle and print the results.
        let result_handler = thread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                println!("Result {}: {:?}", i, result);
                i += 1;
            }
        });

        // Spawn 100 tasks using the CustomRuntime.
        for i in 0 .. 100 {
            // Generate metadata for each task
            let metadata = Metadata {
                request_id: format!("req-{}", i),
                command_id: format!("cmd-{}", i),
                agent_id:   "agent-1234".to_string(),
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
                    AgentCommands::Terminate => task_type_a(command.metadata),
                    AgentCommands::Checkin => task_type_a(command.metadata),
                    AgentCommands::INVALID => task_type_b(command.metadata),
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
    pub fn task_type_a(metadata: Metadata) -> TaskOutput {
        let mut output = TaskOutput::new();
        output.with_metadata(metadata);
        output.output = Some("Result from task type A".to_string());
        output
    }

    // Simulated asynchronous task that takes 3 seconds to complete.
    pub fn task_type_b(metadata: Metadata) -> TaskOutput {
        thread::sleep(Duration::from_secs(3)); // Simulate some work
        let mut output = TaskOutput::new();
        output.with_metadata(metadata);
        output.output = Some("Result from task type B".to_string());
        output
    }
}
