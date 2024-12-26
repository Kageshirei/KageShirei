//! Server module migration main file
use sea_orm_migration::prelude::*;
use srv_mod_migration::Migrator;
#[async_std::main]
async fn main() { cli::run_cli(Migrator).await; }
