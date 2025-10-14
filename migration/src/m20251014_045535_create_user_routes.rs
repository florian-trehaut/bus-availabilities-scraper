use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserRoutes::Table)
                    .if_not_exists()
                    .col(uuid(UserRoutes::Id).primary_key())
                    .col(uuid(UserRoutes::UserId))
                    .col(integer(UserRoutes::AreaId))
                    .col(integer(UserRoutes::RouteId))
                    .col(string(UserRoutes::DepartureStation))
                    .col(string(UserRoutes::ArrivalStation))
                    .col(string(UserRoutes::DateStart))
                    .col(string(UserRoutes::DateEnd))
                    .col(string_null(UserRoutes::DepartureTimeMin))
                    .col(string_null(UserRoutes::DepartureTimeMax))
                    .col(timestamp(UserRoutes::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_routes_user_id")
                            .from(UserRoutes::Table, UserRoutes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserRoutes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserRoutes {
    Table,
    Id,
    UserId,
    AreaId,
    RouteId,
    DepartureStation,
    ArrivalStation,
    DateStart,
    DateEnd,
    DepartureTimeMin,
    DepartureTimeMax,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
