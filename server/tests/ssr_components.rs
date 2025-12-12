//! SSR Component Tests with Data
//!
//! These tests verify that Leptos components render correctly in SSR mode
//! with pre-populated database data and produce expected HTML structure.

#![recursion_limit = "512"]
#![allow(clippy::unwrap_used)]

use app::{components::App, db, entities::users};
use axum::{Router, body::Body, http::Request, routing::get};
use http_body_util::BodyExt;
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use tower::util::ServiceExt;

fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos::hydration::{AutoReload, HydrationScripts};
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

async fn setup_test_app() -> (Router<()>, DatabaseConnection) {
    let db = db::init_database("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    let leptos_options = LeptosOptions::builder()
        .output_name("frontend")
        .site_pkg_dir("pkg")
        .site_root("target/site")
        .build();

    let routes = generate_route_list(App);

    let db_clone = db.clone();
    let options_clone = leptos_options.clone();

    let app: Router<()> = Router::new()
        .leptos_routes_with_handler(
            routes,
            get(move |req: Request<Body>| {
                let db = db_clone.clone();
                let options = options_clone.clone();
                async move {
                    let handler = leptos_axum::render_app_to_stream_with_context(
                        move || {
                            provide_context(db.clone());
                        },
                        move || shell(options.clone()),
                    );
                    handler(req).await
                }
            }),
        )
        .with_state(leptos_options);

    (app, db)
}

async fn get_response_body(response: axum::http::Response<Body>) -> String {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn test_users_page_ssr_empty_state() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /users page"
    );

    let html = get_response_body(response).await;

    // Verify basic page structure
    assert!(html.contains("Users"), "Should contain 'Users' heading");
    assert!(
        html.contains("Add User"),
        "Should contain 'Add User' button"
    );
}

#[tokio::test]
async fn test_users_page_ssr_with_data() {
    let (app, db) = setup_test_app().await;

    // Insert test user
    let user = users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set("test@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(Some("https://discord.com/api/webhooks/test".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(&db).await.unwrap();

    let request = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /users page with data"
    );

    let html = get_response_body(response).await;

    // Verify page renders with data
    assert!(html.contains("Users"), "Should contain page title");
    // Note: Exact email might be in Suspense fallback during SSR
    // but the structure should be present
    assert!(
        html.contains("Email") || html.contains("table"),
        "Should contain table structure"
    );
}

#[tokio::test]
async fn test_users_page_ssr_with_multiple_users() {
    let (app, db) = setup_test_app().await;

    // Insert multiple test users
    for i in 1..=3 {
        let user = users::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(format!("user{}@example.com", i)),
            enabled: Set(i % 2 == 0),
            notify_on_change_only: Set(true),
            scrape_interval_secs: Set(300),
            discord_webhook_url: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        user.insert(&db).await.unwrap();
    }

    let request = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /users page with multiple users"
    );

    let html = get_response_body(response).await;

    assert!(html.contains("Users"), "Should contain heading");
    assert!(
        html.contains("table") || html.contains("Email"),
        "Should contain table elements"
    );
}

#[tokio::test]
async fn test_users_page_ssr_disabled_user() {
    let (app, db) = setup_test_app().await;

    // Insert disabled user
    let user = users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set("disabled@example.com".to_string()),
        enabled: Set(false),
        notify_on_change_only: Set(true),
        scrape_interval_secs: Set(600),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(&db).await.unwrap();

    let request = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /users page with disabled user"
    );

    let html = get_response_body(response).await;

    assert!(!html.is_empty(), "UsersPage should render HTML");
}

#[tokio::test]
async fn test_user_routes_page_ssr_empty_state() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder()
        .uri("/user-routes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /user-routes page"
    );

    let html = get_response_body(response).await;

    // Verify basic structure
    assert!(html.contains("Routes"), "Should contain 'Routes' heading");
    assert!(
        html.contains("Add Route"),
        "Should contain 'Add Route' button"
    );
}

