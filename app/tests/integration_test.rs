#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::doc_markdown,
    clippy::uninlined_format_args
)]

use app::db::init_database;
use app::entities::{user_passengers, user_routes, users};
use app::repositories::{get_all_active_user_routes, get_route_state, update_route_state};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

#[tokio::test]
async fn test_multi_user_scenario() {
    let db = init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let route1_id = Uuid::new_v4();
    let route2_id = Uuid::new_v4();

    let user1 = users::ActiveModel {
        id: Set(user1_id),
        email: Set("user1@test.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(Some("https://discord.com/webhook1".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    user1.insert(&db).await.unwrap();

    let user2 = users::ActiveModel {
        id: Set(user2_id),
        email: Set("user2@test.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(600),
        discord_webhook_url: Set(Some("https://discord.com/webhook2".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    user2.insert(&db).await.unwrap();

    let route1 = user_routes::ActiveModel {
        id: Set(route1_id),
        user_id: Set(user1_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-10-12".to_string()),
        date_end: Set("2025-10-19".to_string()),
        departure_time_min: Set(Some("06:00".to_string())),
        departure_time_max: Set(Some("10:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    route1.insert(&db).await.unwrap();

    let route2 = user_routes::ActiveModel {
        id: Set(route2_id),
        user_id: Set(user2_id),
        area_id: Set(1),
        route_id: Set("110".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("064".to_string()),
        date_start: Set("2025-10-15".to_string()),
        date_end: Set("2025-10-20".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    route2.insert(&db).await.unwrap();

    let passengers1 = user_passengers::ActiveModel {
        user_route_id: Set(route1_id),
        adult_men: Set(1),
        adult_women: Set(1),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers1.insert(&db).await.unwrap();

    let passengers2 = user_passengers::ActiveModel {
        user_route_id: Set(route2_id),
        adult_men: Set(2),
        adult_women: Set(0),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers2.insert(&db).await.unwrap();

    let routes = get_all_active_user_routes(&db).await.unwrap();
    assert_eq!(routes.len(), 2);

    let user1_route = routes.iter().find(|r| r.email == "user1@test.com").unwrap();
    assert_eq!(user1_route.scrape_interval_secs, 300);
    assert!(user1_route.notify_on_change_only);
    assert_eq!(user1_route.passengers.total(), 2);
    assert_eq!(user1_route.departure_time_min, Some("06:00".to_string()));

    let user2_route = routes.iter().find(|r| r.email == "user2@test.com").unwrap();
    assert_eq!(user2_route.scrape_interval_secs, 600);
    assert!(!user2_route.notify_on_change_only);
    assert_eq!(user2_route.passengers.total(), 2);
    assert_eq!(user2_route.departure_time_min, None);
}

#[tokio::test]
async fn test_route_state_isolation() {
    let db = init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    let user_id = Uuid::new_v4();
    let route1_id = Uuid::new_v4();
    let route2_id = Uuid::new_v4();

    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set("test@test.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(&db).await.unwrap();

    let route1 = user_routes::ActiveModel {
        id: Set(route1_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-10-12".to_string()),
        date_end: Set("2025-10-19".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    route1.insert(&db).await.unwrap();

    let route2 = user_routes::ActiveModel {
        id: Set(route2_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("110".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("064".to_string()),
        date_start: Set("2025-10-15".to_string()),
        date_end: Set("2025-10-20".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    route2.insert(&db).await.unwrap();

    let passengers1 = user_passengers::ActiveModel {
        user_route_id: Set(route1_id),
        adult_men: Set(1),
        adult_women: Set(0),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers1.insert(&db).await.unwrap();

    let passengers2 = user_passengers::ActiveModel {
        user_route_id: Set(route2_id),
        adult_men: Set(2),
        adult_women: Set(0),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers2.insert(&db).await.unwrap();

    update_route_state(&db, route1_id, "hash1".to_string(), false)
        .await
        .unwrap();
    update_route_state(&db, route2_id, "hash2".to_string(), true)
        .await
        .unwrap();

    let state1 = get_route_state(&db, route1_id).await.unwrap().unwrap();
    let state2 = get_route_state(&db, route2_id).await.unwrap().unwrap();

    assert_eq!(state1.last_seen_hash, "hash1");
    assert_eq!(state2.last_seen_hash, "hash2");

    update_route_state(&db, route1_id, "hash1_updated".to_string(), true)
        .await
        .unwrap();

    let state1 = get_route_state(&db, route1_id).await.unwrap().unwrap();
    let state2 = get_route_state(&db, route2_id).await.unwrap().unwrap();

    assert_eq!(state1.last_seen_hash, "hash1_updated");
    assert_eq!(state2.last_seen_hash, "hash2");
}
