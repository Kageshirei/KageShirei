pub mod tokio_runtime_wrapper;

use tokio::{
    runtime::Handle,
    sync::{mpsc, oneshot},
};
pub use tokio_runtime_wrapper::TokioRuntimeWrapper;

// Task structure holds the name of the task and a sender to return the result.
pub struct Task {
    name:     String,
    response: oneshot::Sender<String>,
}

// Simulated asynchronous task that takes 2 seconds to complete.
async fn task_type_a() -> String { "Result from task type A".to_string() }

// Simulated asynchronous task that takes 3 seconds to complete.
async fn task_type_b() -> String {
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    "Result from task type B".to_string()
}

// Function to handle a task based on its name.
async fn handle_task(task: Task) {
    let result = match task.name.as_str() {
        "Long Task" => task_type_b().await,
        _ => task_type_a().await,
    };
    let _ = task.response.send(result);
}

// TaskSpawner manages task spawning and communication between the main thread and worker threads.
#[derive(Clone)]
pub struct TaskSpawner {
    spawn: mpsc::Sender<Task>,
}

impl TaskSpawner {
    // Creates a new TaskSpawner with a given Tokio runtime handle.
    pub fn new(rt: Handle) -> TaskSpawner {
        let (send, mut recv) = mpsc::channel(16);

        rt.spawn(async move {
            while let Some(task) = recv.recv().await {
                tokio::spawn(handle_task(task));
            }
        });

        TaskSpawner {
            spawn: send,
        }
    }

    // Method to spawn a new task by name, returning a receiver to get the task's result.
    pub async fn spawn_task(&self, name: String) -> oneshot::Receiver<String> {
        let (response_tx, response_rx) = oneshot::channel();
        let task = Task {
            name,
            response: response_tx,
        };
        if let Err(_) = self.spawn.send(task).await {
            panic!("The shared runtime has shut down.");
        }
        response_rx
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use tokio::{
        runtime::{Builder, Runtime},
        sync::mpsc,
    };

    use super::*;

    #[test]
    fn custom_runtime_test_with_tokio() {
        // Create a multi-threaded Tokio runtime with 8 worker threads.
        let rt = Arc::new(
            Builder::new_multi_thread()
                .worker_threads(8)
                .enable_all()
                .build()
                .unwrap(),
        );

        let spawner = TaskSpawner::new(rt.handle().clone());
        let (result_tx, mut result_rx) = mpsc::channel::<String>(16);

        // Thread to handle results.
        let result_handler = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut i = 0;
                while let Some(result) = result_rx.recv().await {
                    println!("Result {}: {}", i, result);
                    i += 1;
                }
            });
        });

        // Spawn 100 tasks with logic for alternating long and short tasks.
        for i in 0 .. 100 {
            let task_name = if i % 2 == 0 {
                "Long Task".to_string()
            }
            else {
                format!("Test Task {}", i)
            };

            let result_tx = result_tx.clone();
            let receiver = rt.block_on(async { spawner.spawn_task(task_name).await });

            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    if let Ok(result) = receiver.await {
                        let _ = result_tx.send(result).await;
                    }
                });
            });
        }

        drop(result_tx); // Close the result channel, indicating no more tasks will send results.
        result_handler.join().unwrap(); // Wait for the result handler to finish processing all results.
    }
}
