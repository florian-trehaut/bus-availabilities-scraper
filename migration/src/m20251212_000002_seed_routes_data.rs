use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Seed routes data so dropdowns work without requiring API seeding
        // Route 155 corresponds to the stations already seeded (001, 498)
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                INSERT OR IGNORE INTO routes (route_id, area_id, name, switch_changeable_flg, created_at)
                VALUES
                    ('155', 1, 'Shinjuku - Kamikochi / Shirahone Onsen', NULL, CURRENT_TIMESTAMP);
                "#,
            )
            .await?;

        // Update existing stations to link to route 155
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE stations SET route_id = '155' WHERE station_id IN ('001', '498');
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DELETE FROM routes WHERE route_id = '155';
                UPDATE stations SET route_id = NULL WHERE station_id IN ('001', '498');
                "#,
            )
            .await?;

        Ok(())
    }
}
