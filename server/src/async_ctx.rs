//! # Async Context
//! This module provides the necessary functions to run futures in a tokio runtime.

use std::future::Future;

use log::{info, trace, warn};
use srv_mod_config::SharedConfig;

/// Run the given future in a tokio runtime
///
/// # Arguments
///
/// * `future` - The future to run
pub fn enter<F: Future + Send>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

/// Initialize the context for the async server runtime and run the given future
///
/// # Arguments
///
/// * `debug_level` - The debug level to use
/// * `config` - The shared configuration
/// * `future` - The future to run
pub async fn init_context<F: Future + Send>(debug_level: u8, config: SharedConfig, future: F) -> F::Output {
    info!("Initializing context...");
    trace!("Starting async runtime...");

    if debug_level > config.read().await.debug_level.unwrap_or(0) {
        warn!(
            "Command line debug level is higher than the defined in the configuration file, debug level will be \
             overridden"
        );
        config.write().await.debug_level = Some(debug_level);
    }

    trace!("Loaded configuration: {:?}", config);
    info!("Configuration successfully loaded!");
    warn!("Switching context to the async server runtime...");

    future.await
}
