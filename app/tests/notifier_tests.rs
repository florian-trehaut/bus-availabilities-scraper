//! Additional notifier tests targeting uncovered edge cases
//!
//! Covers:
//! - Network errors (lines 62-64, 97-99)
//! - HTTP error responses (lines 56-57)
//! - Empty available_plans (line 114)
//! - SeatAvailability::Available with None seats (line 125)

use app::notifier::{DiscordNotifier, NotificationContext};
use app::types::{BusSchedule, PricingPlan, SeatAvailability};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_context() -> NotificationContext {
    NotificationContext {
        departure_station_name: "Tokyo".to_string(),
        arrival_station_name: "Osaka".to_string(),
        date_range: ("20250201".to_string(), "20250210".to_string()),
        passenger_count: 3,
        time_filter: Some(("09:00".to_string(), "18:00".to_string())),
    }
}

#[tokio::test]
async fn test_send_startup_notification_http_error_response() {
    let mock_server = MockServer::start().await;

    // Return 400 Bad Request to trigger lines 56-57
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Should handle error gracefully and still return Ok
    let result = notifier.send_startup_notification(&webhook_url, 2, 5).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_startup_notification_network_error() {
    let notifier = DiscordNotifier::new();
    // Use invalid URL to trigger network error (lines 62-64)
    let invalid_url = "http://invalid-host-that-does-not-exist:9999/webhook";

    // Should handle network error gracefully and still return Ok
    let result = notifier.send_startup_notification(invalid_url, 1, 3).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_http_error_response() {
    let mock_server = MockServer::start().await;

    // Return 503 Service Unavailable
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    let schedules = vec![BusSchedule {
        bus_number: "Bus_1".to_string(),
        route_name: "Test Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250201".to_string(),
        departure_time: "10:30".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250201".to_string(),
        arrival_time: "14:00".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 111,
            plan_index: 0,
            plan_name: "Economy".to_string(),
            price: 3000,
            display_price: "3,000円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(10),
            },
        }],
    }];

    let context = test_context();

    // Should handle error gracefully and still return Ok
    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_network_error() {
    let notifier = DiscordNotifier::new();
    // Use invalid URL to trigger network error (lines 97-99)
    let invalid_url = "http://non-existent-webhook-server.invalid:8888/webhook";

    let schedules = vec![BusSchedule {
        bus_number: "Bus_2".to_string(),
        route_name: "Network Test".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250202".to_string(),
        departure_time: "11:00".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250202".to_string(),
        arrival_time: "15:30".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 222,
            plan_index: 0,
            plan_name: "Standard".to_string(),
            price: 4000,
            display_price: "4,000円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(5),
            },
        }],
    }];

    let context = test_context();

    // Should handle network error gracefully and still return Ok
    let result = notifier
        .send_availability_alert(invalid_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_with_empty_available_plans() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Mix of schedules: one with plans, one without (tests line 114 continue)
    let schedules = vec![
        BusSchedule {
            bus_number: "Bus_Empty".to_string(),
            route_name: "No Plans".to_string(),
            departure_station: "001".to_string(),
            departure_date: "20250203".to_string(),
            departure_time: "08:00".to_string(),
            arrival_station: "064".to_string(),
            arrival_date: "20250203".to_string(),
            arrival_time: "12:00".to_string(),
            way_no: 1,
            available_plans: vec![], // Empty - should be skipped in loop
        },
        BusSchedule {
            bus_number: "Bus_WithPlan".to_string(),
            route_name: "Has Plans".to_string(),
            departure_station: "001".to_string(),
            departure_date: "20250203".to_string(),
            departure_time: "09:00".to_string(),
            arrival_station: "064".to_string(),
            arrival_date: "20250203".to_string(),
            arrival_time: "13:00".to_string(),
            way_no: 1,
            available_plans: vec![PricingPlan {
                plan_id: 333,
                plan_index: 0,
                plan_name: "Premium".to_string(),
                price: 5000,
                display_price: "5,000円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(2),
                },
            }],
        },
    ];

    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_with_none_remaining_seats() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Test line 125: remaining_seats: None case
    let schedules = vec![BusSchedule {
        bus_number: "Bus_UnknownSeats".to_string(),
        route_name: "No Seat Count".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250204".to_string(),
        departure_time: "07:30".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250204".to_string(),
        arrival_time: "11:45".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 444,
            plan_index: 0,
            plan_name: "Basic".to_string(),
            price: 2500,
            display_price: "2,500円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: None, // Test None case (line 125)
            },
        }],
    }];

    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notification_context_building_variations() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .expect(2)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    let schedule = BusSchedule {
        bus_number: "Bus_Test".to_string(),
        route_name: "Context Test".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250205".to_string(),
        departure_time: "12:00".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250205".to_string(),
        arrival_time: "16:00".to_string(),
        way_no: 1,
        available_plans: vec![PricingPlan {
            plan_id: 555,
            plan_index: 0,
            plan_name: "Test Plan".to_string(),
            price: 3500,
            display_price: "3,500円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(8),
            },
        }],
    };

    // Test with time_filter
    let context_with_filter = NotificationContext {
        departure_station_name: "Kyoto".to_string(),
        arrival_station_name: "Nagoya".to_string(),
        date_range: ("20250205".to_string(), "20250212".to_string()),
        passenger_count: 1,
        time_filter: Some(("06:00".to_string(), "22:00".to_string())),
    };

    let result1 = notifier
        .send_availability_alert(&webhook_url, &[schedule.clone()], &context_with_filter)
        .await;
    assert!(result1.is_ok());

    // Test without time_filter
    let context_no_filter = NotificationContext {
        departure_station_name: "Fukuoka".to_string(),
        arrival_station_name: "Hiroshima".to_string(),
        date_range: ("20250205".to_string(), "20250212".to_string()),
        passenger_count: 4,
        time_filter: None,
    };

    let result2 = notifier
        .send_availability_alert(&webhook_url, &[schedule], &context_no_filter)
        .await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_send_availability_alert_multiple_plans_per_schedule() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notifier = DiscordNotifier::new();
    let webhook_url = format!("{}/webhook", mock_server.uri());

    // Schedule with multiple pricing plans
    let schedules = vec![BusSchedule {
        bus_number: "Bus_MultiPlan".to_string(),
        route_name: "Multi Plan Route".to_string(),
        departure_station: "001".to_string(),
        departure_date: "20250206".to_string(),
        departure_time: "14:00".to_string(),
        arrival_station: "064".to_string(),
        arrival_date: "20250206".to_string(),
        arrival_time: "18:00".to_string(),
        way_no: 1,
        available_plans: vec![
            PricingPlan {
                plan_id: 601,
                plan_index: 0,
                plan_name: "Economy".to_string(),
                price: 2000,
                display_price: "2,000円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(15),
                },
            },
            PricingPlan {
                plan_id: 602,
                plan_index: 1,
                plan_name: "Business".to_string(),
                price: 5000,
                display_price: "5,000円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: None,
                },
            },
            PricingPlan {
                plan_id: 603,
                plan_index: 2,
                plan_name: "First Class".to_string(),
                price: 8000,
                display_price: "8,000円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(3),
                },
            },
        ],
    }];

    let context = test_context();

    let result = notifier
        .send_availability_alert(&webhook_url, &schedules, &context)
        .await;

    assert!(result.is_ok());
}
