use std::sync::Arc;
use std::thread;

use rs2_runtime::Runtime;

#[cfg(feature = "std-runtime")]
use std::sync::mpsc;

use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;

use crate::command::{exit_command, task_type_a, task_type_b};

// Task structure holds the command and a sender to return the result.
#[cfg(feature = "std-runtime")]
pub struct Task {
    command: SimpleAgentCommand,
    response: mpsc::Sender<TaskOutput>,
}

#[cfg(feature = "std-runtime")]
#[derive(Clone)]
pub struct TaskSpawnerCustomRuntime<R: Runtime> {
    runtime: Arc<R>,
    spawn: mpsc::Sender<Task>, // Channel to send tasks to be processed.
}

#[cfg(feature = "std-runtime")]
impl<R: Runtime> TaskSpawnerCustomRuntime<R> {
    /// Creates a new TaskSpawnerCustomRuntime with a given runtime.
    ///
    /// # Arguments
    ///
    /// * `runtime` - An `Arc<R>` used to spawn tasks.
    ///
    /// # Returns
    ///
    /// * A new instance of TaskSpawnerCustomRuntime.
    pub fn new(runtime: Arc<R>) -> TaskSpawnerCustomRuntime<R> {
        let (send, recv) = mpsc::channel::<Task>(); // Channel to queue tasks for processing.

        // Clone the runtime to move it into the thread.
        let runtime_clone = runtime.clone();

        // Spawn a background thread to receive and handle tasks.
        thread::spawn(move || {
            while let Ok(task) = recv.recv() {
                runtime_clone.spawn(move || handle_task(task));
            }
        });

        TaskSpawnerCustomRuntime {
            runtime,
            spawn: send,
        }
    }

    /// Method to spawn a new task, returning a receiver to get the task's result.
    ///
    /// # Arguments
    ///
    /// * `command` - The SimpleAgentCommand containing the operation and metadata for the task.
    ///
    /// # Returns
    ///
    /// * A `mpsc::Receiver<TaskOutput>` that can be awaited to get the result of the task.
    pub fn spawn_task(&self, command: SimpleAgentCommand) -> mpsc::Receiver<TaskOutput> {
        let (response_tx, response_rx) = mpsc::channel(); // Channel to receive the task result.
        let task = Task {
            command,
            response: response_tx,
        };
        if let Err(_) = self.spawn.send(task) {
            panic!("The shared runtime has shut down."); // Handle case where the runtime shuts down unexpectedly.
        }
        response_rx // Return the receiver to get the task result.
    }
}

#[cfg(feature = "std-runtime")]
// Function to handle a task based on its `AgentCommands` variant.
fn handle_task(task: Task) {
    let result = match task.command.op {
        AgentCommands::Terminate => {
            // Create a default TaskOutput
            let mut output = TaskOutput::new();
            output.with_metadata(task.command.metadata.clone());
            output.output = Some("Process will terminate".to_string());

            // Return this output before exiting
            output
        }
        AgentCommands::Checkin => task_type_a(task.command.metadata),
        AgentCommands::Test => task_type_b(task.command.metadata),
        // Add more commands here if needed
    };

    let _ = task.response.send(result); // Send the result back to the sender.

    if let AgentCommands::Terminate = task.command.op {
        exit_command(1); // Perform the exit after sending the response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mod_std_runtime::CustomRuntime;
    use rs2_communication_protocol::metadata::Metadata;
    use std::sync::Arc;

    #[cfg(feature = "std-runtime")]
    #[test]
    fn sync_async_test_custom_runtime() {
        // Create a new CustomRuntime with 16 worker threads.
        let rt = Arc::new(CustomRuntime::new(16));

        let spawner = TaskSpawnerCustomRuntime::new(rt.clone()); // Create a TaskSpawner using the runtime.
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>(); // Channel for results.

        // Spawn a separate thread to handle and print the results.
        let result_handler = thread::spawn(move || {
            let mut i = 0;
            for result in result_rx {
                println!("Result {}: {:?}", i, result);
                i += 1;
            }
        });

        // Spawn 100 tasks with the logic for naming based on whether the index is even or odd.
        for i in 0..40 {
            // Generate metadata for each task
            let metadata = Metadata {
                request_id: format!("req-{}", i),
                command_id: format!("cmd-{}", i),
                agent_id: "agent-1234".to_string(),
                path: None,
            };

            // Assign "Long Task" name if the index is even, otherwise assign a numbered "Test Task".
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
            let spawner_clone = spawner.clone();
            let runtime_clone = rt.clone(); // Clona la runtime

            // Usa il runtime per eseguire ogni task
            runtime_clone.clone().spawn(move || {
                // Spawn la task usando lo spawner
                let receiver = spawner_clone.spawn_task(command);

                // Qui creiamo una nuova task per gestire il risultato senza attendere
                let result_tx_clone = result_tx.clone();
                runtime_clone.spawn(move || {
                    if let Ok(result) = receiver.recv() {
                        result_tx_clone.send(result).unwrap(); // Inviamo il risultato tramite il canale
                    }
                });
            });
        }

        // Drop the result transmitter to signal that no more results will be sent.
        drop(result_tx);

        // Wait for the result handler to finish processing all results.
        result_handler.join().unwrap();

        // Shutdown the runtime to ensure all threads are properly terminated.
        rt.shutdown();
    }
}
