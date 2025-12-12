#![allow(clippy::unwrap_used)]

//! SeaORM entity relationship tests
//!
//! Tests for:
//! 1. Related entity queries using find_related()
//! 2. Cascade DELETE behavior (FK constraints)
//! 3. Relation definitions (has_many, has_one, belongs_to)

use app::db::init_database;
use app::entities::{prelude::*, route_states, user_passengers, user_routes, users};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};
use uuid::Uuid;

/// Test helper: setup in-memory database with migrations
async fn setup_test_db() -> sea_orm::DatabaseConnection {
    let db = init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

/// Test helper: create test user
async fn create_test_user(db: &sea_orm::DatabaseConnection, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(Some("https://discord.com/webhook".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(db).await.unwrap();
    user_id
}

/// Test helper: create test route
async fn create_test_route(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    route_name: &str,
) -> Uuid {
    let route_id = Uuid::new_v4();
    let route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set(route_name.to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-10-12".to_string()),
        date_end: Set("2025-10-19".to_string()),
        departure_time_min: Set(Some("06:00".to_string())),
        departure_time_max: Set(Some("10:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    route.insert(db).await.unwrap();
    route_id
}

/// Test helper: create test passengers
async fn create_test_passengers(db: &sea_orm::DatabaseConnection, user_route_id: Uuid) {
    let passengers = user_passengers::ActiveModel {
        user_route_id: Set(user_route_id),
        adult_men: Set(1),
        adult_women: Set(1),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers.insert(db).await.unwrap();
}

/// Test helper: create test route state
async fn create_test_route_state(
    db: &sea_orm::DatabaseConnection,
    user_route_id: Uuid,
    hash: &str,
) {
    let state = route_states::ActiveModel {
        user_route_id: Set(user_route_id),
        last_seen_hash: Set(hash.to_string()),
        last_check: Set(Some(chrono::Utc::now())),
        total_checks: Set(1),
        total_alerts: Set(0),
    };
    state.insert(db).await.unwrap();
}

// =============================================================================
// RELATED ENTITY QUERIES
// =============================================================================

#[tokio::test]
async fn test_user_has_many_routes() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;

    create_test_route(&db, user_id, "155").await;
    create_test_route(&db, user_id, "110").await;
    create_test_route(&db, user_id, "201").await;

    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();

    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();

    assert_eq!(routes.len(), 3);
    assert!(routes.iter().any(|r| r.route_id == "155"));
    assert!(routes.iter().any(|r| r.route_id == "110"));
    assert!(routes.iter().any(|r| r.route_id == "201"));
}

#[tokio::test]
async fn test_route_belongs_to_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let user = route.find_related(Users).one(&db).await.unwrap().unwrap();

    assert_eq!(user.id, user_id);
    assert_eq!(user.email, "user@test.com");
}

#[tokio::test]
async fn test_route_has_one_passengers() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_passengers(&db, route_id).await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let passengers = route
        .find_related(UserPassengers)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(passengers.user_route_id, route_id);
    assert_eq!(passengers.adult_men, 1);
    assert_eq!(passengers.adult_women, 1);
}

#[tokio::test]
async fn test_route_has_one_route_state() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_route_state(&db, route_id, "hash123").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let state = route
        .find_related(RouteStates)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(state.user_route_id, route_id);
    assert_eq!(state.last_seen_hash, "hash123");
}

#[tokio::test]
async fn test_passengers_belongs_to_route() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_passengers(&db, route_id).await;

    let passengers = UserPassengers::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let route = passengers
        .find_related(UserRoutes)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(route.id, route_id);
    assert_eq!(route.route_id, "155");
}

#[tokio::test]
async fn test_route_state_belongs_to_route() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_route_state(&db, route_id, "hash123").await;

    let state = RouteStates::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let route = state
        .find_related(UserRoutes)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(route.id, route_id);
    assert_eq!(route.route_id, "155");
}

// =============================================================================
// CASCADE DELETE BEHAVIOR
// =============================================================================

#[tokio::test]
async fn test_cascade_delete_user_routes() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;

    let route1_id = create_test_route(&db, user_id, "155").await;
    let route2_id = create_test_route(&db, user_id, "110").await;

    create_test_passengers(&db, route1_id).await;
    create_test_passengers(&db, route2_id).await;
    create_test_route_state(&db, route1_id, "hash1").await;
    create_test_route_state(&db, route2_id, "hash2").await;

    // Verify data exists
    assert_eq!(UserRoutes::find().all(&db).await.unwrap().len(), 2);
    assert_eq!(UserPassengers::find().all(&db).await.unwrap().len(), 2);
    assert_eq!(RouteStates::find().all(&db).await.unwrap().len(), 2);

    // Delete user
    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();
    user.delete(&db).await.unwrap();

    // Verify cascade delete
    assert_eq!(UserRoutes::find().all(&db).await.unwrap().len(), 0);
    assert_eq!(UserPassengers::find().all(&db).await.unwrap().len(), 0);
    assert_eq!(RouteStates::find().all(&db).await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_cascade_delete_route_only_affects_own_children() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;

    let route1_id = create_test_route(&db, user_id, "155").await;
    let route2_id = create_test_route(&db, user_id, "110").await;

    create_test_passengers(&db, route1_id).await;
    create_test_passengers(&db, route2_id).await;
    create_test_route_state(&db, route1_id, "hash1").await;
    create_test_route_state(&db, route2_id, "hash2").await;

    // Delete route1
    let route1 = UserRoutes::find_by_id(route1_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    route1.delete(&db).await.unwrap();

    // Verify only route1 and its children are deleted
    assert_eq!(UserRoutes::find().all(&db).await.unwrap().len(), 1);
    assert_eq!(UserPassengers::find().all(&db).await.unwrap().len(), 1);
    assert_eq!(RouteStates::find().all(&db).await.unwrap().len(), 1);

    // Verify route2 data still exists
    let remaining_route = UserRoutes::find().one(&db).await.unwrap().unwrap();
    assert_eq!(remaining_route.id, route2_id);

    let remaining_passengers = UserPassengers::find().one(&db).await.unwrap().unwrap();
    assert_eq!(remaining_passengers.user_route_id, route2_id);

    let remaining_state = RouteStates::find().one(&db).await.unwrap().unwrap();
    assert_eq!(remaining_state.user_route_id, route2_id);
}

#[tokio::test]
async fn test_cascade_delete_preserves_other_users() {
    let db = setup_test_db().await;

    let user1_id = create_test_user(&db, "user1@test.com").await;
    let user2_id = create_test_user(&db, "user2@test.com").await;

    let route1_id = create_test_route(&db, user1_id, "155").await;
    let route2_id = create_test_route(&db, user2_id, "110").await;

    create_test_passengers(&db, route1_id).await;
    create_test_passengers(&db, route2_id).await;

    // Delete user1
    let user1 = Users::find_by_id(user1_id).one(&db).await.unwrap().unwrap();
    user1.delete(&db).await.unwrap();

    // Verify user2 data is untouched
    assert_eq!(Users::find().all(&db).await.unwrap().len(), 1);
    assert_eq!(UserRoutes::find().all(&db).await.unwrap().len(), 1);
    assert_eq!(UserPassengers::find().all(&db).await.unwrap().len(), 1);

    let remaining_user = Users::find().one(&db).await.unwrap().unwrap();
    assert_eq!(remaining_user.id, user2_id);
    assert_eq!(remaining_user.email, "user2@test.com");
}

// =============================================================================
// RELATION DEFINITIONS
// =============================================================================

#[tokio::test]
async fn test_user_routes_relation_is_has_many() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;

    // Create no routes
    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();
    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(routes.len(), 0);

    // Create one route
    create_test_route(&db, user_id, "155").await;
    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(routes.len(), 1);

    // Create multiple routes
    create_test_route(&db, user_id, "110").await;
    create_test_route(&db, user_id, "201").await;
    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(routes.len(), 3);
}

#[tokio::test]
async fn test_route_passengers_relation_is_has_one() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    // No passengers initially
    let passengers = route.find_related(UserPassengers).one(&db).await.unwrap();
    assert!(passengers.is_none());

    // Create passengers
    create_test_passengers(&db, route_id).await;
    let passengers = route.find_related(UserPassengers).one(&db).await.unwrap();
    assert!(passengers.is_some());
}

