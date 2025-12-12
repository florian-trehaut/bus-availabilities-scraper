//! Integration tests for api_impl module.
//!
//! Tests the extracted business logic from server functions using
//! real SQLite in-memory databases (no mocking internal logic).

use app::api::{UserFormDto, UserRouteFormDto};
use app::api_impl::{
    create_user_impl, create_user_route_impl, delete_user_impl, delete_user_route_impl,
    get_user_routes_impl, get_users_impl, parse_uuid, update_user_impl, update_user_route_impl,
    user_route_to_dto, user_route_with_passengers_to_dto, user_to_dto,
};
use app::entities::{user_passengers, user_routes, users};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use uuid::Uuid;

async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    db
}

async fn create_test_user(db: &DatabaseConnection, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(db).await.expect("Failed to create test user");
    user_id
}

// === UUID Parsing Tests ===

#[test]
fn test_parse_uuid_valid_uuid() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let result = parse_uuid(uuid_str);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), uuid_str);
}

#[test]
fn test_parse_uuid_invalid_format() {
    let result = parse_uuid("not-a-valid-uuid");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid UUID"));
}

#[test]
fn test_parse_uuid_empty_string() {
    let result = parse_uuid("");
    assert!(result.is_err());
}

#[test]
fn test_parse_uuid_partial_uuid() {
    let result = parse_uuid("550e8400-e29b");
    assert!(result.is_err());
}

// === DTO Conversion Tests ===

#[test]
fn test_user_to_dto_converts_all_fields() {
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let user = users::Model {
        id: user_id,
        email: "test@example.com".to_string(),
        enabled: true,
        notify_on_change_only: false,
        scrape_interval_secs: 600,
        discord_webhook_url: Some("https://discord.webhook".to_string()),
        created_at: now,
    };

    let dto = user_to_dto(user);

    assert_eq!(dto.id, user_id.to_string());
    assert_eq!(dto.email, "test@example.com");
    assert!(dto.enabled);
    assert!(!dto.notify_on_change_only);
    assert_eq!(dto.scrape_interval_secs, 600);
    assert_eq!(dto.discord_webhook_url, Some("https://discord.webhook".to_string()));
}

#[test]
fn test_user_route_to_dto_converts_all_fields() {
    let route_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let route = user_routes::Model {
        id: route_id,
        user_id,
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: Some("08:00".to_string()),
        departure_time_max: Some("18:00".to_string()),
        created_at: chrono::Utc::now(),
    };

    let dto = user_route_to_dto(route);

    assert_eq!(dto.id, route_id.to_string());
    assert_eq!(dto.user_id, user_id.to_string());
    assert_eq!(dto.area_id, 100);
    assert_eq!(dto.route_id, "155");
    assert_eq!(dto.departure_time_min, Some("08:00".to_string()));
    assert_eq!(dto.departure_time_max, Some("18:00".to_string()));
}

