use std::sync::Arc;

pub use bb8;
use cuid2::CuidConstructor;
pub use diesel;
pub use diesel_async;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
pub use diesel_migrations;
pub use humantime;
use once_cell::sync::Lazy;

pub static CUID2: Lazy<CuidConstructor> = Lazy::new(|| {
	let mut cuid2 = CuidConstructor::new();
	cuid2.set_length(32);
	cuid2
});

pub mod migration;
pub mod schema;
pub mod models;
pub mod schema_extension;

pub type Pool = Arc<bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>>;

