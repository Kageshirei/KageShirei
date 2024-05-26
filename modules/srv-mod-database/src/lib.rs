use std::sync::Arc;

pub use bb8;
pub use diesel;
pub use diesel_async;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
pub use diesel_migrations;
pub use humantime;
pub use uuid;

pub mod migration;
pub mod schema;
pub mod models;

pub type Pool = Arc<bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>>;