#[test]
fn test_user_route_with_passengers_to_dto_with_none() {
    let route_id = Uuid::new_v4();
    let route = user_routes::Model {
        id: route_id,
        user_id: Uuid::new_v4(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        created_at: chrono::Utc::now(),
    };

    let dto = user_route_with_passengers_to_dto(route, None);

    // Should default all passengers to 0
    assert_eq!(dto.adult_men, 0);
    assert_eq!(dto.adult_women, 0);
    assert_eq!(dto.child_men, 0);
    assert_eq!(dto.child_women, 0);
    assert_eq!(dto.handicap_adult_men, 0);
    assert_eq!(dto.handicap_adult_women, 0);
    assert_eq!(dto.handicap_child_men, 0);
    assert_eq!(dto.handicap_child_women, 0);
}

#[test]
fn test_user_route_with_passengers_to_dto_with_passengers() {
    let route_id = Uuid::new_v4();
    let route = user_routes::Model {
        id: route_id,
        user_id: Uuid::new_v4(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        created_at: chrono::Utc::now(),
    };

    let passengers = user_passengers::Model {
        user_route_id: route_id,
        adult_men: 2,
        adult_women: 1,
        child_men: 1,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let dto = user_route_with_passengers_to_dto(route, Some(passengers));

    assert_eq!(dto.adult_men, 2);
    assert_eq!(dto.adult_women, 1);
    assert_eq!(dto.child_men, 1);
    assert_eq!(dto.child_women, 0);
}

// === User CRUD Integration Tests ===

#[tokio::test]
async fn test_get_users_impl_empty_database() {
    let db = setup_test_db().await;

    let users = get_users_impl(&db).await.unwrap();

    assert!(users.is_empty());
}

#[tokio::test]
async fn test_get_users_impl_returns_all_users() {
    let db = setup_test_db().await;
    create_test_user(&db, "user1@test.com").await;
    create_test_user(&db, "user2@test.com").await;

    let users = get_users_impl(&db).await.unwrap();

    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_create_user_impl_success() {
    let db = setup_test_db().await;

    let form = UserFormDto {
        email: "newuser@test.com".to_string(),
        enabled: true,
        notify_on_change_only: false,
        scrape_interval_secs: 600,
        discord_webhook_url: Some("https://webhook.url".to_string()),
    };

    let user = create_user_impl(&db, form).await.unwrap();

    assert_eq!(user.email, "newuser@test.com");
    assert!(user.enabled);
    assert!(!user.notify_on_change_only);
    assert_eq!(user.scrape_interval_secs, 600);
    assert_eq!(user.discord_webhook_url, Some("https://webhook.url".to_string()));
}

#[tokio::test]
async fn test_create_user_impl_without_webhook() {
    let db = setup_test_db().await;

    let form = UserFormDto {
        email: "nowebhook@test.com".to_string(),
        enabled: true,
        notify_on_change_only: true,
        scrape_interval_secs: 300,
        discord_webhook_url: None,
    };

    let user = create_user_impl(&db, form).await.unwrap();

    assert!(user.discord_webhook_url.is_none());
}

#[tokio::test]
async fn test_update_user_impl_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "original@test.com").await;

    let form = UserFormDto {
        email: "updated@test.com".to_string(),
        enabled: false,
        notify_on_change_only: true,
        scrape_interval_secs: 900,
        discord_webhook_url: Some("https://new.webhook".to_string()),
    };

    let updated = update_user_impl(&db, user_id, form).await.unwrap();

    assert_eq!(updated.email, "updated@test.com");
    assert!(!updated.enabled);
    assert!(updated.notify_on_change_only);
    assert_eq!(updated.scrape_interval_secs, 900);
}

#[tokio::test]
async fn test_update_user_impl_not_found() {
    let db = setup_test_db().await;
    let non_existent_id = Uuid::new_v4();

    let form = UserFormDto {
        email: "test@test.com".to_string(),
        enabled: true,
        notify_on_change_only: true,
        scrape_interval_secs: 300,
        discord_webhook_url: None,
    };

    let result = update_user_impl(&db, non_existent_id, form).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Not found") || err.contains("not found"));
}

#[tokio::test]
async fn test_delete_user_impl_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "todelete@test.com").await;

    let result = delete_user_impl(&db, user_id).await;
    assert!(result.is_ok());

    // Verify user is deleted
    let users = get_users_impl(&db).await.unwrap();
    assert!(users.is_empty());
}

// === User Route CRUD Integration Tests ===

#[tokio::test]
async fn test_get_user_routes_impl_empty() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "noroutes@test.com").await;

    let routes = get_user_routes_impl(&db, user_id).await.unwrap();

    assert!(routes.is_empty());
}

#[tokio::test]
async fn test_create_user_route_impl_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "withroute@test.com").await;

    let form = UserRouteFormDto {
        user_id: user_id.to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: Some("08:00".to_string()),
        departure_time_max: Some("18:00".to_string()),
        adult_men: 2,
        adult_women: 1,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let route = create_user_route_impl(&db, form).await.unwrap();

    assert_eq!(route.user_id, user_id.to_string());
    assert_eq!(route.area_id, 100);
    assert_eq!(route.route_id, "155");
    assert_eq!(route.departure_station, "001");
    assert_eq!(route.arrival_station, "064");
}

