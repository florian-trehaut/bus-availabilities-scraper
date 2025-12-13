//! Integration tests for the tracker module
//!
//! These tests verify:
//! - Station cache population from API
//! - `ScrapeRequest` building from `UserRouteWithDetails`
//! - `NotificationContext` building with station name resolution
//! - Tracker initialization with/without routes
//! - State hash edge cases
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::too_many_arguments,
    clippy::useless_vec
)]

use app::{
    db,
    entities::{user_passengers, user_routes, users},
    notifier::NotificationContext,
    repositories::{PassengerDetails, UserRouteWithDetails, get_all_active_user_routes},
    scraper::BusScraper,
    types::{DateRange, PassengerCount, ScrapeRequest, TimeFilter},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

// Type alias from tracker.rs
type StationCache = Arc<tokio::sync::RwLock<HashMap<String, String>>>;

// Re-implement calculate_state_hash for testing (same logic as tracker.rs)
fn calculate_state_hash(schedules: &[app::types::BusSchedule]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for schedule in schedules {
        schedule.departure_date.hash(&mut hasher);
        schedule.departure_time.hash(&mut hasher);
        for plan in &schedule.available_plans {
            plan.plan_id.hash(&mut hasher);
            plan.price.hash(&mut hasher);
            let app::types::SeatAvailability::Available { remaining_seats } = &plan.availability;
            remaining_seats.hash(&mut hasher);
        }
    }
    hasher.finish()
}

// =============================================================================
// Test Setup Helpers
// =============================================================================

async fn setup_test_db() -> DatabaseConnection {
    let db = db::init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn create_test_user(
    db: &DatabaseConnection,
    enabled: bool,
    notify_on_change: bool,
    webhook_url: Option<String>,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        enabled: Set(enabled),
        notify_on_change_only: Set(notify_on_change),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(webhook_url),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(db).await.unwrap();
    user_id
}

async fn create_test_route_with_details(
    db: &DatabaseConnection,
    user_id: Uuid,
    area_id: i32,
    route_id: &str,
    departure_station: &str,
    arrival_station: &str,
    departure_time_min: Option<String>,
    departure_time_max: Option<String>,
    adult_men: i16,
) -> Uuid {
    let route_uuid = Uuid::new_v4();
    let route = user_routes::ActiveModel {
        id: Set(route_uuid),
        user_id: Set(user_id),
        area_id: Set(area_id),
        route_id: Set(route_id.to_string()),
        departure_station: Set(departure_station.to_string()),
        arrival_station: Set(arrival_station.to_string()),
        date_start: Set("2025-01-15".to_string()),
        date_end: Set("2025-01-20".to_string()),
        departure_time_min: Set(departure_time_min),
        departure_time_max: Set(departure_time_max),
        created_at: Set(chrono::Utc::now()),
    };
    route.insert(db).await.unwrap();

    let passengers = user_passengers::ActiveModel {
        user_route_id: Set(route_uuid),
        adult_men: Set(adult_men),
        adult_women: Set(0),
        child_men: Set(0),
        child_women: Set(0),
        handicap_adult_men: Set(0),
        handicap_adult_women: Set(0),
        handicap_child_men: Set(0),
        handicap_child_women: Set(0),
    };
    passengers.insert(db).await.unwrap();

    route_uuid
}

fn build_user_route_details(
    user_route_id: Uuid,
    email: &str,
    notify_on_change_only: bool,
    scrape_interval_secs: i64,
    discord_webhook_url: Option<String>,
    area_id: i32,
    route_id: &str,
    departure_station: &str,
    arrival_station: &str,
    date_start: &str,
    date_end: &str,
    departure_time_min: Option<String>,
    departure_time_max: Option<String>,
    passengers: PassengerDetails,
) -> UserRouteWithDetails {
    UserRouteWithDetails {
        user_route_id,
        email: email.to_string(),
        notify_on_change_only,
        scrape_interval_secs,
        discord_webhook_url,
        area_id,
        route_id: route_id.to_string(),
        departure_station: departure_station.to_string(),
        arrival_station: arrival_station.to_string(),
        date_start: date_start.to_string(),
        date_end: date_end.to_string(),
        departure_time_min,
        departure_time_max,
        passengers,
    }
}

// =============================================================================
// populate_station_cache Tests
// =============================================================================

#[tokio::test]
async fn test_populate_station_cache_success() {
    let mock_server = MockServer::start().await;

    let stations_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<stations>
    <id>001</id>
    <name>Tokyo Station</name>
    <id>064</id>
    <name>Osaka Station</name>
    <id>498</id>
    <name>Kyoto Station</name>
</stations>"#;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=station_geton"))
        .and(body_string_contains("id=155"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stations_xml))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

    // Populate cache
    let result = scraper.fetch_departure_stations("155").await.unwrap();
    let mut cache_lock = cache.write().await;
    for station in result {
        cache_lock.insert(station.id, station.name);
    }
    drop(cache_lock);

    // Verify cache contents
    let cache_lock = cache.read().await;
    assert_eq!(cache_lock.len(), 3);
    assert_eq!(cache_lock.get("001").unwrap(), "Tokyo Station");
    assert_eq!(cache_lock.get("064").unwrap(), "Osaka Station");
    assert_eq!(cache_lock.get("498").unwrap(), "Kyoto Station");
}