#[tokio::test]
async fn test_route_state_relation_is_has_one() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    // No state initially
    let state = route.find_related(RouteStates).one(&db).await.unwrap();
    assert!(state.is_none());

    // Create state
    create_test_route_state(&db, route_id, "hash123").await;
    let state = route.find_related(RouteStates).one(&db).await.unwrap();
    assert!(state.is_some());
}

#[tokio::test]
async fn test_bidirectional_user_route_relation() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    // Forward: User -> Routes
    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();
    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].id, route_id);

    // Reverse: Route -> User
    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let user_from_route = route.find_related(Users).one(&db).await.unwrap().unwrap();
    assert_eq!(user_from_route.id, user_id);
}

#[tokio::test]
async fn test_bidirectional_route_passengers_relation() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_passengers(&db, route_id).await;

    // Forward: Route -> Passengers
    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let passengers = route
        .find_related(UserPassengers)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(passengers.user_route_id, route_id);

    // Reverse: Passengers -> Route
    let route_from_passengers = passengers
        .find_related(UserRoutes)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(route_from_passengers.id, route_id);
}

#[tokio::test]
async fn test_bidirectional_route_state_relation() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_route_state(&db, route_id, "hash123").await;

    // Forward: Route -> State
    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let state = route
        .find_related(RouteStates)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.user_route_id, route_id);

    // Reverse: State -> Route
    let route_from_state = state
        .find_related(UserRoutes)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(route_from_state.id, route_id);
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[tokio::test]
async fn test_user_with_no_routes() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;

    let user = Users::find_by_id(user_id).one(&db).await.unwrap().unwrap();
    let routes = user.find_related(UserRoutes).all(&db).await.unwrap();

    assert_eq!(routes.len(), 0);
}

