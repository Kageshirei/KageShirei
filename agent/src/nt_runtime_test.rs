#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::format;
    use alloc::sync::Arc;
    use mod_nt_runtime::channel::channel;
    use mod_nt_runtime::nt_threadpool::{task_type_a, task_type_b};
    use mod_nt_runtime::NoStdRuntime;
    use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
    use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
    use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
    use rs2_communication_protocol::metadata::Metadata;
    use rs2_runtime::Runtime;
    use spin::Mutex;

    #[test]
    fn custom_runtime_test() {
        let runtime = Arc::new(NoStdRuntime::new(16));

        // Create a channel to receive results from tasks.
        let (result_tx, result_rx) = channel();

        // Spawn a separate task to handle and print the results.
        let result_handler = runtime.clone();
        result_handler.spawn(move || {
            let mut i = 0;
            while let Some(result) = result_rx.recv() {
                // This would typically print out the result; adapted to a no_std environment, this might involve storing or processing the result differently.
                // In a real no_std environment, you might store these in a buffer or use another mechanism instead of `println!`.
                // For simplicity, we are using a dummy function to simulate the result processing.
                dummy_result_processing(i, result);
                i += 1;
            }
        });

        // Spawn 100 tasks using the NoStdRuntime.
        for i in 0..100 {
            // Generate metadata for each task
            let metadata = Metadata {
                request_id: format!("req-{}", i),
                command_id: format!("cmd-{}", i),
                agent_id: "agent-1234".to_string(),
                path: None,
            };

            let command = if i % 2 == 0 {
                SimpleAgentCommand {
                    op: AgentCommands::Test,
                    metadata,
                }
            } else {
                SimpleAgentCommand {
                    op: AgentCommands::Checkin,
                    metadata,
                }
            };

            let result_tx = result_tx.clone();
            let runtime_clone = runtime.clone();

            runtime_clone.spawn(move || {
                let result = match command.op {
                    AgentCommands::Terminate => task_type_a(command.metadata),
                    AgentCommands::Checkin => task_type_a(command.metadata),
                    AgentCommands::Test => task_type_b(command.metadata),
                };
                result_tx.send(result).unwrap();
            });
        }

        // Close the sender channel to signal that no more results will be sent.
        drop(result_tx);

        // Wait for the result handler to finish processing all results.
        runtime.shutdown();
    }

    #[test]
    fn test_block_on() {
        // Create a new NoStdRuntime with a thread pool of 4 workers.
        let runtime = Arc::new(NoStdRuntime::new(4));

        // A simple asynchronous task that returns a value after some work.
        async fn async_task() -> u32 {
            42 // Return the answer to life, the universe, and everything.
        }

        // Use block_on to run the async_task and get the result.
        let result = runtime.block_on(async_task());

        // Assert that the result is as expected.
        assert_eq!(result, 42);

        // Shutdown the runtime to ensure all threads are properly terminated.
        runtime.shutdown();
    }

    // Dummy function to simulate processing results in a no_std environment
    fn dummy_result_processing(_index: usize, _result: TaskOutput) {
        // Here you could add code to store the results in a buffer or handle them appropriately
    }
}