#[tokio::test]
async fn test_populate_station_cache_empty_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=station_geton"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<?xml version="1.0" encoding="UTF-8"?><stations></stations>"#),
        )
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let result = scraper.fetch_departure_stations("999").await.unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_populate_station_cache_with_duplicate_ids() {
    let mock_server = MockServer::start().await;

    // API returns duplicate station IDs (last one wins)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<stations>
    <id>001</id>
    <name>Tokyo Station</name>
    <id>001</id>
    <name>Tokyo Station Updated</name>
</stations>"#;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=station_geton"))
        .respond_with(ResponseTemplate::new(200).set_body_string(xml))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

    let result = scraper.fetch_departure_stations("155").await.unwrap();
    let mut cache_lock = cache.write().await;
    for station in result {
        cache_lock.insert(station.id, station.name);
    }
    drop(cache_lock);

    let cache_lock = cache.read().await;
    // Only one entry due to duplicate ID
    assert_eq!(cache_lock.len(), 1);
    assert_eq!(cache_lock.get("001").unwrap(), "Tokyo Station Updated");
}

// =============================================================================
// build_scrape_request Tests
// =============================================================================

#[tokio::test]
async fn test_build_scrape_request_basic() {
    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        None,
        None,
        PassengerDetails {
            adult_men: 2,
            adult_women: 1,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    // Simulate build_scrape_request logic
    let request = ScrapeRequest {
        area_id: user_route.area_id as u32,
        route_id: user_route.route_id.parse().unwrap_or(0),
        departure_station: user_route.departure_station.clone(),
        arrival_station: user_route.arrival_station.clone(),
        date_range: DateRange {
            start: user_route.date_start.clone(),
            end: user_route.date_end.clone(),
        },
        passengers: PassengerCount {
            adult_men: user_route.passengers.adult_men as u8,
            adult_women: user_route.passengers.adult_women as u8,
            child_men: user_route.passengers.child_men as u8,
            child_women: user_route.passengers.child_women as u8,
            handicap_adult_men: user_route.passengers.handicap_adult_men as u8,
            handicap_adult_women: user_route.passengers.handicap_adult_women as u8,
            handicap_child_men: user_route.passengers.handicap_child_men as u8,
            handicap_child_women: user_route.passengers.handicap_child_women as u8,
        },
        time_filter: None,
    };

    assert_eq!(request.area_id, 1);
    assert_eq!(request.route_id, 155);
    assert_eq!(request.departure_station, "001");
    assert_eq!(request.arrival_station, "498");
    assert_eq!(request.date_range.start, "2025-01-15");
    assert_eq!(request.date_range.end, "2025-01-20");
    assert_eq!(request.passengers.adult_men, 2);
    assert_eq!(request.passengers.adult_women, 1);
    assert_eq!(request.passengers.total(), 3);
    assert!(request.time_filter.is_none());
}

#[tokio::test]
async fn test_build_scrape_request_with_time_filter() {
    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        Some("08:00".to_string()),
        Some("18:00".to_string()),
        PassengerDetails {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    let request = ScrapeRequest {
        area_id: user_route.area_id as u32,
        route_id: user_route.route_id.parse().unwrap_or(0),
        departure_station: user_route.departure_station.clone(),
        arrival_station: user_route.arrival_station.clone(),
        date_range: DateRange {
            start: user_route.date_start.clone(),
            end: user_route.date_end.clone(),
        },
        passengers: PassengerCount {
            adult_men: user_route.passengers.adult_men as u8,
            adult_women: user_route.passengers.adult_women as u8,
            child_men: user_route.passengers.child_men as u8,
            child_women: user_route.passengers.child_women as u8,
            handicap_adult_men: user_route.passengers.handicap_adult_men as u8,
            handicap_adult_women: user_route.passengers.handicap_adult_women as u8,
            handicap_child_men: user_route.passengers.handicap_child_men as u8,
            handicap_child_women: user_route.passengers.handicap_child_women as u8,
        },
        time_filter: match (
            &user_route.departure_time_min,
            &user_route.departure_time_max,
        ) {
            (None, None) => None,
            (min, max) => Some(TimeFilter {
                departure_min: min.clone(),
                departure_max: max.clone(),
            }),
        },
    };

    assert!(request.time_filter.is_some());
    let time_filter = request.time_filter.unwrap();
    assert_eq!(time_filter.departure_min, Some("08:00".to_string()));
    assert_eq!(time_filter.departure_max, Some("18:00".to_string()));
}

#[tokio::test]
async fn test_build_scrape_request_with_partial_time_filter() {
    // Only min time
    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        Some("08:00".to_string()),
        None,
        PassengerDetails {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    let request = ScrapeRequest {
        area_id: user_route.area_id as u32,
        route_id: user_route.route_id.parse().unwrap_or(0),
        departure_station: user_route.departure_station.clone(),
        arrival_station: user_route.arrival_station.clone(),
        date_range: DateRange {
            start: user_route.date_start.clone(),
            end: user_route.date_end.clone(),
        },
        passengers: PassengerCount {
            adult_men: user_route.passengers.adult_men as u8,
            adult_women: user_route.passengers.adult_women as u8,
            child_men: user_route.passengers.child_men as u8,
            child_women: user_route.passengers.child_women as u8,
            handicap_adult_men: user_route.passengers.handicap_adult_men as u8,
            handicap_adult_women: user_route.passengers.handicap_adult_women as u8,
            handicap_child_men: user_route.passengers.handicap_child_men as u8,
            handicap_child_women: user_route.passengers.handicap_child_women as u8,
        },
        time_filter: match (
            &user_route.departure_time_min,
            &user_route.departure_time_max,
        ) {
            (None, None) => None,
            (min, max) => Some(TimeFilter {
                departure_min: min.clone(),
                departure_max: max.clone(),
            }),
        },
    };

    assert!(request.time_filter.is_some());
    let time_filter = request.time_filter.unwrap();
    assert_eq!(time_filter.departure_min, Some("08:00".to_string()));
    assert_eq!(time_filter.departure_max, None);
}

#[tokio::test]
async fn test_build_scrape_request_with_all_passenger_types() {
    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        None,
        None,
        PassengerDetails {
            adult_men: 2,
            adult_women: 1,
            child_men: 1,
            child_women: 1,
            handicap_adult_men: 1,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 1,
        },
    );

    let request = ScrapeRequest {
        area_id: user_route.area_id as u32,
        route_id: user_route.route_id.parse().unwrap_or(0),
        departure_station: user_route.departure_station.clone(),
        arrival_station: user_route.arrival_station.clone(),
        date_range: DateRange {
            start: user_route.date_start.clone(),
            end: user_route.date_end.clone(),
        },
        passengers: PassengerCount {
            adult_men: user_route.passengers.adult_men as u8,
            adult_women: user_route.passengers.adult_women as u8,
            child_men: user_route.passengers.child_men as u8,
            child_women: user_route.passengers.child_women as u8,
            handicap_adult_men: user_route.passengers.handicap_adult_men as u8,
            handicap_adult_women: user_route.passengers.handicap_adult_women as u8,
            handicap_child_men: user_route.passengers.handicap_child_men as u8,
            handicap_child_women: user_route.passengers.handicap_child_women as u8,
        },
        time_filter: None,
    };

    assert_eq!(request.passengers.adult_men, 2);
    assert_eq!(request.passengers.adult_women, 1);
    assert_eq!(request.passengers.child_men, 1);
    assert_eq!(request.passengers.child_women, 1);
    assert_eq!(request.passengers.handicap_adult_men, 1);
    assert_eq!(request.passengers.handicap_adult_women, 0);
    assert_eq!(request.passengers.handicap_child_men, 0);
    assert_eq!(request.passengers.handicap_child_women, 1);
    assert_eq!(request.passengers.total(), 7);
}

#[tokio::test]
async fn test_build_scrape_request_invalid_route_id() {
    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "invalid",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        None,
        None,
        PassengerDetails {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    // Should fallback to 0 when route_id cannot be parsed
    let request = ScrapeRequest {
        area_id: user_route.area_id as u32,
        route_id: user_route.route_id.parse().unwrap_or(0),
        departure_station: user_route.departure_station.clone(),
        arrival_station: user_route.arrival_station.clone(),
        date_range: DateRange {
            start: user_route.date_start.clone(),
            end: user_route.date_end.clone(),
        },
        passengers: PassengerCount {
            adult_men: user_route.passengers.adult_men as u8,
            adult_women: user_route.passengers.adult_women as u8,
            child_men: user_route.passengers.child_men as u8,
            child_women: user_route.passengers.child_women as u8,
            handicap_adult_men: user_route.passengers.handicap_adult_men as u8,
            handicap_adult_women: user_route.passengers.handicap_adult_women as u8,
            handicap_child_men: user_route.passengers.handicap_child_men as u8,
            handicap_child_women: user_route.passengers.handicap_child_women as u8,
        },
        time_filter: None,
    };

    assert_eq!(request.route_id, 0);
}

// =============================================================================
// build_notification_context Tests
// =============================================================================

#[tokio::test]
async fn test_build_notification_context_with_cached_stations() {
    let cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    {
        let mut cache_lock = cache.write().await;
        cache_lock.insert("001".to_string(), "Tokyo Station".to_string());
        cache_lock.insert("498".to_string(), "Kyoto Station".to_string());
    }

    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        Some("08:00".to_string()),
        Some("18:00".to_string()),
        PassengerDetails {
            adult_men: 2,
            adult_women: 1,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    // Simulate build_notification_context
    let cache_lock = cache.read().await;
    let departure_name = cache_lock
        .get(&user_route.departure_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.departure_station));
    let arrival_name = cache_lock
        .get(&user_route.arrival_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.arrival_station));

    let context = NotificationContext {
        departure_station_name: departure_name,
        arrival_station_name: arrival_name,
        date_range: (user_route.date_start.clone(), user_route.date_end.clone()),
        passenger_count: user_route.passengers.total() as u8,
        time_filter: match (
            &user_route.departure_time_min,
            &user_route.departure_time_max,
        ) {
            (Some(min), Some(max)) => Some((min.clone(), max.clone())),
            _ => None,
        },
    };

    assert_eq!(context.departure_station_name, "Tokyo Station");
    assert_eq!(context.arrival_station_name, "Kyoto Station");
    assert_eq!(
        context.date_range,
        ("2025-01-15".to_string(), "2025-01-20".to_string())
    );
    assert_eq!(context.passenger_count, 3);
    assert_eq!(
        context.time_filter,
        Some(("08:00".to_string(), "18:00".to_string()))
    );
}

#[tokio::test]
async fn test_build_notification_context_with_missing_stations() {
    let cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    // Cache is empty - no stations

    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        None,
        None,
        PassengerDetails {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    let cache_lock = cache.read().await;
    let departure_name = cache_lock
        .get(&user_route.departure_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.departure_station));
    let arrival_name = cache_lock
        .get(&user_route.arrival_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.arrival_station));

    let context = NotificationContext {
        departure_station_name: departure_name,
        arrival_station_name: arrival_name,
        date_range: (user_route.date_start.clone(), user_route.date_end.clone()),
        passenger_count: user_route.passengers.total() as u8,
        time_filter: None,
    };

    // Should fall back to generic names
    assert_eq!(context.departure_station_name, "Station 001");
    assert_eq!(context.arrival_station_name, "Station 498");
}

#[tokio::test]
async fn test_build_notification_context_no_time_filter() {
    let cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    {
        let mut cache_lock = cache.write().await;
        cache_lock.insert("001".to_string(), "Tokyo Station".to_string());
        cache_lock.insert("498".to_string(), "Kyoto Station".to_string());
    }

    let user_route = build_user_route_details(
        Uuid::new_v4(),
        "test@example.com",
        true,
        300,
        None,
        1,
        "155",
        "001",
        "498",
        "2025-01-15",
        "2025-01-20",
        None,
        None,
        PassengerDetails {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        },
    );

    let cache_lock = cache.read().await;
    let departure_name = cache_lock
        .get(&user_route.departure_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.departure_station));
    let arrival_name = cache_lock
        .get(&user_route.arrival_station)
        .cloned()
        .unwrap_or_else(|| format!("Station {}", user_route.arrival_station));

    let context = NotificationContext {
        departure_station_name: departure_name,
        arrival_station_name: arrival_name,
        date_range: (user_route.date_start.clone(), user_route.date_end.clone()),
        passenger_count: user_route.passengers.total() as u8,
        time_filter: match (
            &user_route.departure_time_min,
            &user_route.departure_time_max,
        ) {
            (Some(min), Some(max)) => Some((min.clone(), max.clone())),
            _ => None,
        },
    };

    assert!(context.time_filter.is_none());
}

