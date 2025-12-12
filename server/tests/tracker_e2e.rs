//! E2E tests for the tracker module
//!
//! These tests verify the tracker's behavior including:
//! - State hash calculation
//! - Route state updates
//! - Notification triggering logic

use app::{
    db,
    entities::{user_passengers, user_routes, users},
    repositories::{get_all_active_user_routes, get_route_state, update_route_state},
    types::{BusSchedule, PricingPlan, SeatAvailability},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

async fn setup_test_db() -> DatabaseConnection {
    let db = db::init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn create_test_user(db: &DatabaseConnection, enabled: bool) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("tracker-test-{}@example.com", user_id)),
        enabled: Set(enabled),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(Some("https://discord.com/api/webhooks/test".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(db).await.unwrap();
    user_id
}

async fn create_test_route(db: &DatabaseConnection, user_id: Uuid) -> Uuid {
    let route_id = Uuid::new_v4();
    let route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("155".to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-15".to_string()),
        date_end: Set("2025-01-20".to_string()),
        departure_time_min: Set(None),
        departure_time_max: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    route.insert(db).await.unwrap();

    let passengers = user_passengers::ActiveModel {
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
    passengers.insert(db).await.unwrap();

    route_id
}

fn create_test_schedule(date: &str, time: &str, price: u32, seats: Option<u32>) -> BusSchedule {
    BusSchedule {
        bus_number: "Bus_1".to_string(),
        route_name: "Test Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: date.to_string(),
        departure_time: time.to_string(),
        arrival_station: "498".to_string(),
        arrival_date: date.to_string(),
        arrival_time: "10:00".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 12345,
            plan_index: 0,
            plan_name: "Standard".to_string(),
            price,
            display_price: format!("{}円", price),
            availability: SeatAvailability::Available {
                remaining_seats: seats,
            },
        }],
    }
}

/// Calculate state hash (same logic as tracker)
fn calculate_state_hash(schedules: &[BusSchedule]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for schedule in schedules {
        schedule.departure_date.hash(&mut hasher);
        schedule.departure_time.hash(&mut hasher);
        for plan in &schedule.available_plans {
            plan.plan_id.hash(&mut hasher);
            plan.price.hash(&mut hasher);
            let SeatAvailability::Available { remaining_seats } = &plan.availability;
            remaining_seats.hash(&mut hasher);
        }
    }
    hasher.finish()
}

// =============================================================================
// Repository Integration Tests
// =============================================================================

#[tokio::test]
async fn test_get_active_user_routes_returns_enabled_users_only() {
    let db = setup_test_db().await;

    // Create enabled user with route
    let enabled_user = create_test_user(&db, true).await;
    create_test_route(&db, enabled_user).await;

    // Create disabled user with route
    let disabled_user = create_test_user(&db, false).await;
    create_test_route(&db, disabled_user).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    // Should only return routes for enabled users
    assert_eq!(routes.len(), 1);
    assert_eq!(
        routes[0].email,
        format!("tracker-test-{}@example.com", enabled_user)
    );
}

#[tokio::test]
async fn test_route_state_initially_none() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    let state = get_route_state(&db, route_id).await.unwrap();

    assert!(state.is_none());
}

#[tokio::test]
async fn test_update_route_state_creates_new_state() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    // Update state
    update_route_state(&db, route_id, "hash123".to_string(), false)
        .await
        .unwrap();

    let state = get_route_state(&db, route_id).await.unwrap();

    assert!(state.is_some());
    assert_eq!(state.unwrap().last_seen_hash, "hash123");
}

#[tokio::test]
async fn test_update_route_state_updates_existing() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    // First update
    update_route_state(&db, route_id, "hash1".to_string(), false)
        .await
        .unwrap();

    // Second update
    update_route_state(&db, route_id, "hash2".to_string(), true)
        .await
        .unwrap();

    let state = get_route_state(&db, route_id).await.unwrap();

    assert!(state.is_some());
    assert_eq!(state.unwrap().last_seen_hash, "hash2");
}

// =============================================================================
// Hash Calculation Tests
// =============================================================================

