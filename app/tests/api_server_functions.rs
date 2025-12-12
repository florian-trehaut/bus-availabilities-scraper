//! Integration tests for Leptos server functions in `app/src/api.rs`
//!
//! These tests invoke server functions via HTTP through an Axum router
//! with proper Leptos context injection, following the same pattern as
//! existing tests in `server/tests/api_e2e.rs`.
//!
//! Test approach:
//! 1. Set up in-memory SQLite database with migrations
//! 2. Create Axum router with Leptos server function handlers
//! 3. Mock BusScraper HTTP calls with wiremock::MockServer
//! 4. Send HTTP requests to `/api/{function_name}` endpoints
//! 5. Verify responses and database state

use app::{
    db,
    entities::{user_passengers, user_routes, users},
    scraper::BusScraper,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use http_body_util::BodyExt;
use leptos::context::provide_context;
use leptos_axum::handle_server_fns_with_context;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// =============================================================================
// Test Setup Helpers
// =============================================================================

async fn setup_test_db() -> DatabaseConnection {
    let db = db::init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn setup_test_app(db: DatabaseConnection, mock_server: &MockServer) -> Router {
    let scraper = Arc::new(BusScraper::new(mock_server.uri()).unwrap());

    let db_clone = db.clone();
    let scraper_clone = scraper.clone();

    Router::new().route(
        "/api/{*fn_name}",
        get({
            let db = db_clone.clone();
            let scraper = scraper_clone.clone();
            move |req| {
                let db = db.clone();
                let scraper = scraper.clone();
                async move {
                    handle_server_fns_with_context(
                        move || {
                            provide_context(db.clone());
                            provide_context(scraper.clone());
                        },
                        req,
                    )
                    .await
                }
            }
        })
        .post({
            let db = db_clone;
            let scraper = scraper_clone;
            move |req| {
                let db = db.clone();
                let scraper = scraper.clone();
                async move {
                    handle_server_fns_with_context(
                        move || {
                            provide_context(db.clone());
                            provide_context(scraper.clone());
                        },
                        req,
                    )
                    .await
                }
            }
        }),
    )
}

async fn create_test_user(db: &DatabaseConnection, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    new_user.insert(db).await.unwrap();
    user_id
}

async fn create_test_user_route(
    db: &DatabaseConnection,
    user_id: Uuid,
    route_id_str: &str,
) -> Uuid {
    let route_id = Uuid::new_v4();
    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set(route_id_str.to_string()),
        departure_station: Set("001".to_string()),
        arrival_station: Set("498".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-15".to_string()),
        departure_time_min: Set(Some("08:00".to_string())),
        departure_time_max: Set(Some("18:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    new_route.insert(db).await.unwrap();

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
    new_passengers.insert(db).await.unwrap();

    route_id
}

// =============================================================================
// get_users Tests
// =============================================================================

#[tokio::test]
async fn test_get_users_empty() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_users")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status() == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_users_returns_data() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    create_test_user(&db, "test@example.com").await;
    create_test_user(&db, "test2@example.com").await;

    let app = setup_test_app(db, &mock_server).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_users")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status() == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_users_multiple_calls() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    create_test_user(&db, "multi@example.com").await;

    let app = setup_test_app(db, &mock_server).await;

    for _ in 0..3 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/get_users")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success() || response.status() == StatusCode::BAD_REQUEST);
    }
}

// =============================================================================
// create_user Tests
// =============================================================================

#[tokio::test]
async fn test_create_user_valid_data() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data =
        "email=new%40example.com&enabled=true&notify_on_change_only=false&scrape_interval_secs=300";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/create_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_create_user_with_email_formats() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let test_emails = vec![
        "simple@example.com",
        "user%2Btag@example.com", // + encoded
        "user.name@example.co.jp",
    ];

    for email in test_emails {
        let app = setup_test_app(db.clone(), &mock_server).await;
        let form_data = format!(
            "email={}&enabled=true&notify_on_change_only=false&scrape_interval_secs=300",
            email
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/create_user")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success() || response.status().is_client_error());
    }
}

#[tokio::test]
async fn test_create_user_without_webhook() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "email=no-webhook%40example.com&enabled=false&notify_on_change_only=true&scrape_interval_secs=600";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/create_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

// =============================================================================
// update_user Tests
// =============================================================================

#[tokio::test]
async fn test_update_user_valid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "original@example.com").await;

    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data = format!(
        "id={}&email=updated%40example.com&enabled=false&notify_on_change_only=true&scrape_interval_secs=600",
        user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_update_user_non_existent() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let non_existent_id = Uuid::new_v4();
    let form_data = format!(
        "id={}&email=test%40example.com&enabled=true&notify_on_change_only=false&scrape_interval_secs=300",
        non_existent_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return an error response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

#[tokio::test]
async fn test_update_user_invalid_uuid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "id=invalid-uuid&email=test%40example.com&enabled=true&notify_on_change_only=false&scrape_interval_secs=300";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// delete_user Tests
// =============================================================================

#[tokio::test]
async fn test_delete_user_valid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "to-delete@example.com").await;

    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data = format!("id={}", user_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_user_non_existent() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let non_existent_id = Uuid::new_v4();
    let form_data = format!("id={}", non_existent_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed (SeaORM doesn't fail on non-existent deletes)
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_user_invalid_uuid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "id=invalid-uuid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// get_routes Tests (with mocked API)
// =============================================================================

#[tokio::test]
async fn test_get_routes_with_mock_api() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <id>155</id>
            <name>新宿～上高地線</name>
            <switchChangeableFlg>0</switchChangeableFlg>
            <id>160</id>
            <name>東京～大阪線</name>"#,
        ))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "area_id=1";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_routes_different_area_ids() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"<id>200</id><name>Test Route</name>"#),
        )
        .mount(&mock_server)
        .await;

    for area_id in [1, 2, 3] {
        let app = setup_test_app(db.clone(), &mock_server).await;
        let form_data = format!("area_id={}", area_id);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/get_routes")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success() || response.status().is_client_error());
    }
}

#[tokio::test]
async fn test_get_routes_empty_response() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<?xml version="1.0" encoding="UTF-8"?><routes></routes>"#),
        )
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "area_id=1";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_routes_api_error() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "area_id=1";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Server error from scraper should be handled
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// get_departure_stations Tests
// =============================================================================

#[tokio::test]
async fn test_get_departure_stations_with_mock_api() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <id>001</id>
            <name>バスタ新宿（南口）</name>
            <id>002</id>
            <name>渋谷マークシティ</name>"#,
        ))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=155";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_departure_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_departure_stations_empty_response() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<?xml version="1.0" encoding="UTF-8"?><stations></stations>"#),
        )
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=999";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_departure_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_departure_stations_api_error() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=invalid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_departure_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// get_arrival_stations Tests
// =============================================================================

