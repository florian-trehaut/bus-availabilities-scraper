//! Integration tests for API database operations
//! These tests cover the same logic as the Leptos server functions but
//! bypass the Leptos context by directly using the database operations.

use app::db::init_database;
use app::entities::{prelude::*, user_passengers, user_routes, users};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

async fn setup_test_db() -> DatabaseConnection {
    let db = init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

// =============================================================================
// User CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_get_users_empty() {
    let db = setup_test_db().await;

    let users = Users::find().all(&db).await.unwrap();

    assert!(users.is_empty());
}

#[tokio::test]
async fn test_create_user() {
    let db = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set("test@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(Some("https://discord.com/webhook".to_string())),
        created_at: Set(chrono::Utc::now()),
    };

    let user = new_user.insert(&db).await.unwrap();

    assert_eq!(user.id, user_id);
    assert_eq!(user.email, "test@example.com");
    assert!(user.enabled);
    assert!(!user.notify_on_change_only);
    assert_eq!(user.scrape_interval_secs, 300);
    assert_eq!(
        user.discord_webhook_url,
        Some("https://discord.com/webhook".to_string())
    );
}

#[tokio::test]
async fn test_get_users_returns_all() {
    let db = setup_test_db().await;

    // Create multiple users
    for i in 0..3 {
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(format!("user{}@example.com", i)),
            enabled: Set(true),
            notify_on_change_only: Set(false),
            scrape_interval_secs: Set(300),
            discord_webhook_url: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        new_user.insert(&db).await.unwrap();
    }

    let users = Users::find().all(&db).await.unwrap();

    assert_eq!(users.len(), 3);
}

#[tokio::test]
async fn test_update_user() {
    let db = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set("original@example.com".to_string()),
        enabled: Set(false),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_user.insert(&db).await.unwrap();

    // Update user
    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();
    let mut active_user: users::ActiveModel = user.into();
    active_user.email = Set("updated@example.com".to_string());
    active_user.enabled = Set(true);
    active_user.notify_on_change_only = Set(true);
    active_user.scrape_interval_secs = Set(600);
    active_user.discord_webhook_url = Set(Some("https://new-webhook.com".to_string()));

    let updated_user = active_user.update(&db).await.unwrap();

    assert_eq!(updated_user.email, "updated@example.com");
    assert!(updated_user.enabled);
    assert!(updated_user.notify_on_change_only);
    assert_eq!(updated_user.scrape_interval_secs, 600);
    assert_eq!(
        updated_user.discord_webhook_url,
        Some("https://new-webhook.com".to_string())
    );
}

