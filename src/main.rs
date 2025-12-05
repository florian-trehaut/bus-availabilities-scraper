mod config;
mod db;
mod entities;
mod error;
mod html_parser;
mod notifier;
mod repositories;
mod scraper;
mod seed;
mod seed_routes;
mod types;

use crate::repositories::{
    get_all_active_user_routes, get_route_state, get_station_name, update_route_state,
    UserRouteWithDetails,
};
use migration::{Migrator, MigratorTrait};
use notifier::{DiscordNotifier, NotificationContext};
use scraper::BusScraper;
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use types::{DateRange, PassengerCount, ScrapeRequest, SeatAvailability, TimeFilter};
use xxhash_rust::xxh64::Xxh64;

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

    let should_seed_routes = dotenvy::var("SEED_ROUTES_CATALOG")
        .map(|v| v == "true")
        .unwrap_or(false);

    if should_seed_routes {
        info!("Seeding routes catalog from Highway Bus API...");
        let temp_scraper = scraper::BusScraper::new(
            dotenvy::var("BASE_URL")
                .unwrap_or_else(|_| "https://www.highwaybus.com/gp".to_string()),
        )?;
        if let Err(e) = seed_routes::seed_routes_catalog(&db, &temp_scraper).await {
            error!("Failed to seed routes catalog: {}", e);
        }
    }

    let should_seed = dotenvy::var("SEED_FROM_ENV")
        .map(|v| v == "true")
        .unwrap_or(false);

    if should_seed {
        info!("Seeding database from .env configuration...");
        seed::seed_from_env(&db).await?;
    }

    let base_url =
        dotenvy::var("BASE_URL").unwrap_or_else(|_| "https://www.highwaybus.com/gp".to_string());

    let scraper = Arc::new(BusScraper::new(base_url.clone())?);
    let db = Arc::new(db);

    let user_routes = get_all_active_user_routes(&db).await?;

    if user_routes.is_empty() {
        warn!("No active user routes found in database");
        warn!("Set SEED_FROM_ENV=true to seed from .env configuration");
        return Ok(());
    }

    info!("Starting tracking for {} user route(s)", user_routes.len());

    let unique_users: HashSet<String> = user_routes.iter().map(|r| r.email.clone()).collect();

    let unique_webhooks: HashSet<String> = user_routes
        .iter()
        .filter_map(|r| r.discord_webhook_url.clone())
        .collect();

    let notifier = DiscordNotifier::new()?;
    for webhook_url in &unique_webhooks {
        if let Err(e) = notifier
            .send_startup_notification(webhook_url, unique_users.len(), user_routes.len())
            .await
        {
            error!("Failed to send startup notification: {}", e);
        }
    }

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let mut handles = Vec::new();

    for user_route in user_routes {
        let shutdown_rx = shutdown_tx.subscribe();
        let tracker = UserTracker {
            user_route,
            scraper: Arc::clone(&scraper),
            db: Arc::clone(&db),
            notifier: DiscordNotifier::new()?,
        };

        let handle = tokio::spawn(async move {
            tracker.run_with_shutdown(shutdown_rx).await;
        });

        handles.push(handle);
    }

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    drop(shutdown_tx);

    let shutdown_timeout = Duration::from_secs(5);
    if tokio::time::timeout(shutdown_timeout, async {
        for handle in handles {
            let _ = handle.await;
        }
    })
    .await
    .is_err()
    {
        warn!("Shutdown timeout reached, some tasks may not have completed");
    }

    info!("Shutdown complete");
    Ok(())
}

struct UserTracker {
    user_route: UserRouteWithDetails,
    scraper: Arc<BusScraper>,
    db: Arc<DatabaseConnection>,
    notifier: DiscordNotifier,
}

