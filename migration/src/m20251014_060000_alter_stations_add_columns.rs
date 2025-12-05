use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add area_id column (default 1 for existing rows)
        manager
            .alter_table(
                Table::alter()
                    .table(Stations::Table)
                    .add_column(
                        ColumnDef::new(Stations::AreaId)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .to_owned(),
            )
            .await?;

        // Add created_at column with constant default
        // SQLite doesn't support CURRENT_TIMESTAMP in ALTER TABLE ADD COLUMN
        manager
            .alter_table(
                Table::alter()
                    .table(Stations::Table)
                    .add_column(
                        ColumnDef::new(Stations::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default("2025-10-14 00:00:00"),
                    )
                    .to_owned(),
            )
            .await?;

        // Drop old index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_stations_route_id")
                    .table(Stations::Table)
                    .to_owned(),
            )
            .await
            .ok(); // Ignore error if index doesn't exist

        // Make route_id nullable by recreating column
        // SQLite doesn't support ALTER COLUMN, so we need workaround:
        // Create temp table, copy data, drop old, rename temp
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE stations_new (
                    station_id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    area_id INTEGER NOT NULL DEFAULT 1,
                    route_id INTEGER,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                INSERT INTO stations_new (station_id, name, area_id, route_id, created_at)
                SELECT station_id, name, area_id, route_id, created_at FROM stations;

                DROP TABLE stations;

                ALTER TABLE stations_new RENAME TO stations;
                "#,
            )
            .await?;

        // Create new composite index
        manager
            .create_index(
                Index::create()
                    .name("idx_stations_area_route")
                    .table(Stations::Table)
                    .col(Stations::AreaId)
                    .col(Stations::RouteId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: recreate table with old schema
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE stations_old (
                    station_id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    route_id INTEGER NOT NULL
                );

                INSERT INTO stations_old (station_id, name, route_id)
                SELECT station_id, name, COALESCE(route_id, 0) FROM stations;

                DROP TABLE stations;

                ALTER TABLE stations_old RENAME TO stations;
                "#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stations_route_id")
                    .table(Stations::Table)
                    .col(Stations::RouteId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Stations {
    Table,
    #[allow(dead_code)]
    StationId,
    #[allow(dead_code)]
    Name,
    AreaId,
    RouteId,
    CreatedAt,
}
