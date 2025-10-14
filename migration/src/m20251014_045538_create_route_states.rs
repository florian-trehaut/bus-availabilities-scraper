use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RouteStates::Table)
                    .if_not_exists()
                    .col(uuid(RouteStates::UserRouteId).primary_key())
                    .col(string(RouteStates::LastSeenHash).default(""))
                    .col(timestamp_null(RouteStates::LastCheck))
                    .col(big_integer(RouteStates::TotalChecks).default(0))
                    .col(big_integer(RouteStates::TotalAlerts).default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_route_states_user_route_id")
                            .from(RouteStates::Table, RouteStates::UserRouteId)
                            .to(UserRoutes::Table, UserRoutes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RouteStates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RouteStates {
    Table,
    UserRouteId,
    LastSeenHash,
    LastCheck,
    TotalChecks,
    TotalAlerts,
}

#[derive(DeriveIden)]
enum UserRoutes {
    Table,
    Id,
}
