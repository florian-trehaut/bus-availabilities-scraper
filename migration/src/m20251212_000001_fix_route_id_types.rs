use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support ALTER COLUMN, so we need to recreate tables

        // 1. Recreate stations table with route_id as TEXT
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE stations_new (
                    station_id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    area_id INTEGER NOT NULL DEFAULT 1,
                    route_id TEXT,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                INSERT INTO stations_new (station_id, name, area_id, route_id, created_at)
                SELECT station_id, name, area_id, CAST(route_id AS TEXT), created_at FROM stations;

                DROP TABLE stations;

                ALTER TABLE stations_new RENAME TO stations;
                "#,
            )
            .await?;

        // Recreate index for stations
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

        // 2. Recreate user_routes table with route_id as TEXT
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE user_routes_new (
                    id BLOB PRIMARY KEY NOT NULL,
                    user_id BLOB NOT NULL,
                    area_id INTEGER NOT NULL,
                    route_id TEXT NOT NULL,
                    departure_station TEXT NOT NULL,
                    arrival_station TEXT NOT NULL,
                    date_start TEXT NOT NULL,
                    date_end TEXT NOT NULL,
                    departure_time_min TEXT,
                    departure_time_max TEXT,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
                );

                INSERT INTO user_routes_new (id, user_id, area_id, route_id, departure_station, arrival_station, date_start, date_end, departure_time_min, departure_time_max, created_at)
                SELECT id, user_id, area_id, CAST(route_id AS TEXT), departure_station, arrival_station, date_start, date_end, departure_time_min, departure_time_max, created_at FROM user_routes;

                DROP TABLE user_routes;

                ALTER TABLE user_routes_new RENAME TO user_routes;
                "#,
            )
            .await?;

        // Recreate index for user_routes
        manager
            .create_index(
                Index::create()
                    .name("idx_user_routes_user_id")
                    .table(UserRoutes::Table)
                    .col(UserRoutes::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: recreate tables with INTEGER route_id

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE stations_old (
                    station_id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    area_id INTEGER NOT NULL DEFAULT 1,
                    route_id INTEGER,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                INSERT INTO stations_old (station_id, name, area_id, route_id, created_at)
                SELECT station_id, name, area_id, CAST(route_id AS INTEGER), created_at FROM stations;

                DROP TABLE stations;

                ALTER TABLE stations_old RENAME TO stations;
                "#,
            )
            .await?;

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

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE user_routes_old (
                    id BLOB PRIMARY KEY NOT NULL,
                    user_id BLOB NOT NULL,
                    area_id INTEGER NOT NULL,
                    route_id INTEGER NOT NULL,
                    departure_station TEXT NOT NULL,
                    arrival_station TEXT NOT NULL,
                    date_start TEXT NOT NULL,
                    date_end TEXT NOT NULL,
                    departure_time_min TEXT,
                    departure_time_max TEXT,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
                );

                INSERT INTO user_routes_old (id, user_id, area_id, route_id, departure_station, arrival_station, date_start, date_end, departure_time_min, departure_time_max, created_at)
                SELECT id, user_id, area_id, CAST(route_id AS INTEGER), departure_station, arrival_station, date_start, date_end, departure_time_min, departure_time_max, created_at FROM user_routes;

                DROP TABLE user_routes;

                ALTER TABLE user_routes_old RENAME TO user_routes;
                "#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_routes_user_id")
                    .table(UserRoutes::Table)
                    .col(UserRoutes::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Stations {
    Table,
    AreaId,
    RouteId,
}

#[derive(DeriveIden)]
enum UserRoutes {
    Table,
    UserId,
}
