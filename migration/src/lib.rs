pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20251014_045535_create_user_routes;
mod m20251014_045537_create_user_passengers;
mod m20251014_045538_create_route_states;
mod m20251014_045540_create_stations;
mod m20251014_054404_create_routes_catalog;
mod m20251014_060000_alter_stations_add_columns;
mod m20251212_000001_fix_route_id_types;
mod m20251212_000002_seed_routes_data;
mod m20251212_000003_drop_routes_stations_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251014_045535_create_user_routes::Migration),
            Box::new(m20251014_045537_create_user_passengers::Migration),
            Box::new(m20251014_045538_create_route_states::Migration),
            Box::new(m20251014_054404_create_routes_catalog::Migration),
            Box::new(m20251014_045540_create_stations::Migration),
            Box::new(m20251014_060000_alter_stations_add_columns::Migration),
            Box::new(m20251212_000001_fix_route_id_types::Migration),
            Box::new(m20251212_000002_seed_routes_data::Migration),
            Box::new(m20251212_000003_drop_routes_stations_tables::Migration),
        ]
    }
}
