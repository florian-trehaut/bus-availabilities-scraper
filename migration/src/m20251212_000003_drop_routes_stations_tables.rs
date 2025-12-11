use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop stations table first (has FK to routes)
        manager
            .drop_table(Table::drop().table(Stations::Table).if_exists().to_owned())
            .await?;

        // Drop routes table
        manager
            .drop_table(Table::drop().table(Routes::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate routes table
        manager
            .create_table(
                Table::create()
                    .table(Routes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Routes::RouteId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Routes::AreaId).integer().not_null())
                    .col(ColumnDef::new(Routes::Name).string().not_null())
                    .col(ColumnDef::new(Routes::SwitchChangeableFlg).string())
                    .col(ColumnDef::new(Routes::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Recreate stations table
        manager
            .create_table(
                Table::create()
                    .table(Stations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Stations::StationId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Stations::Name).string().not_null())
                    .col(ColumnDef::new(Stations::AreaId).integer().not_null())
                    .col(ColumnDef::new(Stations::RouteId).string())
                    .col(ColumnDef::new(Stations::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Routes {
    Table,
    RouteId,
    AreaId,
    Name,
    SwitchChangeableFlg,
    CreatedAt,
}

#[derive(Iden)]
enum Stations {
    Table,
    StationId,
    Name,
    AreaId,
    RouteId,
    CreatedAt,
}
