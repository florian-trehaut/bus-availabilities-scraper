//! Integration tests for seed.rs
//!
//! Tests database seeding functionality with SQLite in-memory

use app::db::init_database;
use app::entities::{prelude::*, users};
use app::seed::seed_from_env;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use temp_env::async_with_vars;

async fn setup_test_db() -> sea_orm::DatabaseConnection {
    let db = init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

fn base_env_vars() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("AREA_ID", Some("100")),
        ("ROUTE_ID", Some("110")),
        ("DEPARTURE_STATION", Some("001")),
        ("ARRIVAL_STATION", Some("064")),
        ("DATE_START", Some("2025-01-15")),
        ("DATE_END", Some("2025-01-20")),
        ("ADULT_MEN", Some("2")),
        ("ADULT_WOMEN", Some("1")),
        ("CHILD_MEN", Some("0")),
        ("CHILD_WOMEN", Some("0")),
        ("HANDICAP_ADULT_MEN", Some("0")),
        ("HANDICAP_ADULT_WOMEN", Some("0")),
        ("HANDICAP_CHILD_MEN", Some("0")),
        ("HANDICAP_CHILD_WOMEN", Some("0")),
        ("SCRAPE_INTERVAL_SECS", Some("300")),
        ("NOTIFY_ON_CHANGE_ONLY", Some("true")),
        ("DISCORD_WEBHOOK_URL", Some("https://discord.webhook/test")),
    ]
}

#[tokio::test]
#[serial]
async fn test_seed_creates_user() {
    async_with_vars(base_env_vars(), async {
        let db = setup_test_db().await;
        seed_from_env(&db).await.unwrap();

        let user = Users::find()
            .filter(users::Column::Email.eq("beta@bus-scraper.local"))
            .one(&db)
            .await
            .unwrap();

        assert!(user.is_some());
        let user = user.unwrap();
        assert!(user.enabled);
        assert!(user.notify_on_change_only);
        assert_eq!(user.scrape_interval_secs, 300);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_seed_creates_route() {
    async_with_vars(base_env_vars(), async {
        let db = setup_test_db().await;
        seed_from_env(&db).await.unwrap();

        let routes = UserRoutes::find().all(&db).await.unwrap();

        assert_eq!(routes.len(), 1);
        let route = &routes[0];
        assert_eq!(route.area_id, 100);
        assert_eq!(route.route_id, "110");
        assert_eq!(route.departure_station, "001");
        assert_eq!(route.arrival_station, "064");
        assert_eq!(route.date_start, "2025-01-15");
        assert_eq!(route.date_end, "2025-01-20");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_seed_creates_passengers() {
    async_with_vars(base_env_vars(), async {
        let db = setup_test_db().await;
        seed_from_env(&db).await.unwrap();

        let routes = UserRoutes::find().all(&db).await.unwrap();
        assert_eq!(routes.len(), 1);

        let passengers = UserPassengers::find_by_id(routes[0].id)
            .one(&db)
            .await
            .unwrap();

        assert!(passengers.is_some());
        let passengers = passengers.unwrap();
        assert_eq!(passengers.adult_men, 2);
        assert_eq!(passengers.adult_women, 1);
        assert_eq!(passengers.child_men, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_seed_with_time_filter() {
    let mut vars = base_env_vars();
    vars.push(("DEPARTURE_TIME_MIN", Some("08:00")));
    vars.push(("DEPARTURE_TIME_MAX", Some("12:00")));

    async_with_vars(vars, async {
        let db = setup_test_db().await;
        seed_from_env(&db).await.unwrap();

        let routes = UserRoutes::find().all(&db).await.unwrap();
        assert_eq!(routes.len(), 1);

        let route = &routes[0];
        assert_eq!(route.departure_time_min, Some("08:00".to_string()));
        assert_eq!(route.departure_time_max, Some("12:00".to_string()));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_seed_discord_webhook_url() {
    async_with_vars(base_env_vars(), async {
        let db = setup_test_db().await;
        seed_from_env(&db).await.unwrap();

        let user = Users::find()
            .filter(users::Column::Email.eq("beta@bus-scraper.local"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            user.discord_webhook_url,
            Some("https://discord.webhook/test".to_string())
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_seed_idempotent_second_call() {
    // Test that calling seed twice doesn't create duplicates
    async_with_vars(base_env_vars(), async {
        let db = setup_test_db().await;

        // First call
        seed_from_env(&db).await.unwrap();

        // Second call - should update, not create new
        seed_from_env(&db).await.unwrap();

        // Still only one user
        let users_list = Users::find().all(&db).await.unwrap();
        assert_eq!(users_list.len(), 1);

        // Still only one route
        let routes = UserRoutes::find().all(&db).await.unwrap();
        assert_eq!(routes.len(), 1);
    })
    .await;
}
