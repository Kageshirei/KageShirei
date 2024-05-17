use std::sync::Arc;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

pub type Pool = Arc<bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>>;

pub mod schema;
pub mod models;
pub mod migration;
