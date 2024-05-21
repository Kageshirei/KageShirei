use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::time::Instant;
use tracing::{info, instrument};

use rs2_utils::duration_extension::DurationExt;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Run any pending migrations on the database
///
/// Migrations are done synchronously as it is a blocking operation, this is fine as it is only done once on startup
/// additionally, the migration process is scoped to drop the connection after the migration is complete and
/// initiate a new connection pool shared by the servers
#[instrument(name = "Database migration", skip_all, fields(latency))]
pub fn run_pending(
	connection_string: &str,
	reuse_connection: bool,
) -> anyhow::Result<Option<PgConnection>> {
	let latency = Instant::now();
	info!("Running migrations");

	let mut connection = PgConnection::establish(connection_string)
		.map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
	connection
		.run_pending_migrations(MIGRATIONS)
		.map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

	tracing::Span::current().record(
		"latency",
		humantime::format_duration(latency.elapsed().round()).to_string(),
	);
	info!("Migrations completed successfully");

	// if we are reusing the connection, return it
	if !reuse_connection {
		Ok(None)
	} else {
		Ok(Some(connection))
	}
}
