use std::{path::PathBuf, sync::Arc};

use kageshirei_utils::{unrecoverable_error::unrecoverable_error};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};
use validator::{Validate, ValidationErrors};

use crate::{
    api_server::ApiServerConfig,
    database::DatabaseConfig,
    handlers::HandlerConfig,
    jwt::JwtConfig,
    logging::LogConfig,
};

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
    pub api_server: ApiServerConfig,

    /// The log configuration
    #[validate(nested)]
    pub log: LogConfig,

    /// The JWT configuration
    #[validate(nested)]
    pub jwt: JwtConfig,

    /// The database configuration
    #[validate(nested)]
    pub database: DatabaseConfig,

    /// The handlers configuration
    #[validate(nested)]
    pub handlers: Vec<HandlerConfig>,

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
    pub fn load(path: &PathBuf) -> Result<SharedConfig, String> {
        let path = std::env::current_dir().unwrap().join(path);
        if !path.exists() {
            error!("Failed to load configuration");
            error!(
                "Cannot parse configuration file: Configuration file not found at {}",
                path.display()
            );
            unrecoverable_error()?; // Exit with error state
        }

        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let config: Self = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        Self::handle_loading_errors(config.validate())?;

        Ok(Arc::new(RwLock::new(config)))
    }

    /// handle the loading errors if any, exiting if errors are found
    fn handle_loading_errors(result: Result<(), ValidationErrors>) -> Result<(), String> {
        if let Err(e) = result {
            error!("Failed to load configuration");
            print_validation_error::print_validation_error(e)?;
            unrecoverable_error()?; // Exit with error state
        }

        Ok(())
    }
}