// =============================================================================
// run_tracker Tests
// =============================================================================

#[tokio::test]
async fn test_run_tracker_with_no_routes() {
    let db = setup_test_db().await;

    // No users or routes created
    let routes = get_all_active_user_routes(&db).await.unwrap();

    assert_eq!(routes.len(), 0);
    // run_tracker should return early with warning
}

#[tokio::test]
async fn test_run_tracker_with_disabled_user() {
    let db = setup_test_db().await;

    // Create disabled user with route
    let user_id = create_test_user(
        &db,
        false,
        true,
        Some("https://discord.com/webhook".to_string()),
    )
    .await;
    create_test_route_with_details(&db, user_id, 1, "155", "001", "498", None, None, 1).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    // Should not return routes for disabled users
    assert_eq!(routes.len(), 0);
}

#[tokio::test]
async fn test_run_tracker_with_multiple_enabled_users() {
    let db = setup_test_db().await;

    // Create 3 enabled users with routes
    for _ in 0..3 {
        let user_id = create_test_user(
            &db,
            true,
            true,
            Some("https://discord.com/webhook".to_string()),
        )
        .await;
        create_test_route_with_details(&db, user_id, 1, "155", "001", "498", None, None, 1).await;
    }

    let routes = get_all_active_user_routes(&db).await.unwrap();

    assert_eq!(routes.len(), 3);
}