#[tokio::test]
async fn test_route_with_no_passengers() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let passengers = route.find_related(UserPassengers).one(&db).await.unwrap();

    assert!(passengers.is_none());
}

#[tokio::test]
async fn test_route_with_no_state() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, "user@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;

    let route = UserRoutes::find_by_id(route_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let state = route.find_related(RouteStates).one(&db).await.unwrap();

    assert!(state.is_none());
}

#[tokio::test]
async fn test_multiple_users_no_cross_contamination() {
    let db = setup_test_db().await;

    let user1_id = create_test_user(&db, "user1@test.com").await;
    let user2_id = create_test_user(&db, "user2@test.com").await;

    create_test_route(&db, user1_id, "155").await;
    create_test_route(&db, user1_id, "110").await;
    create_test_route(&db, user2_id, "201").await;

    let user1 = Users::find_by_id(user1_id).one(&db).await.unwrap().unwrap();
    let user1_routes = user1.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(user1_routes.len(), 2);
    assert!(user1_routes.iter().all(|r| r.user_id == user1_id));

    let user2 = Users::find_by_id(user2_id).one(&db).await.unwrap().unwrap();
    let user2_routes = user2.find_related(UserRoutes).all(&db).await.unwrap();
    assert_eq!(user2_routes.len(), 1);
    assert!(user2_routes.iter().all(|r| r.user_id == user2_id));
}

// =============================================================================
// REPOSITORY INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_get_all_active_user_routes_missing_passengers_error() {
    use app::repositories::get_all_active_user_routes;

    let db = setup_test_db().await;

    // Create an enabled user
    let user_id = create_test_user(&db, "missing-passengers@test.com").await;

    // Create a route WITHOUT passengers - this should cause an error
    create_test_route(&db, user_id, "999").await;

    // This should return an error because passengers are missing for the route
    let result = get_all_active_user_routes(&db).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("No passengers found"),
        "Expected 'No passengers found' error, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_get_all_active_user_routes_success_with_passengers() {
    use app::repositories::get_all_active_user_routes;

    let db = setup_test_db().await;

    // Create user with route AND passengers
    let user_id = create_test_user(&db, "has-passengers@test.com").await;
    let route_id = create_test_route(&db, user_id, "155").await;
    create_test_passengers(&db, route_id).await;

    let result = get_all_active_user_routes(&db).await;

    assert!(result.is_ok());
    let routes = result.unwrap();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].email, "has-passengers@test.com");
    assert_eq!(routes[0].route_id, "155");
    assert_eq!(routes[0].passengers.adult_men, 1);
    assert_eq!(routes[0].passengers.adult_women, 1);
}
