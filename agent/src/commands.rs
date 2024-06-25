use core::ffi::c_void;
use libc_print::libc_println;
use mod_agentcore::instance;
use mod_protocol_json::protocol::JsonProtocol;
use mod_win32::nt_time::check_kill_date;
use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

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

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a IdentEncryptor struct from a raw pointer.
pub unsafe fn encryptor_from_raw(ptr: *mut c_void) -> &'static mut IdentEncryptor {
    &mut *(ptr as *mut IdentEncryptor)
}

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut JsonProtocol<IdentEncryptor> {
    &mut *(ptr as *mut JsonProtocol<IdentEncryptor>)
}
