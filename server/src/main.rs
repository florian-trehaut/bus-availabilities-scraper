#![recursion_limit = "512"]

mod tracker;

use app::{components::App, db, scraper::BusScraper};
use axum::extract::FromRef;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::Request,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list, handle_server_fns_with_context};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    db: DatabaseConnection,
    scraper: Arc<BusScraper>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = dotenvy::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/bus_scraper.db?mode=rwc".to_string());

    info!("Connecting to database: {}", database_url);
    let db = db::init_database(&database_url).await?;

    info!("Running migrations...");
    Migrator::up(&db, None).await?;

    // Create scraper for live API fetching
    let base_url =
        dotenvy::var("BASE_URL").unwrap_or_else(|_| "https://www.highwaybus.com/gp".to_string());
    let scraper = Arc::new(BusScraper::new(base_url)?);

    let should_seed = dotenvy::var("SEED_FROM_ENV")
        .map(|v| v == "true")
        .unwrap_or(false);

    if should_seed {
        info!("Seeding database from .env configuration...");
        app::seed::seed_from_env(&db).await?;
    }

    let leptos_options = LeptosOptions::builder()
        .output_name("frontend")
        .site_pkg_dir("pkg")
        .site_root("target/site")
        .build();

    let state = AppState {
        leptos_options,
        db: db.clone(),
        scraper: scraper.clone(),
    };

    let db_for_tracker = Arc::new(db);

    let enable_tracker = dotenvy::var("ENABLE_TRACKER")
        .map(|v| v == "true")
        .unwrap_or(true);

    if enable_tracker {
        let db_clone = Arc::clone(&db_for_tracker);
        tokio::spawn(async move {
            if let Err(e) = tracker::run_tracker(db_clone).await {
                error!("Tracker error: {}", e);
            }
        });
    }

    let routes = generate_route_list(App);

    let app = Router::new()
        .route(
            "/api/{*fn_name}",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let db = state.db.clone();
                let scraper = state.scraper.clone();
                move || {
                    provide_context(db.clone());
                    provide_context(scraper.clone());
                }
            },
            {
                let options = state.leptos_options.clone();
                move || shell(options.clone())
            },
        )
        .fallback(file_and_error_handler)
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Listening on http://127.0.0.1:3000");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos::hydration::{AutoReload, HydrationScripts};
    use leptos_meta::MetaTags;
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/pkg/frontend.css"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(state.db.clone());
            provide_context(state.scraper.clone());
        },
        req,
    )
    .await
}

async fn file_and_error_handler(State(state): State<AppState>, req: Request<Body>) -> Response {
    let path = req.uri().path();

    if path.starts_with("/pkg") {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Html("<h1>404 Not Found</h1>"),
        )
            .into_response();
    }

    let options = state.leptos_options.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state.db.clone());
            provide_context(state.scraper.clone());
        },
        move || shell(options.clone()),
    );
    handler(req).await.into_response()
}

#[allow(clippy::expect_used)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutting down gracefully...");
}