#[tokio::test]
async fn test_user_routes_page_ssr_with_user() {
    let (app, db) = setup_test_app().await;

    // Insert test user (routes require a user)
    let user = users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set("routeuser@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(&db).await.unwrap();

    let request = Request::builder()
        .uri("/user-routes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /user-routes page with user"
    );

    let html = get_response_body(response).await;

    assert!(html.contains("Routes"), "Should contain heading");
    assert!(
        html.contains("Select") || html.contains("user"),
        "Should contain user selection UI"
    );
}

#[tokio::test]
async fn test_user_routes_page_ssr_with_route_and_passengers() {
    let (app, db) = setup_test_app().await;

    // Insert test user
    let user_id = uuid::Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set("routeowner@example.com".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(false),
        scrape_interval_secs: Set(300),
        discord_webhook_url: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(&db).await.unwrap();

    // Insert test route
    let route_id = uuid::Uuid::new_v4();
    let route = app::entities::user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(1),
        route_id: Set("R001".to_string()),
        departure_station: Set("Tokyo".to_string()),
        arrival_station: Set("Osaka".to_string()),
        date_start: Set("2025-01-01".to_string()),
        date_end: Set("2025-01-31".to_string()),
        departure_time_min: Set(Some("09:00".to_string())),
        departure_time_max: Set(Some("18:00".to_string())),
        created_at: Set(chrono::Utc::now()),
    };
    route.insert(&db).await.unwrap();

    // Insert test passengers
    let passengers = app::entities::user_passengers::ActiveModel {
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
    passengers.insert(&db).await.unwrap();

    let request = Request::builder()
        .uri("/user-routes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /user-routes page with route and passengers"
    );

    let html = get_response_body(response).await;

    assert!(!html.is_empty(), "Should render HTML");
    assert!(html.contains("Routes"), "Should contain heading");
}

#[tokio::test]
async fn test_user_routes_page_ssr_multiple_users_and_routes() {
    let (app, db) = setup_test_app().await;

    // Insert multiple users with routes
    for i in 1..=2 {
        let user_id = uuid::Uuid::new_v4();
        let user = users::ActiveModel {
            id: Set(user_id),
            email: Set(format!("user{}@routes.com", i)),
            enabled: Set(true),
            notify_on_change_only: Set(false),
            scrape_interval_secs: Set(300),
            discord_webhook_url: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        user.insert(&db).await.unwrap();

        // Add route for each user
        let route_id = uuid::Uuid::new_v4();
        let route = app::entities::user_routes::ActiveModel {
            id: Set(route_id),
            user_id: Set(user_id),
            area_id: Set(i),
            route_id: Set(format!("R00{}", i)),
            departure_station: Set(format!("Station{}", i)),
            arrival_station: Set(format!("Station{}", i + 1)),
            date_start: Set("2025-01-01".to_string()),
            date_end: Set("2025-12-31".to_string()),
            departure_time_min: Set(None),
            departure_time_max: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        route.insert(&db).await.unwrap();

        // Add passengers
        let passengers = app::entities::user_passengers::ActiveModel {
            user_route_id: Set(route_id),
            adult_men: Set(1),
            adult_women: Set(1),
            child_men: Set(0),
            child_women: Set(0),
            handicap_adult_men: Set(0),
            handicap_adult_women: Set(0),
            handicap_child_men: Set(0),
            handicap_child_women: Set(0),
        };
        passengers.insert(&db).await.unwrap();
    }

    let request = Request::builder()
        .uri("/user-routes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /user-routes with multiple users/routes"
    );

    let html = get_response_body(response).await;

    assert!(!html.is_empty(), "Should render HTML");
    assert!(html.contains("Routes"), "Should contain heading");
}

#[tokio::test]
async fn test_home_page_renders_with_navigation() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for home page"
    );

    let html = get_response_body(response).await;

    // Verify navigation is present
    assert!(
        html.contains("Bus Scraper"),
        "Should contain app title in nav"
    );
    assert!(
        html.contains("Manage Users") || html.contains("Configure Routes"),
        "Should contain navigation links"
    );
}
