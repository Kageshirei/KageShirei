use std::path::PathBuf;
use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};
use validator::{Validate, ValidationErrors};

use crate::config::api_server::ApiServerConfig;
use crate::config::database::DatabaseConfig;
use crate::config::jwt::JwtConfig;
use crate::config::log::LogConfig;
use crate::print_validation_error;
use crate::unrecoverable_error::unrecoverable_error;

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
	pub fn load(path: &PathBuf) -> anyhow::Result<SharedConfig> {
		let path = std::env::current_dir().unwrap().join(path);
		if !path.exists() {
			error!("Failed to load configuration");
			error!("Cannot parse configuration file: Configuration file not found at {}", path.display());
			unrecoverable_error()?; // Exit with error state
		}

		let file = std::fs::File::open(path)?;
		let config: Self = serde_json::from_reader(file)?;

		Self::handle_loading_errors(config.validate())?;

		Ok(Arc::new(RwLock::new(config.clone())))
	}

	/// handle the loading errors if any, exiting if errors are found
	fn handle_loading_errors(result: Result<(), ValidationErrors>) -> anyhow::Result<()> {
		if let Err(e) = result {
			error!("Failed to load configuration");
			print_validation_error::print_validation_error(e)?;
			unrecoverable_error()?; // Exit with error state
		}

		Ok(())
	}
}
