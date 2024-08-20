use tokio::runtime::Handle;
use tokio::sync::{mpsc, oneshot};

use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;

use crate::command::{exit_command, task_type_a, task_type_b};

// Task structure holds the command and a sender to return the result.
pub struct Task {
    command: SimpleAgentCommand,
    response: oneshot::Sender<TaskOutput>,
}

#[derive(Clone)]
pub struct TaskSpawner {
    spawn: mpsc::Sender<Task>, // Tokio mpsc channel to send tasks to be processed.
}

impl TaskSpawner {
    /// Creates a new TaskSpawner with a given Tokio runtime handle.
    ///
    /// # Arguments
    ///
    /// * `rt` - A handle to the Tokio runtime used to spawn tasks.
    ///
    /// # Returns
    ///
    /// * A new instance of TaskSpawner.
    pub fn new(rt: Handle) -> TaskSpawner {
        let (send, mut recv) = mpsc::channel(16); // Tokio mpsc channel to queue tasks for processing.

        // Spawn a background task to receive and handle tasks.
        rt.spawn(async move {
            while let Some(task) = recv.recv().await {
                tokio::spawn(handle_task(task)); // Spawn a new async task for each received task.
            }
        });

        TaskSpawner { spawn: send }
    }

    /// Method to spawn a new task, returning a receiver to get the task's result.
    ///
    /// # Arguments
    ///
    /// * `command` - The SimpleAgentCommand containing the operation and metadata for the task.
    ///
    /// # Returns
    ///
    /// * A `oneshot::Receiver<TaskOutput>` that can be awaited to get the result of the task.
    pub async fn spawn_task(&self, command: SimpleAgentCommand) -> oneshot::Receiver<TaskOutput> {
        let (response_tx, response_rx) = oneshot::channel(); // Channel to receive the task result.
        let task = Task {
            command,
            response: response_tx,
        };
        if let Err(_) = self.spawn.send(task).await {
            panic!("The shared runtime has shut down."); // Handle case where the runtime shuts down unexpectedly.
        }
        response_rx // Return the receiver to get the task result.
    }
}

// Function to handle a task based on its `AgentCommands` variant.
async fn handle_task(task: Task) {
    let result = match task.command.op {
        AgentCommands::Terminate => {
            // Create a default TaskOutput
            let mut output = TaskOutput::new();
            output.with_metadata(task.command.metadata.clone());
            output.output = Some("Process will terminate".to_string());

            // Return this output before exiting
            output
        }
        AgentCommands::Checkin => task_type_a(task.command.metadata).await,
        AgentCommands::Test => task_type_b(task.command.metadata).await,
        // Add more commands here if needed
    };

    let _ = task.response.send(result); // Send the result back to the sender.

    if let AgentCommands::Terminate = task.command.op {
        exit_command(1).await; // Perform the async exit after sending the response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs2_communication_protocol::metadata::Metadata;
    use std::sync::Arc;
    use tokio::runtime::Builder;
    use tokio::sync::mpsc;

    #[test]
    fn sync_async_test() {
        // Create a new multi-threaded Tokio runtime with 4 worker threads.
        let rt = Arc::new(
            Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .unwrap(),
        );

        let spawner = TaskSpawner::new(rt.handle().clone()); // Create a TaskSpawner using the runtime handle.
        let (result_tx, mut result_rx) = mpsc::channel::<TaskOutput>(16); // Tokio mpsc channel for results.

        // Spawn a thread to handle the results as they are received.
        let result_handler = rt.spawn(async move {
            let mut i = 0;
            while let Some(result) = result_rx.recv().await {
                println!("Result {}: {:?}", i, result); // Print each result as it's received.
                i += 1;
            }
        });

        // Spawn 100 tasks with the logic for naming based on whether the index is even or odd.
        for i in 0..100 {
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

            let result_tx = result_tx.clone(); // Clone the result transmitter for each task.
            let receiver = rt.block_on(async { spawner.spawn_task(command).await }); // Spawn the task and get the result receiver.

            // Spawn a task to send the task's result to the result handler.
            rt.spawn(async move {
                if let Ok(result) = receiver.await {
                    let _ = result_tx.send(result).await; // Send the result to the result handler.
                }
            });
        }

        rt.block_on(async {
            drop(result_tx); // Close the result channel, indicating no more tasks will send results.
            result_handler.await.unwrap(); // Wait for the result handler to finish processing all results.
        });
    }
}
