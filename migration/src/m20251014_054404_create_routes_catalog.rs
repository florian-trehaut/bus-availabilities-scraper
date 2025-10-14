use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Routes::Table)
                    .if_not_exists()
                    .col(string(Routes::RouteId).primary_key())
                    .col(integer(Routes::AreaId))
                    .col(string(Routes::Name))
                    .col(string_null(Routes::SwitchChangeableFlg))
                    .col(timestamp(Routes::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_routes_area_id")
                    .table(Routes::Table)
                    .col(Routes::AreaId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Routes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Routes {
    Table,
    RouteId,
    AreaId,
    Name,
    SwitchChangeableFlg,
    CreatedAt,
}
