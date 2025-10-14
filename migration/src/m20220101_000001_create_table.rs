use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .col(string(Users::Email).unique_key())
                    .col(boolean(Users::Enabled).default(true))
                    .col(boolean(Users::NotifyOnChangeOnly).default(true))
                    .col(big_integer(Users::ScrapeIntervalSecs).default(300))
                    .col(string_null(Users::DiscordWebhookUrl))
                    .col(timestamp(Users::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_enabled")
                    .table(Users::Table)
                    .col(Users::Enabled)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Enabled,
    NotifyOnChangeOnly,
    ScrapeIntervalSecs,
    DiscordWebhookUrl,
    CreatedAt,
}
