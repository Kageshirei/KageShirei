//! The configuration module for the server
//!
//! This module contains the configuration for the server, including the API server, logging, JWT,
//! database, and handlers.
//!
//! It is designed to be the only module that knows how to load the configuration from a file, and
//! to provide a shared configuration object that can be accessed by other modules.

use std::{path::PathBuf, sync::Arc};

use kageshirei_utils::unrecoverable_error::unrecoverable_error;
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};
use validator::{Validate, ValidationErrors};

pub mod api_server;
pub mod database;
mod errors;
pub mod handlers;
pub mod jwt;
pub mod logging;
pub(crate) mod print_validation_error;
pub mod sse;
mod validators;

pub use errors::Configuration;

pub type SharedConfig = Arc<RwLock<RootConfig>>;
pub type ReadOnlyConfig<'a> = RwLockReadGuard<'a, RootConfig>;

/// Root server configuration
#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct RootConfig {
    /// The api server configuration
    #[validate(nested)]
    pub api_server: api_server::Config,

    /// The log configuration
    #[validate(nested)]
    pub log: logging::Config,

    /// The JWT configuration
    #[validate(nested)]
    pub jwt: jwt::Config,

    /// The database configuration
    #[validate(nested)]
    pub database: database::Config,

    /// The handlers configuration
    #[validate(nested)]
    pub handlers: Vec<handlers::Config>,

    /// The level of debug output to provide, in the range 0-2
    ///
    /// 0: Info
    /// 1: Debug
    /// 2: Trace
    ///
    /// Defaults to 0 or inherit from command line (if higher)
    #[validate(range(min = 0, max = 2))]
    pub debug_level: Option<u8>,
}

impl RootConfig {
    /// Load the configuration from a file
    pub fn load(path: &PathBuf) -> Result<SharedConfig, Configuration> {
        let path = std::env::current_dir().unwrap().join(path);
        if !path.exists() {
            error!("Failed to load configuration");
            error!(
                "Cannot parse configuration file: Configuration file not found at {}",
                path.display()
            );
            unrecoverable_error().map_err(Configuration::Unrecoverable)?; // Exit with error state
        }

        let file = std::fs::File::open(path).map_err(|e| Configuration::Generic(Box::new(e)))?;
        let config: Self = serde_json::from_reader(file).map_err(|e| Configuration::Generic(Box::new(e)))?;

        Self::handle_loading_errors(config.validate())?;

        Ok(Arc::new(RwLock::new(config)))
    }

    /// handle the loading errors if any, exiting if errors are found
    fn handle_loading_errors(result: Result<(), ValidationErrors>) -> Result<(), Configuration> {
        if let Err(e) = result {
            error!("Failed to load configuration");
            print_validation_error::print_validation_error(e)?;
            unrecoverable_error().map_err(Configuration::Unrecoverable)?; // Exit with error state
        }

        Ok(())
    }
}
