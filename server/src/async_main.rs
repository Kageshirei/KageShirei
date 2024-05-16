use std::fs;
use std::sync::Arc;

use axum::handler::Handler;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use log::error;
use tokio::{join, select, signal};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

use crate::config::config::{ReadOnlyConfig, SharedConfig};
use crate::config::log::ConsoleLogFormat;

mod api_server;

fn build_logger<S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>>(debug_level: u8, format: ConsoleLogFormat) -> Box<dyn Layer<S> + Send + Sync + 'static> {
	match format {
		ConsoleLogFormat::Json => tracing_subscriber::fmt::layer()
			.json()
			.with_span_list(true).with_level(true)
			.with_line_number(debug_level >= 1)
			.with_thread_ids(true)
			.with_thread_names(true)
			.with_target(true)
			.with_file(debug_level >= 1)
			.with_filter(match debug_level {
				0 => LevelFilter::INFO,
				1 => LevelFilter::DEBUG,
				_ => LevelFilter::TRACE,
			})
			.boxed(),
		ConsoleLogFormat::Pretty => tracing_subscriber::fmt::layer()
			.with_writer(std::io::stdout)
			.pretty().with_level(true)
			.with_line_number(debug_level >= 1)
			.with_thread_ids(true)
			.with_thread_names(true)
			.with_target(true)
			.with_file(debug_level >= 1)
			.with_filter(match debug_level {
				0 => LevelFilter::INFO,
				1 => LevelFilter::DEBUG,
				_ => LevelFilter::TRACE,
			})
			.boxed(),
		ConsoleLogFormat::Full => tracing_subscriber::fmt::layer()
			.with_writer(std::io::stdout).with_level(true)
			.with_line_number(debug_level >= 1)
			.with_thread_ids(true)
			.with_thread_names(true)
			.with_target(true)
			.with_file(debug_level >= 1)
			.with_filter(match debug_level {
				0 => LevelFilter::INFO,
				1 => LevelFilter::DEBUG,
				_ => LevelFilter::TRACE,
			})
			.boxed(),
		ConsoleLogFormat::Compact => tracing_subscriber::fmt::layer()
			.with_writer(std::io::stdout)
			.compact()
			.with_level(true)
			.with_line_number(false)
			.with_thread_ids(false)
			.with_thread_names(false)
			.with_target(true)
			.with_file(false)
			.with_filter(match debug_level {
				0 => LevelFilter::INFO,
				1 => LevelFilter::DEBUG,
				_ => LevelFilter::TRACE,
			})
			.boxed(),
	}
}

/// Set up the logging for the server
pub fn setup_logging(config: &ReadOnlyConfig) -> anyhow::Result<()> {
	if !config.log.console.enabled && !config.log.file.enabled {
		error!("No logging enabled, this is not supported, exiting");
		return Err(anyhow::anyhow!("No logging enabled, cannot continue"));
	}

	let mut layers = Vec::new();

	if config.log.console.enabled {
		layers.push(
			build_logger(config.debug_level.unwrap_or(0), config.log.console.format.clone())
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

	let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&readonly_config.database.url);
	let pool = Arc::new(
		RwLock::new(bb8::Pool::builder()
			.max_size(readonly_config.database.pool_size as u32)
			.build(connection_manager)
			.await?
		)
	);

	// create a cancellation token to be used to signal the servers to shutdown
	let cancellation_token = CancellationToken::new();

	let api_server_task = api_server::start(
		config.clone(),
		cancellation_token.clone(),
		pool.clone(),
	);
	let api_server_thread = tokio::spawn(async move {
		api_server_task.await.unwrap();
	});

	let cancellation_handler_thread = tokio::spawn(async move {
		handle_shutdown_signals(cancellation_token).await;
	});

	// wait for all the threads to finish
	let _ = join!(
		cancellation_handler_thread,
		api_server_thread
	);

	Ok(())
}

/// Handle the shutdown signal gracefully closing all connections and waiting for all requests to complete
async fn handle_shutdown_signals(cancellation_token: CancellationToken) {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
		let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
		let terminate = std::future::pending::<()>();

	select! {
        _ = ctrl_c => {
			warn!("Received Ctrl+C, shutting down ...");
			cancellation_token.cancel();
		},
        _ = terminate => {
			warn!("Received termination signal, shutting down ...");
			cancellation_token.cancel();
		},
    }
}