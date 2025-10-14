use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stations::Table)
                    .if_not_exists()
                    .col(string(Stations::StationId).primary_key())
                    .col(string(Stations::Name))
                    .col(integer(Stations::RouteId))
                    .to_owned(),
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

        let insert_stations = Query::insert()
            .into_table(Stations::Table)
            .columns([Stations::StationId, Stations::Name, Stations::RouteId])
            .values_panic(["001".into(), "Busta Shinjuku".into(), 155.into()])
            .values_panic(["498".into(), "Kamikochi Bus Terminal".into(), 155.into()])
            .to_owned();

        manager.exec_stmt(insert_stations).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Stations {
    Table,
    StationId,
    Name,
    RouteId,
}
