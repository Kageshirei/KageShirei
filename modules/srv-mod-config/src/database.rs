use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct DatabaseConfig {
    /// Connection string for the database, refer to
    /// https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING-URIS for more information
    pub url: String,
    /// The maximum number of connections to keep in the pool
    #[validate(range(min = 1, max = 100))]
    pub pool_size: u8,
}
