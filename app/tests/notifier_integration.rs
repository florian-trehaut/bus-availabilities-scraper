//! Integration tests for notifier.rs using wiremock
//!
//! Tests Discord webhook notifications with mocked HTTP server
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::doc_markdown,
    clippy::uninlined_format_args
)]

use app::notifier::{DiscordNotifier, NotificationContext};
use app::types::{BusSchedule, PricingPlan, SeatAvailability};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_bus_schedule() -> BusSchedule {
    BusSchedule {
        bus_number: "Bus_1".to_string(),
        route_name: "Test Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250115".to_string(),
        departure_time: "08:30".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250115".to_string(),
        arrival_time: "10:45".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 12345,
            plan_index: 0,
            plan_name: "Standard".to_string(),
            price: 2100,
            display_price: "2,100円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(5),
            },
        }],
    }
}

fn test_context() -> NotificationContext {
    NotificationContext {
        departure_station_name: "Shinjuku".to_string(),
        arrival_station_name: "Kawaguchiko".to_string(),
        date_range: ("20250115".to_string(), "20250120".to_string()),
        passenger_count: 2,
        time_filter: None,
    }
}

#[tokio::test]
async fn test_send_startup_notification_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    let result = notifier
        .send_startup_notification(&webhook_url, 5, 10)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_startup_notification_failure_handled() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Should not return error even on 500 status
    let result = notifier
        .send_startup_notification(&webhook_url, 5, 10)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let schedules = vec![test_bus_schedule()];
    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_empty_schedules() {
    let mock_server = MockServer::start().await;

    // Should NOT call webhook when schedules are empty
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0) // Expect NO calls
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let schedules: Vec<BusSchedule> = vec![];
    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_with_time_filter() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let schedules = vec![test_bus_schedule()];
    let mut context = test_context();
    context.time_filter = Some(("08:00".to_string(), "12:00".to_string()));

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_failure_handled() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(429)) // Rate limited
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());
    let schedules = vec![test_bus_schedule()];
    let context = test_context();

    // Should not return error even on rate limit
    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_multiple_schedules() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    let mut schedule2 = test_bus_schedule();
    schedule2.bus_number = "Bus_2".to_string();
    schedule2.departure_time = "10:00".to_string();
    schedule2.arrival_time = "12:15".to_string();

    let schedules = vec![test_bus_schedule(), schedule2];
    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notifier_default_trait() {
    let notifier = DiscordNotifier::default();
    // Just verify it can be created via Default trait
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let webhook_url = format!("{}/webhook", mock_server.uri());
    let result = notifier.send_startup_notification(&webhook_url, 1, 1).await;

    assert!(result.is_ok());
}