impl UserTracker {
    async fn run_with_shutdown(self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!(
            "Starting tracker for user {} (route {})",
            self.user_route.email, self.user_route.user_route_id
        );

        let interval_secs = u64::try_from(self.user_route.scrape_interval_secs).unwrap_or(300);
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.check_and_notify().await {
                        error!(
                            "Error checking availability for user {} route {}: {}",
                            self.user_route.email, self.user_route.user_route_id, e
                        );
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!(
                        "Tracker for user {} (route {}) shutting down",
                        self.user_route.email, self.user_route.user_route_id
                    );
                    break;
                }
            }
        }
    }

    async fn check_and_notify(&self) -> error::Result<()> {
        let request = self.build_scrape_request()?;

        let schedules = self.scraper.check_availability_full(&request).await?;
        let total_schedules = schedules.len();

        let schedules_with_seats: Vec<_> = schedules
            .into_iter()
            .filter(|s| !s.available_plans.is_empty())
            .collect();

        let current_hash = calculate_state_hash(&schedules_with_seats);

        let state = get_route_state(&self.db, self.user_route.user_route_id).await?;

        let hash_str = format!("{}", current_hash);
        let state_changed = state.as_ref().is_none_or(|s| s.last_seen_hash != hash_str);

        let should_notify = if self.user_route.notify_on_change_only {
            state_changed && !schedules_with_seats.is_empty()
        } else {
            !schedules_with_seats.is_empty()
        };

        if should_notify {
            if let Some(ref webhook_url) = self.user_route.discord_webhook_url {
                info!(
                    "Sending notification for user {} - {} buses with seats",
                    self.user_route.email,
                    schedules_with_seats.len()
                );

                let context = self.build_notification_context().await?;
                self.notifier
                    .send_availability_alert(webhook_url, &schedules_with_seats, &context)
                    .await?;

                update_route_state(
                    &self.db,
                    self.user_route.user_route_id,
                    format!("{}", current_hash),
                    true,
                )
                .await?;
            }
        } else {
            if total_schedules > 0 && schedules_with_seats.is_empty() {
                info!(
                    "User {} - Found {} buses but no seats available",
                    self.user_route.email, total_schedules
                );
            }

            if !schedules_with_seats.is_empty() {
                update_route_state(
                    &self.db,
                    self.user_route.user_route_id,
                    format!("{}", current_hash),
                    false,
                )
                .await?;
            }
        }

        Ok(())
    }

    fn build_scrape_request(&self) -> error::Result<ScrapeRequest> {
        use crate::error::ScraperError;

        let area_id = u32::try_from(self.user_route.area_id)
            .map_err(|_| ScraperError::Config("Invalid area_id: negative value".into()))?;
        let route_id = u32::try_from(self.user_route.route_id)
            .map_err(|_| ScraperError::Config("Invalid route_id: negative value".into()))?;

        let passengers = &self.user_route.passengers;
        let convert_passenger = |v: i16, name: &str| -> error::Result<u8> {
            u8::try_from(v)
                .map_err(|_| ScraperError::Config(format!("Invalid {}: value out of range", name)))
        };

        Ok(ScrapeRequest {
            area_id,
            route_id,
            departure_station: self.user_route.departure_station.clone(),
            arrival_station: self.user_route.arrival_station.clone(),
            date_range: DateRange {
                start: self.user_route.date_start.clone(),
                end: self.user_route.date_end.clone(),
            },
            passengers: PassengerCount {
                adult_men: convert_passenger(passengers.adult_men, "adult_men")?,
                adult_women: convert_passenger(passengers.adult_women, "adult_women")?,
                child_men: convert_passenger(passengers.child_men, "child_men")?,
                child_women: convert_passenger(passengers.child_women, "child_women")?,
                handicap_adult_men: convert_passenger(
                    passengers.handicap_adult_men,
                    "handicap_adult_men",
                )?,
                handicap_adult_women: convert_passenger(
                    passengers.handicap_adult_women,
                    "handicap_adult_women",
                )?,
                handicap_child_men: convert_passenger(
                    passengers.handicap_child_men,
                    "handicap_child_men",
                )?,
                handicap_child_women: convert_passenger(
                    passengers.handicap_child_women,
                    "handicap_child_women",
                )?,
            },
            time_filter: match (
                &self.user_route.departure_time_min,
                &self.user_route.departure_time_max,
            ) {
                (None, None) => None,
                (min, max) => Some(TimeFilter {
                    departure_min: min.clone(),
                    departure_max: max.clone(),
                }),
            },
        })
    }

    async fn build_notification_context(&self) -> error::Result<NotificationContext> {
        let departure_name = get_station_name(&self.db, &self.user_route.departure_station)
            .await?
            .unwrap_or_else(|| format!("Station {}", self.user_route.departure_station));

        let arrival_name = get_station_name(&self.db, &self.user_route.arrival_station)
            .await?
            .unwrap_or_else(|| format!("Station {}", self.user_route.arrival_station));

        Ok(NotificationContext {
            departure_station_name: departure_name,
            arrival_station_name: arrival_name,
            date_range: (
                self.user_route.date_start.clone(),
                self.user_route.date_end.clone(),
            ),
            passenger_count: self.user_route.passengers.total() as u8,
            time_filter: match (
                &self.user_route.departure_time_min,
                &self.user_route.departure_time_max,
            ) {
                (Some(min), Some(max)) => Some((min.clone(), max.clone())),
                _ => None,
            },
        })
    }
}

fn calculate_state_hash(schedules: &[types::BusSchedule]) -> u64 {
    let mut hasher = Xxh64::new(0);

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
