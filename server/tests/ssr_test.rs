//! SSR integration tests
//!
//! These tests verify that server-side rendering works correctly
//! for all routes without panicking.

#![recursion_limit = "512"]
#![allow(clippy::unwrap_used)]

use app::{components::App, db};
use axum::{Router, body::Body, http::Request, routing::get};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
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

/// Test that the /users page renders without panicking
///
/// This test reproduces the `spawn_local` panic that occurs when
/// `ServerAction` or Effect is used incorrectly in SSR context.
#[tokio::test]
async fn test_users_page_ssr_no_panic() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200, not panic
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for /users page SSR"
    );
}

/// Test that the home page renders without panicking
#[tokio::test]
async fn test_home_page_ssr_no_panic() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected 200 OK for home page SSR"
    );
}
