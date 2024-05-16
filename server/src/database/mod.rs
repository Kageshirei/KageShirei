use std::sync::Arc;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use tokio::sync::RwLock;

pub type Pool = Arc<RwLock<bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>>>;

pub mod schema;