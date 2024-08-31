#[cfg(test)]
mod tests {
    extern crate alloc;

    use std::sync::mpsc;

    use alloc::sync::Arc;
    use mod_nostd_runtime::nostd_channel::{channel, Receiver, Sender};
    use mod_nostd_runtime::nostd_runtime::NoStdRuntime;
    use mod_nostd_runtime::nostd_threadpool::{task_type_a, task_type_b};
    use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
    use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
    use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
    use rs2_communication_protocol::metadata::Metadata;
    use rs2_runtime::Runtime;
    use spin::Mutex;

    fn custom_runtime_test() {
        let results = Arc::new(Mutex::new(vec![None; 10]));

        let runtime = Arc::new(NoStdRuntime::new(16));

        for i in 0..10 {
            let metadata = Metadata {
                request_id: alloc::format!("req-{}", i),
                command_id: alloc::format!("cmd-{}", i),
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

            let runtime_clone = Arc::clone(&runtime);
            let results_clone = Arc::clone(&results);

            runtime_clone.spawn(move || {
                let result = match command.op {
                    AgentCommands::Terminate => task_type_a(command.metadata),
                    AgentCommands::Checkin => task_type_a(command.metadata),
                    AgentCommands::Test => task_type_b(command.metadata),
                };

                // Blocchiamo il Mutex per inserire il risultato nel giusto slot.
                let mut results = results_clone.lock();
                results[i] = Some(result);
            });
        }

        // Simuliamo l'esecuzione di tutti i task
        // runtime.execute_all(); // Eseguiamo tutti i task

        // Simuliamo l'elaborazione dei risultati dopo che tutti i task sono stati completati.
        let results = results.lock(); // Blocchiamo il Mutex per accedere ai risultati.
        for (i, result) in results.iter().enumerate() {
            if let Some(output) = result {
                println!("Result {}: {:?}", i, output);
            } else {
                println!("Result {} is None", i); // Aiuto per il debug
            }
        }

        // runtime.shutdown();
    }

    #[test]
    fn custom_runtime_test_channel() {
        // let results = Arc::new(Mutex::new(vec![None; 10]));
        let runtime = Arc::new(NoStdRuntime::new(16));

        let (result_tx, result_rx) = channel::<TaskOutput>();
        // let (result_tx, result_rx) = mpsc::channel();

        // Spawn a separate thread to handle and print the results.
        let result_handler = runtime.clone();
        result_handler.spawn(move || {
            let mut i = 0;
            for result in result_rx {
                println!("Result {}: {:?}", i, result);
                i += 1;
            }
        });

        for i in 0..10 {
            let metadata = Metadata {
                request_id: alloc::format!("req-{}", i),
                command_id: alloc::format!("cmd-{}", i),
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
            let runtime_clone = Arc::clone(&runtime);

            runtime_clone.spawn(move || {
                let result = match command.op {
                    AgentCommands::Terminate => task_type_a(command.metadata),
                    AgentCommands::Checkin => task_type_a(command.metadata),
                    AgentCommands::Test => task_type_b(command.metadata),
                };
                result_tx.send(result).unwrap();

                // Blocchiamo il Mutex per inserire il risultato nel giusto slot.
                // let mut results = results_clone.lock();
                // results[i] = Some(result);
            });
        }

        // Close the sender channel to signal that no more results will be sent.
        drop(result_tx);

        // Wait for the result handler to finish processing all results.
        // result_handler.join().unwrap();

        // Shutdown the runtime to ensure all threads are properly terminated.
        runtime.shutdown();
    }

    // Dummy function to simulate processing results in a no_std environment
    fn dummy_result_processing(_index: usize, _result: TaskOutput) {
        // Here you could add code to store the results in a buffer or handle them appropriately
    }
}