#[tokio::test]
async fn test_run_tracker_with_mixed_enabled_disabled() {
    let db = setup_test_db().await;

    // 2 enabled users
    for _ in 0..2 {
        let user_id = create_test_user(
            &db,
            true,
            true,
            Some("https://discord.com/webhook".to_string()),
        )
        .await;
        create_test_route_with_details(&db, user_id, 1, "155", "001", "498", None, None, 1).await;
    }

    // 1 disabled user
    let disabled_user = create_test_user(
        &db,
        false,
        true,
        Some("https://discord.com/webhook".to_string()),
    )
    .await;
    create_test_route_with_details(&db, disabled_user, 1, "155", "001", "498", None, None, 1).await;

    let routes = get_all_active_user_routes(&db).await.unwrap();

    // Should only return enabled users
    assert_eq!(routes.len(), 2);
}

// =============================================================================
// calculate_state_hash Edge Cases
// =============================================================================

#[tokio::test]
async fn test_calculate_state_hash_with_multiple_plans_per_schedule() {
    use app::types::{BusSchedule, PricingPlan, SeatAvailability};

    let schedule = BusSchedule {
        bus_number: "Bus_1".to_string(),
        route_name: "Test Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250115".to_string(),
        departure_time: "08:30".to_string(),
        arrival_station: "498".to_string(),
        arrival_date: "20250115".to_string(),
        arrival_time: "10:00".to_string(),
        way_no: 1,
        available_plans: vec![
            PricingPlan {
                plan_id: 12345,
                plan_index: 0,
                plan_name: "Standard".to_string(),
                price: 2100,
                display_price: "2100円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(5),
                },
            },
            PricingPlan {
                plan_id: 12346,
                plan_index: 1,
                plan_name: "Premium".to_string(),
                price: 3200,
                display_price: "3200円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(3),
                },
            },
        ],
    };

    let schedules = vec![schedule];
    let hash1 = calculate_state_hash(&schedules);

    // Change second plan's seats
    let mut schedule2 = schedules[0].clone();
    schedule2.available_plans[1].availability = SeatAvailability::Available {
        remaining_seats: Some(2),
    };
    let schedules2 = vec![schedule2];
    let hash2 = calculate_state_hash(&schedules2);

    assert_ne!(hash1, hash2);
}