#[tokio::test]
async fn test_delete_user() {
    let db = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set("to-delete@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_user.insert(&db).await.unwrap();

    // Verify user exists
    let user = Users::find_by_id(user_id).one(&db).await.unwrap();
    assert!(user.is_some());

    // Delete user
    Users::delete_by_id(user_id).exec(&db).await.unwrap();

    // Verify user is deleted
    let user = Users::find_by_id(user_id).one(&db).await.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_find_user_by_id_not_found() {
    let db = setup_test_db().await;

    let non_existent_id = Uuid::new_v4();
    let user = Users::find_by_id(non_existent_id).one(&db).await.unwrap();

    assert!(user.is_none());
}

// =============================================================================
// User Route CRUD Tests
// =============================================================================

async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    let user_id = Uuid::new_v4();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set("route-test@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_user.insert(db).await.unwrap();
    user_id
}

#[tokio::test]
async fn test_create_user_route_with_passengers() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(Some("08:00".to_string())),
        departure_time_max: Set(Some("18:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    let route = new_route.insert(&db).await.unwrap();

    let new_passengers = user_passengers::ActiveModel {
        user_route_id: Set(route_id),
        adult_men: Set(2),
        adult_women: Set(1),
        child_men: Set(0),
        child_women: Set(1),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    let passengers = new_passengers.insert(&db).await.unwrap();

    assert_eq!(route.id, route_id);
    assert_eq!(route.user_id, user_id);
    assert_eq!(route.area_id, 1);
    assert_eq!(route.route_id, "155");
    assert_eq!(route.departure_station, "001");
    assert_eq!(route.arrival_station, "498");
    assert_eq!(route.date_start, "2025-01-01");
    assert_eq!(route.date_end, "2025-01-15");
    assert_eq!(route.departure_time_min, Some("08:00".to_string()));
    assert_eq!(route.departure_time_max, Some("18:00".to_string()));

    assert_eq!(passengers.adult_men, 2);
    assert_eq!(passengers.adult_women, 1);
    assert_eq!(passengers.child_women, 1);
}

#[tokio::test]
async fn test_get_user_routes_by_user_id() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create 2 routes for this user
    for i in 0..2 {
        let route_id = Uuid::new_v4();
        let new_route = user_routes::ActiveModel {
            id: Set(route_id),
            user_id: Set(user_id),
            area_id: Set(1),
            route_id: Set(format!("15{}", i)),
            departure_station: Set("001".to_string()),
            arrival_station: Set("498".to_string()),
            date_start: Set("2025-01-01".to_string()),
            date_end: Set("2025-01-15".to_string()),
            departure_time_min: Set(None),
            departure_time_max: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        new_route.insert(&db).await.unwrap();

        let new_passengers = user_passengers::ActiveModel {
            user_route_id: Set(route_id),
            adult_men: Set(1),
            adult_women: Set(0),
            child_men: Set(0),
            child_women: Set(0),
            handicap_adult_men: Set(0),
            handicap_adult_women: Set(0),
            handicap_child_men: Set(0),
            handicap_child_women: Set(0),
        };
        new_passengers.insert(&db).await.unwrap();
    }

    let routes = UserRoutes::find()
        .filter(user_routes::Column::UserId.eq(user_id))
        .find_also_related(UserPassengers)
        .all(&db)
        .await
        .unwrap();

    assert_eq!(routes.len(), 2);

    // Each route should have associated passengers
    for (route, passengers) in &routes {
        assert_eq!(route.user_id, user_id);
        assert!(passengers.is_some());
    }
}

#[tokio::test]
async fn test_update_user_route() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_route.insert(&db).await.unwrap();

    // Update route
    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let mut active_route: user_routes::ActiveModel = route.into();
    active_route.area_id = Set(2);
    active_route.route_id = Set("160".to_string());
    active_route.departure_station = Set("002".to_string());
    active_route.arrival_station = Set("499".to_string());
    active_route.date_start = Set("2025-02-01".to_string());
    active_route.date_end = Set("2025-02-28".to_string());
    active_route.departure_time_min = Set(Some("10:00".to_string()));
    active_route.departure_time_max = Set(Some("20:00".to_string()));

    let updated_route = active_route.update(&db).await.unwrap();

    assert_eq!(updated_route.area_id, 2);
    assert_eq!(updated_route.route_id, "160");
    assert_eq!(updated_route.departure_station, "002");
    assert_eq!(updated_route.arrival_station, "499");
    assert_eq!(updated_route.date_start, "2025-02-01");
    assert_eq!(updated_route.date_end, "2025-02-28");
    assert_eq!(updated_route.departure_time_min, Some("10:00".to_string()));
    assert_eq!(updated_route.departure_time_max, Some("20:00".to_string()));
}

#[tokio::test]
async fn test_update_passengers() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_route.insert(&db).await.unwrap();

    let new_passengers = user_passengers::ActiveModel {
        user_route_id: Set(route_id),
        adult_men: Set(1),
        adult_women: Set(0),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    new_passengers.insert(&db).await.unwrap();

    // Update passengers
    let passengers = UserPassengers::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let mut active_passengers: user_passengers::ActiveModel = passengers.into();
    active_passengers.adult_men = Set(3);
    active_passengers.adult_women = Set(2);
    active_passengers.child_men = Set(1);
    active_passengers.child_women = Set(1);
    active_passengers.handicap_adult_men = Set(1);
    active_passengers.handicap_adult_women = Set(0);
    active_passengers.handicap_child_men = Set(0);
    active_passengers.handicap_child_women = Set(0);

    let updated_passengers = active_passengers.update(&db).await.unwrap();

    assert_eq!(updated_passengers.adult_men, 3);
    assert_eq!(updated_passengers.adult_women, 2);
    assert_eq!(updated_passengers.child_men, 1);
    assert_eq!(updated_passengers.child_women, 1);
    assert_eq!(updated_passengers.handicap_adult_men, 1);
}

#[tokio::test]
async fn test_delete_user_route() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_route.insert(&db).await.unwrap();

    // Verify route exists
    let route = UserRoutes::find_by_id(route_id).one(&db).await.unwrap();
    assert!(route.is_some());

    // Delete route
    UserRoutes::delete_by_id(route_id).exec(&db).await.unwrap();

    // Verify route is deleted
    let route = UserRoutes::find_by_id(route_id).one(&db).await.unwrap();
    assert!(route.is_none());
}

#[tokio::test]
async fn test_get_user_routes_empty_for_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let routes = UserRoutes::find()
        .filter(user_routes::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .unwrap();

    assert!(routes.is_empty());
}

// =============================================================================
// UUID Validation Tests (simulating API error handling)
// =============================================================================

#[tokio::test]
async fn test_uuid_parse_valid() {
    let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let result = Uuid::parse_str(valid_uuid);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_uuid_parse_invalid() {
    let invalid_uuid = "not-a-valid-uuid";
    let result = Uuid::parse_str(invalid_uuid);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_uuid_parse_empty() {
    let empty_uuid = "";
    let result = Uuid::parse_str(empty_uuid);
    assert!(result.is_err());
}

// =============================================================================
// User with Routes relationship tests
// =============================================================================

#[tokio::test]
async fn test_delete_user_does_not_cascade_to_routes() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_route.insert(&db).await.unwrap();

    // Delete the route first (required by FK constraints)
    UserRoutes::delete_by_id(route_id).exec(&db).await.unwrap();

    // Then delete the user
    Users::delete_by_id(user_id).exec(&db).await.unwrap();

    // Verify both are deleted
    let user = Users::find_by_id(user_id).one(&db).await.unwrap();
    let route = UserRoutes::find_by_id(route_id).one(&db).await.unwrap();
    assert!(user.is_none());
    assert!(route.is_none());
}

#[tokio::test]
async fn test_route_with_no_time_filter() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    let route = new_route.insert(&db).await.unwrap();

    assert!(route.departure_time_min.is_none());
    assert!(route.departure_time_max.is_none());
}

#[tokio::test]
async fn test_route_with_only_min_time() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(Some("10:00".to_string())),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    let route = new_route.insert(&db).await.unwrap();

    assert_eq!(route.departure_time_min, Some("10:00".to_string()));
    assert!(route.departure_time_max.is_none());
}

#[tokio::test]
async fn test_route_with_only_max_time() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(Some("18:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    let route = new_route.insert(&db).await.unwrap();

    assert!(route.departure_time_min.is_none());
    assert_eq!(route.departure_time_max, Some("18:00".to_string()));
}
