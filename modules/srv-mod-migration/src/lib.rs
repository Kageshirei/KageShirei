pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migration_table_name() -> DynIden {
        Alias::new("kageshirei_migrations").into_iden()
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241012_035559_create_users_table::Migration),
            Box::new(m20241012_041618_create_agents_table::Migration),
        ]
    }
}
mod m20241012_035559_create_users_table;
mod m20241012_041618_create_agents_table;
