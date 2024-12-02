//! This module contains the main entry point for the server, this is where the async runtime is
//! started and the server is run.

#![allow(
    clippy::integer_division_remainder_used,
    reason = "Used mainly in macros"
)]

use std::{fs, sync::Arc};

use srv_mod_config::{handlers::HandlerType, logging::ConsoleFormat, ReadOnlyConfig, SharedConfig};
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use tracing::{error, level_filters::LevelFilter, warn};
use tracing_subscriber::{layer::SubscriberExt as _, Layer};

use crate::{
    auto_migrate,
    servers::{api_server, http_handler},
};

/// Build the logger for the server
fn build_logger<S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>>(
    debug_level: u8,
    format: ConsoleFormat,
) -> Box<dyn Layer<S> + Send + Sync + 'static> {
    match format {
        ConsoleFormat::Json => {
            tracing_subscriber::fmt::layer()
                .json()
                .with_span_list(true)
                .with_level(true)
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
                .boxed()
        },
        ConsoleFormat::Pretty => {
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .pretty()
                .with_level(true)
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
                .boxed()
        },
        ConsoleFormat::Full => {
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_level(true)
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
                .boxed()
        },
        ConsoleFormat::Compact => {
            tracing_subscriber::fmt::layer()
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
                .boxed()
        },
    }
}

/// Set up the logging for the server
pub fn setup_logging(config: &ReadOnlyConfig) -> Result<(), String> {
    if !config.log.console.enabled && !config.log.file.enabled {
        error!("No logging enabled, this is not supported, exiting");
        return Err("No logging enabled, cannot continue".to_owned());
    }

    let mut layers = Vec::new();

    if config.log.console.enabled {
        layers.push(build_logger(
            config.debug_level.unwrap_or(0),
            config.log.console.format.clone(),
        ))
    }

    if config.log.file.enabled {
        fs::create_dir_all(config.log.file.path.parent().unwrap()).map_err(|e| {
            error!("Failed to create log directory: {}", e);
            e.to_string()
        })?;

        let file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.log.file.path)
            .map_err(|e| {
                error!("Failed to open log file: {}", e);
                e.to_string()
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
                .boxed(),
        );
    }

    let subscriber = tracing_subscriber::registry().with(layers);

    #[expect(
        clippy::expect_used,
        reason = "This is a safe operation, the cli should die if the logging does not work for some reason"
    )]
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    Ok(())
}

/// The main entry point for async runtime of the server this will be called by the main function
/// and is responsible for setting up the server and running it
pub async fn async_main(config: SharedConfig) -> Result<(), String> {
    let readonly_config = config.read().await;
    setup_logging(&readonly_config)?;

    // run the migrations on server startup
    let db = auto_migrate::run(&readonly_config.database.url, &readonly_config).await?;

    // create a cancellation token to be used to signal the servers to shut down
    let cancellation_token = CancellationToken::new();

    let api_server_thread = api_server::spawn(config.clone(), cancellation_token.clone(), db.clone());

    // create a vector to hold all the threads
    let mut pending_threads = vec![api_server_thread];

    // iterate over all the handlers and start the ones that are enabled
    for handler in readonly_config.handlers.iter() {
        // if the handler is not enabled skip it
        if !handler.enabled {
            continue;
        }

        match handler.r#type {
            HandlerType::Http => {
                pending_threads.push(http_handler::spawn(
                    Arc::new(handler.clone()),
                    cancellation_token.clone(),
                    db.clone(),
                ));
            },
        }
    }

    drop(readonly_config);

    let cancellation_handler_thread = tokio::spawn(async move {
        handle_shutdown_signals(cancellation_token).await;
    });
    pending_threads.push(cancellation_handler_thread);

    // wait for all the threads to finish
    futures::future::join_all(pending_threads).await;

    Ok(())
}

/// Handle the shutdown signal gracefully closing all connections and waiting for all requests to
/// complete
async fn handle_shutdown_signals(cancellation_token: CancellationToken) {
    let ctrl_c = async {
        #[expect(
            clippy::expect_used,
            reason = "This is a safe operation, the cli should die if for some reason it cannot be stopped manually"
        )]
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