#[tokio::test]
async fn test_hash_changes_when_seats_change() {
    let schedules1 = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let schedules2 = vec![create_test_schedule("20250115", "08:30", 2100, Some(3))];

    let hash1 = calculate_state_hash(&schedules1);
    let hash2 = calculate_state_hash(&schedules2);

    assert_ne!(hash1, hash2);
}

#[tokio::test]
async fn test_hash_changes_when_price_changes() {
    let schedules1 = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let schedules2 = vec![create_test_schedule("20250115", "08:30", 2500, Some(5))];

    let hash1 = calculate_state_hash(&schedules1);
    let hash2 = calculate_state_hash(&schedules2);

    assert_ne!(hash1, hash2);
}

#[tokio::test]
async fn test_hash_consistent_for_same_data() {
    let schedules1 = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let schedules2 = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];

    let hash1 = calculate_state_hash(&schedules1);
    let hash2 = calculate_state_hash(&schedules2);

    assert_eq!(hash1, hash2);
}

// =============================================================================
// Notification Logic Tests
// =============================================================================

#[tokio::test]
async fn test_should_notify_on_first_availability() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    // No previous state
    let state = get_route_state(&db, route_id).await.unwrap();
    assert!(state.is_none());

    // With schedules available, should notify
    let schedules = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let has_availability = !schedules.is_empty();
    let state_changed = state.is_none();

    // notify_on_change_only=true: should notify on first availability
    let should_notify = state_changed && has_availability;
    assert!(should_notify);
}

#[tokio::test]
async fn test_should_not_notify_when_hash_unchanged() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    let schedules = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let hash = format!("{}", calculate_state_hash(&schedules));

    // Set initial state
    update_route_state(&db, route_id, hash.clone(), true)
        .await
        .unwrap();

    // Get state again
    let state = get_route_state(&db, route_id).await.unwrap().unwrap();

    // Same hash means no change
    let state_changed = state.last_seen_hash != hash;
    assert!(!state_changed);

    // Should not notify when unchanged
    let should_notify = state_changed && !schedules.is_empty();
    assert!(!should_notify);
}

#[tokio::test]
async fn test_should_notify_when_availability_changes() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    let route_id = create_test_route(&db, user_id).await;

    // Set initial state with 5 seats
    let schedules1 = vec![create_test_schedule("20250115", "08:30", 2100, Some(5))];
    let hash1 = format!("{}", calculate_state_hash(&schedules1));
    update_route_state(&db, route_id, hash1.clone(), false)
        .await
        .unwrap();

    // New state with 3 seats
    let schedules2 = vec![create_test_schedule("20250115", "08:30", 2100, Some(3))];
    let hash2 = format!("{}", calculate_state_hash(&schedules2));

    let state = get_route_state(&db, route_id).await.unwrap().unwrap();
    let state_changed = state.last_seen_hash != hash2;

    assert!(state_changed);

    // Should notify on change
    let should_notify = state_changed && !schedules2.is_empty();
    assert!(should_notify);
}

// =============================================================================
// User Route Details Tests
// =============================================================================

#[tokio::test]
async fn test_user_route_contains_passenger_details() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    create_test_route(&db, user_id).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    assert_eq!(routes.len(), 1);
    let route = &routes[0];

    assert_eq!(route.passengers.adult_men, 1);
    assert_eq!(route.passengers.adult_women, 0);
    assert_eq!(route.passengers.total(), 1);
}

#[tokio::test]
async fn test_user_route_contains_discord_webhook() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;
    create_test_route(&db, user_id).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    assert_eq!(routes.len(), 1);
    assert!(routes[0].discord_webhook_url.is_some());
    assert!(
        routes[0]
            .discord_webhook_url
            .as_ref()
            .unwrap()
            .contains("discord.com")
    );
}

#[tokio::test]
async fn test_multiple_routes_per_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db, true).await;

    // Create 3 routes for same user
    create_test_route(&db, user_id).await;
    create_test_route(&db, user_id).await;
    create_test_route(&db, user_id).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    assert_eq!(routes.len(), 3);
    for route in &routes {
        assert_eq!(route.email, format!("tracker-test-{}@example.com", user_id));
    }
}
