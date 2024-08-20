use std::sync::Arc;
use std::thread;

#[cfg(feature = "std-runtime")]
use mod_std_runtime::CustomRuntime;

use rs2_runtime::Runtime;

#[cfg(feature = "std-runtime")]
use std::sync::mpsc;

#[cfg(feature = "tokio-runtime")]
use tokio::runtime::Handle;
#[cfg(feature = "tokio-runtime")]
use tokio::sync::{mpsc, oneshot};

#[cfg(feature = "tokio-runtime")]
use mod_tokio_runtime::TokioRuntimeWrapper;

use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;

use crate::command::{exit_command, task_type_a, task_type_b};

#[cfg(feature = "tokio-runtime")]
// Task structure holds the command and a sender to return the result.
pub struct Task {
    command: SimpleAgentCommand,
    response: oneshot::Sender<TaskOutput>,
}

#[cfg(feature = "tokio-runtime")]
#[derive(Clone)]
pub struct TaskSpawner {
    spawn: mpsc::Sender<Task>, // Tokio mpsc channel to send tasks to be processed.
}

#[cfg(feature = "tokio-runtime")]
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

#[cfg(feature = "tokio-runtime")]
#[derive(Clone)]
pub struct TaskSpawnerTokioRuntime {
    spawn: mpsc::Sender<Task>, // Tokio mpsc channel to send tasks to be processed.
}

#[cfg(feature = "tokio-runtime")]
impl TaskSpawnerTokioRuntime {
    /// Creates a new TaskSpawnerTokioRuntime with a given `TokioRuntimeWrapper`.
    ///
    /// # Arguments
    ///
    /// * `runtime` - An `Arc<TokioRuntimeWrapper>` used to spawn tasks.
    ///
    /// # Returns
    ///
    /// * A new instance of TaskSpawnerTokioRuntime.
    pub fn new(runtime: Arc<TokioRuntimeWrapper>) -> TaskSpawnerTokioRuntime {
        let (send, mut recv) = mpsc::channel(16); // Tokio mpsc channel to queue tasks for processing.
        let handle = runtime.handle().clone(); // Extract the Tokio runtime handle from the wrapper.

        // Spawn a background task to receive and handle tasks.
        handle.spawn(async move {
            while let Some(task) = recv.recv().await {
                tokio::spawn(handle_task(task)); // Spawn a new async task for each received task.
            }
        });

        TaskSpawnerTokioRuntime { spawn: send }
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
#[cfg(feature = "tokio-runtime")]
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
    use rs2_communication_protocol::metadata::Metadata;
    use std::sync::Arc;

    #[cfg(feature = "tokio-runtime")]
    use mod_tokio_runtime::TokioRuntimeWrapper;
    #[cfg(feature = "tokio-runtime")]
    use tokio::runtime::Builder;
    #[cfg(feature = "tokio-runtime")]
    use tokio::sync::mpsc;

    #[cfg(feature = "tokio-runtime")]
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

    #[cfg(feature = "tokio-runtime")]
    #[test]
    fn sync_async_test_tokio() {
        // Creare una nuova runtime Tokio utilizzando il wrapper
        let rt = Arc::new(TokioRuntimeWrapper::new(4));

        // Creare un TaskSpawnerTokioRuntime utilizzando la runtime di Tokio
        let spawner = TaskSpawnerTokioRuntime::new(rt.clone());

        // Creare un canale per ricevere i risultati delle task
        let (result_tx, mut result_rx) = mpsc::channel::<TaskOutput>(16);

        // Spawnare un task per gestire i risultati man mano che vengono ricevuti
        let result_handler = rt.handle().spawn(async move {
            let mut i = 0;
            while let Some(result) = result_rx.recv().await {
                println!("Result {}: {:?}", i, result);
                i += 1;
            }
        });

        // Spawnare 100 task con logica di assegnazione dei nomi basata sull'indice (pari/dispari)
        for i in 0..100 {
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

            let result_tx = result_tx.clone(); // Clone the result transmitter for each task.
            let spawner_clone = spawner.clone();
            let receiver = rt.block_on(async move { spawner_clone.spawn_task(command).await }); // Spawn the task and get the result receiver.

            // Spawn a task to send the task's result to the result handler.
            rt.handle().spawn(async move {
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

    #[cfg(feature = "std-runtime")]
    #[test]
    fn sync_async_test_custom_runtime() {
        // Create a new CustomRuntime with 4 worker threads.
        let rt = Arc::new(CustomRuntime::new(4));

        let spawner = TaskSpawnerCustomRuntime::new(rt.clone()); // Create a TaskSpawner using the runtime.
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>(); // Channel for results.

        // Spawn a thread to handle the results as they are received.
        thread::spawn(move || {
            let mut i = 0;
            while let Ok(result) = result_rx.recv() {
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

            let receiver = spawner.spawn_task(command); // Spawn the task and get the result receiver.

            // Send the task's result to the result handler.
            if let Ok(result) = receiver.recv() {
                let _ = result_tx.send(result);
            }
        }
    }
}