#[tokio::test]
async fn test_create_user_route_impl_invalid_user_id() {
    let db = setup_test_db().await;

    let form = UserRouteFormDto {
        user_id: "not-a-uuid".to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        adult_men: 1,
        adult_women: 0,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let result = create_user_route_impl(&db, form).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_user_routes_impl_with_passengers() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "withpassengers@test.com").await;

    // Create a route with passengers
    let form = UserRouteFormDto {
        user_id: user_id.to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        adult_men: 3,
        adult_women: 2,
        child_men: 1,
        child_women: 1,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    create_user_route_impl(&db, form).await.unwrap();

    let routes = get_user_routes_impl(&db, user_id).await.unwrap();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].adult_men, 3);
    assert_eq!(routes[0].adult_women, 2);
    assert_eq!(routes[0].child_men, 1);
    assert_eq!(routes[0].child_women, 1);
}

#[tokio::test]
async fn test_update_user_route_impl_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "updateroute@test.com").await;

    // Create initial route
    let create_form = UserRouteFormDto {
        user_id: user_id.to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        adult_men: 1,
        adult_women: 0,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let route = create_user_route_impl(&db, create_form).await.unwrap();
    let route_uuid = parse_uuid(&route.id).unwrap();

    // Update the route
    let update_form = UserRouteFormDto {
        user_id: user_id.to_string(),
        area_id: 200,
        route_id: "200".to_string(),
        departure_station: "010".to_string(),
        arrival_station: "099".to_string(),
        date_start: "20250201".to_string(),
        date_end: "20250228".to_string(),
        departure_time_min: Some("06:00".to_string()),
        departure_time_max: Some("22:00".to_string()),
        adult_men: 2,
        adult_women: 2,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let updated = update_user_route_impl(&db, route_uuid, update_form).await.unwrap();

    assert_eq!(updated.area_id, 200);
    assert_eq!(updated.route_id, "200");
    assert_eq!(updated.departure_station, "010");
}

#[tokio::test]
async fn test_update_user_route_impl_not_found() {
    let db = setup_test_db().await;
    let non_existent_id = Uuid::new_v4();

    let form = UserRouteFormDto {
        user_id: Uuid::new_v4().to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        adult_men: 1,
        adult_women: 0,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let result = update_user_route_impl(&db, non_existent_id, form).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Not found") || err.contains("not found"));
}

#[tokio::test]
async fn test_delete_user_route_impl_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "deleteroute@test.com").await;

    // Create route
    let form = UserRouteFormDto {
        user_id: user_id.to_string(),
        area_id: 100,
        route_id: "155".to_string(),
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_start: "20250101".to_string(),
        date_end: "20250107".to_string(),
        departure_time_min: None,
        departure_time_max: None,
        adult_men: 1,
        adult_women: 0,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    };

    let route = create_user_route_impl(&db, form).await.unwrap();
    let route_uuid = parse_uuid(&route.id).unwrap();

    // Delete route
    let result = delete_user_route_impl(&db, route_uuid).await;
    assert!(result.is_ok());

    // Verify it's deleted
    let routes = get_user_routes_impl(&db, user_id).await.unwrap();
    assert!(routes.is_empty());
}

#[tokio::test]
async fn test_multiple_routes_per_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "multiroute@test.com").await;

    // Create multiple routes
    for i in 1..=3 {
        let form = UserRouteFormDto {
            user_id: user_id.to_string(),
            area_id: 100,
            route_id: format!("{}", 150 + i),
            departure_station: "001".to_string(),
            arrival_station: "064".to_string(),
            date_start: "20250101".to_string(),
            date_end: "20250107".to_string(),
            departure_time_min: None,
            departure_time_max: None,
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };
        create_user_route_impl(&db, form).await.unwrap();
    }

    let routes = get_user_routes_impl(&db, user_id).await.unwrap();

    assert_eq!(routes.len(), 3);
}