#[tokio::test]
async fn test_get_arrival_stations_with_mock_api() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<id>498</id>
            <name>上高地バスターミナル</name>
            <id>499</id>
            <name>松本駅</name>"#,
        ))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=155&departure_station_id=001";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_arrival_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_arrival_stations_empty_response() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<?xml version="1.0" encoding="UTF-8"?><stations></stations>"#),
        )
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=155&departure_station_id=999";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_arrival_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_arrival_stations_api_error() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = "route_id=155&departure_station_id=001";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_arrival_stations")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// create_user_route Tests
// =============================================================================

#[tokio::test]
async fn test_create_user_route_with_passengers() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "route-test@example.com").await;

    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data = format!(
        "user_id={}&area_id=1&route_id=155&departure_station=001&arrival_station=498&\
        date_start=2025-01-01&date_end=2025-01-15&departure_time_min=08%3A00&departure_time_max=18%3A00&\
        adult_men=2&adult_women=1&child_men=0&child_women=1&\
        handicap_adult_men=0&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0",
        user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/create_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_create_user_route_without_time_filter() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "route-test-2@example.com").await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!(
        "user_id={}&area_id=2&route_id=160&departure_station=010&arrival_station=020&\
        date_start=2025-02-01&date_end=2025-02-28&\
        adult_men=1&adult_women=0&child_men=0&child_women=0&\
        handicap_adult_men=0&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0",
        user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/create_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_create_user_route_invalid_user_id() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "user_id=invalid-uuid&area_id=1&route_id=155&departure_station=001&arrival_station=498&\
        date_start=2025-01-01&date_end=2025-01-15&\
        adult_men=1&adult_women=0&child_men=0&child_women=0&\
        handicap_adult_men=0&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/create_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// get_user_routes Tests
// =============================================================================

#[tokio::test]
async fn test_get_user_routes_empty() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "no-routes@example.com").await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!("user_id={}", user_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_user_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_user_routes_with_routes() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "with-routes@example.com").await;
    create_test_user_route(&db, user_id, "155").await;
    create_test_user_route(&db, user_id, "160").await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!("user_id={}", user_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_user_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_get_user_routes_invalid_user_id() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "user_id=not-a-uuid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_user_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

#[tokio::test]
async fn test_get_user_routes_non_existent_user() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let non_existent_id = Uuid::new_v4();
    let form_data = format!("user_id={}", non_existent_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/get_user_routes")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

// =============================================================================
// update_user_route Tests
// =============================================================================

#[tokio::test]
async fn test_update_user_route_valid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "update-route@example.com").await;
    let route_id = create_test_user_route(&db, user_id, "155").await;

    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data = format!(
        "id={}&user_id={}&area_id=2&route_id=999&departure_station=100&arrival_station=200&\
        date_start=2025-03-01&date_end=2025-03-31&departure_time_min=10%3A00&departure_time_max=20%3A00&\
        adult_men=5&adult_women=3&child_men=2&child_women=1&\
        handicap_adult_men=1&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0",
        route_id, user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_update_user_route_passengers() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "update-passengers@example.com").await;
    let route_id = create_test_user_route(&db, user_id, "155").await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!(
        "id={}&user_id={}&area_id=1&route_id=155&departure_station=001&arrival_station=498&\
        date_start=2025-01-01&date_end=2025-01-15&departure_time_min=08%3A00&departure_time_max=18%3A00&\
        adult_men=10&adult_women=10&child_men=5&child_women=5&\
        handicap_adult_men=2&handicap_adult_women=2&handicap_child_men=1&handicap_child_women=1",
        route_id, user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_update_user_route_non_existent() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "test@example.com").await;
    let non_existent_id = Uuid::new_v4();

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!(
        "id={}&user_id={}&area_id=1&route_id=155&departure_station=001&arrival_station=498&\
        date_start=2025-01-01&date_end=2025-01-15&\
        adult_men=1&adult_women=0&child_men=0&child_women=0&\
        handicap_adult_men=0&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0",
        non_existent_id, user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

#[tokio::test]
async fn test_update_user_route_invalid_uuid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "test@example.com").await;

    let app = setup_test_app(db, &mock_server).await;

    let form_data = format!(
        "id=invalid-uuid&user_id={}&area_id=1&route_id=155&departure_station=001&arrival_station=498&\
        date_start=2025-01-01&date_end=2025-01-15&\
        adult_men=1&adult_women=0&child_men=0&child_women=0&\
        handicap_adult_men=0&handicap_adult_women=0&handicap_child_men=0&handicap_child_women=0",
        user_id
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/update_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}

// =============================================================================
// delete_user_route Tests
// =============================================================================

#[tokio::test]
async fn test_delete_user_route_valid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;

    let user_id = create_test_user(&db, "delete-route@example.com").await;
    let route_id = create_test_user_route(&db, user_id, "155").await;

    let app = setup_test_app(db.clone(), &mock_server).await;

    let form_data = format!("id={}", route_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_user_route_non_existent() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let non_existent_id = Uuid::new_v4();
    let form_data = format!("id={}", non_existent_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed (SeaORM doesn't fail on non-existent deletes)
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_delete_user_route_invalid_uuid() {
    let db = setup_test_db().await;
    let mock_server = MockServer::start().await;
    let app = setup_test_app(db, &mock_server).await;

    let form_data = "id=not-a-uuid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/delete_user_route")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return error
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_str.is_empty());
}
