pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migration_table_name() -> DynIden { Alias::new("kageshirei_migrations").into_iden() }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241012_035559_create_users_table::Migration),
            Box::new(m20241012_041618_create_agents_table::Migration),
            Box::new(m20241012_070459_create_agent_profiles_table::Migration),
            Box::new(m20241012_070513_create_filters_table::Migration),
            Box::new(m20241012_070519_create_logs_table::Migration),
            Box::new(m20241012_070535_create_terminal_history_table::Migration),
            Box::new(m20241012_070555_create_agent_command_table::Migration),
        ]
    }
}
pub mod m20241012_035559_create_users_table;
pub mod m20241012_041618_create_agents_table;
pub mod m20241012_070459_create_agent_profiles_table;
pub mod m20241012_070513_create_filters_table;
pub mod m20241012_070519_create_logs_table;
pub mod m20241012_070535_create_terminal_history_table;
pub mod m20241012_070555_create_agent_command_table;
