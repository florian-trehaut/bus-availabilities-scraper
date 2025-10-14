use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserPassengers::Table)
                    .if_not_exists()
                    .col(uuid(UserPassengers::UserRouteId).primary_key())
                    .col(small_integer(UserPassengers::AdultMen).default(0))
                    .col(small_integer(UserPassengers::AdultWomen).default(0))
                    .col(small_integer(UserPassengers::ChildMen).default(0))
                    .col(small_integer(UserPassengers::ChildWomen).default(0))
                    .col(small_integer(UserPassengers::HandicapAdultMen).default(0))
                    .col(small_integer(UserPassengers::HandicapAdultWomen).default(0))
                    .col(small_integer(UserPassengers::HandicapChildMen).default(0))
                    .col(small_integer(UserPassengers::HandicapChildWomen).default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_passengers_user_route_id")
                            .from(UserPassengers::Table, UserPassengers::UserRouteId)
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
            .drop_table(Table::drop().table(UserPassengers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserPassengers {
    Table,
    UserRouteId,
    AdultMen,
    AdultWomen,
    ChildMen,
    ChildWomen,
    HandicapAdultMen,
    HandicapAdultWomen,
    HandicapChildMen,
    HandicapChildWomen,
}

#[derive(DeriveIden)]
enum UserRoutes {
    Table,
    Id,
}
