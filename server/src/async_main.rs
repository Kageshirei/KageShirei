use std::fs;

use log::error;
use tokio::join;
use tracing::{event, info, instrument, Level};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

use crate::config::config::{ReadOnlyConfig, SharedConfig};

mod api_server;

/// Set up the logging for the server
pub fn setup_logging(config: &ReadOnlyConfig) -> anyhow::Result<()> {
	if !config.log.console.enabled && !config.log.file.enabled {
		error!("No logging enabled, this is not supported, exiting");
		return Err(anyhow::anyhow!("No logging enabled, cannot continue"));
	}

	let mut layers = Vec::new();

	if config.log.console.enabled {
		layers.push(
			tracing_subscriber::fmt::layer()
				.with_writer(std::io::stdout)
				.pretty()
				.with_ansi(true)
				.with_level(true)
				.with_line_number(config.debug_level.unwrap_or(0) >= 1)
				.with_thread_ids(true)
				.with_thread_names(true)
				.with_target(true)
				.with_file(config.debug_level.unwrap_or(0) >= 1)
				.with_filter(match config.debug_level.unwrap_or(0) {
					0 => LevelFilter::INFO,
					1 => LevelFilter::DEBUG,
					_ => LevelFilter::TRACE,
				})
				.boxed()
		)
	}

	if config.log.file.enabled {
		fs::create_dir_all(&config.log.file.path.parent().unwrap()).map_err(|e| {
			error!("Failed to create log directory: {}", e);
			e
		})?;

		let file = fs::OpenOptions::new()
			.append(true)
			.create(true)
			.open(&config.log.file.path)
			.map_err(|e| {
				error!("Failed to open log file: {}", e);
				e
			})?;

		layers.push(
			tracing_subscriber::fmt::layer()
				.with_writer(file)
				.json()
				.with_span_list(true)
				.with_ansi(false)
				.with_level(true)
				.with_line_number(config.debug_level.unwrap_or(0) >= 1)
				.with_thread_ids(true)
				.with_thread_names(true)
				.with_target(true)
				.with_file(config.debug_level.unwrap_or(0) >= 1)
				.with_filter(match config.debug_level.unwrap_or(0) {
					0 => LevelFilter::INFO,
					1 => LevelFilter::DEBUG,
					_ => LevelFilter::TRACE,
				})
				.boxed()
		);
	}

	let subscriber = tracing_subscriber::registry()
		.with(layers);

	tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

	Ok(())
}

/// The main entry point for async runtime of the server this will be called by the main function and is responsible
/// for setting up the server and running it
pub async fn async_main(config: SharedConfig) -> anyhow::Result<()> {
	let readonly_config = config.read().await;
	setup_logging(&readonly_config)?;

	let api_server = api_server::start(config.clone());
	let api_server_thread = tokio::spawn(async move { api_server.await.unwrap() });

	// wait for all the threads to finish
	join!(api_server_thread);

	Ok(())
}