#[tokio::test]
async fn test_calculate_state_hash_with_zero_remaining_seats() {
    use app::types::{BusSchedule, PricingPlan, SeatAvailability};

    let schedule = BusSchedule {
        bus_number: "Bus_1".to_string(),
        route_name: "Test Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250115".to_string(),
        departure_time: "08:30".to_string(),
        arrival_station: "498".to_string(),
        arrival_date: "20250115".to_string(),
        arrival_time: "10:00".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 12345,
            plan_index: 0,
            plan_name: "Standard".to_string(),
            price: 2100,
            display_price: "2100円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(0),
            },
        }],
    };

    let hash1 = calculate_state_hash(&vec![schedule.clone()]);

    // Change to Some(1)
    let mut schedule2 = schedule;
    schedule2.available_plans[0].availability = SeatAvailability::Available {
        remaining_seats: Some(1),
    };
    let hash2 = calculate_state_hash(&vec![schedule2]);

    assert_ne!(hash1, hash2);
}

#[tokio::test]
async fn test_calculate_state_hash_large_schedule_list() {
    use app::types::{BusSchedule, PricingPlan, SeatAvailability};

    let mut schedules = Vec::new();
    for i in 0..100 {
        schedules.push(BusSchedule {
            bus_number: format!("Bus_{}", i),
            route_name: "Test Route".to_string(),
            departure_station: "001".to_string(),
            departure_date: "20250115".to_string(),
            departure_time: format!("{:02}:30", 8 + (i % 12)),
            arrival_station: "498".to_string(),
            arrival_date: "20250115".to_string(),
            arrival_time: "10:00".to_string(),
            way_no: 1,
            available_plans: vec![PricingPlan {
                plan_id: 12345 + i,
                plan_index: 0,
                plan_name: "Standard".to_string(),
                price: 2100,
                display_price: "2100円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(5),
                },
            }],
        });
    }

    let hash1 = calculate_state_hash(&schedules);
    let hash2 = calculate_state_hash(&schedules);

    // Same data produces same hash
    assert_eq!(hash1, hash2);

    // Modify one schedule
    schedules[50].available_plans[0].availability = SeatAvailability::Available {
        remaining_seats: Some(4),
    };
    let hash3 = calculate_state_hash(&schedules);

    // Hash should change
    assert_ne!(hash1, hash3);
}
