use app::{
    error,
    notifier::{DiscordNotifier, NotificationContext},
    repositories::{
        get_all_active_user_routes, get_route_state, get_station_name, update_route_state,
        UserRouteWithDetails,
    },
    scraper::BusScraper,
    types::{self, DateRange, PassengerCount, ScrapeRequest, TimeFilter},
};
use sea_orm::DatabaseConnection;
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn run_tracker(db: Arc<DatabaseConnection>) -> anyhow::Result<()> {
    let base_url =
        dotenvy::var("BASE_URL").unwrap_or_else(|_| "https://www.highwaybus.com/gp".to_string());

    let scraper = Arc::new(BusScraper::new(base_url)?);

    let user_routes = get_all_active_user_routes(&db).await?;

    if user_routes.is_empty() {
        warn!("No active user routes found in database");
        return Ok(());
    }

    info!("Starting tracking for {} user route(s)", user_routes.len());

    let unique_users: HashSet<String> = user_routes.iter().map(|r| r.email.clone()).collect();

    let unique_webhooks: HashSet<String> = user_routes
        .iter()
        .filter_map(|r| r.discord_webhook_url.clone())
        .collect();

    let notifier = DiscordNotifier::new();
    for webhook_url in &unique_webhooks {
        if let Err(e) = notifier
            .send_startup_notification(webhook_url, unique_users.len(), user_routes.len())
            .await
        {
            error!("Failed to send startup notification: {}", e);
        }
    }

    for user_route in user_routes {
        let tracker = UserTracker {
            user_route,
            scraper: Arc::clone(&scraper),
            db: Arc::clone(&db),
            notifier: DiscordNotifier::new(),
        };

        tokio::spawn(async move {
            tracker.run().await;
        });
    }

    Ok(())
}

struct UserTracker {
    user_route: UserRouteWithDetails,
    scraper: Arc<BusScraper>,
    db: Arc<DatabaseConnection>,
    notifier: DiscordNotifier,
}

impl UserTracker {
    async fn run(self) {
        info!(
            "Starting tracker for user {} (route {})",
            self.user_route.email, self.user_route.user_route_id
        );

        let mut interval = tokio::time::interval(Duration::from_secs(
            self.user_route.scrape_interval_secs as u64,
        ));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            if let Err(e) = self.check_and_notify().await {
                error!(
                    "Error checking availability for user {} route {}: {}",
                    self.user_route.email, self.user_route.user_route_id, e
                );
            }
        }
    }

    async fn check_and_notify(&self) -> error::Result<()> {
        let request = self.build_scrape_request();

        let schedules = self.scraper.check_availability_full(&request).await?;

        let schedules_with_seats: Vec<_> = schedules
            .iter()
            .filter(|s| !s.available_plans.is_empty())
            .cloned()
            .collect();

        let current_hash = calculate_state_hash(&schedules_with_seats);

        let state = get_route_state(&self.db, self.user_route.user_route_id).await?;

        let hash_str = format!("{current_hash}");
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
                    format!("{current_hash}"),
                    true,
                )
                .await?;
            }
        } else {
            if !schedules.is_empty() && schedules_with_seats.is_empty() {
                info!(
                    "User {} - Found {} buses but no seats available",
                    self.user_route.email,
                    schedules.len()
                );
            }

            if !schedules_with_seats.is_empty() {
                update_route_state(
                    &self.db,
                    self.user_route.user_route_id,
                    format!("{current_hash}"),
                    false,
                )
                .await?;
            }
        }

        Ok(())
    }

    fn build_scrape_request(&self) -> ScrapeRequest {
        ScrapeRequest {
            area_id: self.user_route.area_id as u32,
            route_id: self.user_route.route_id as u32,
            departure_station: self.user_route.departure_station.clone(),
            arrival_station: self.user_route.arrival_station.clone(),
            date_range: DateRange {
                start: self.user_route.date_start.clone(),
                end: self.user_route.date_end.clone(),
            },
            passengers: PassengerCount {
                adult_men: self.user_route.passengers.adult_men as u8,
                adult_women: self.user_route.passengers.adult_women as u8,
                child_men: self.user_route.passengers.child_men as u8,
                child_women: self.user_route.passengers.child_women as u8,
                handicap_adult_men: self.user_route.passengers.handicap_adult_men as u8,
                handicap_adult_women: self.user_route.passengers.handicap_adult_women as u8,
                handicap_child_men: self.user_route.passengers.handicap_child_men as u8,
                handicap_child_women: self.user_route.passengers.handicap_child_women as u8,
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
        }
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
    let mut hasher = DefaultHasher::new();

    for schedule in schedules {
        schedule.departure_date.hash(&mut hasher);
        schedule.departure_time.hash(&mut hasher);

        for plan in &schedule.available_plans {
            plan.plan_id.hash(&mut hasher);
            plan.price.hash(&mut hasher);

            let types::SeatAvailability::Available { remaining_seats } = &plan.availability;
            remaining_seats.hash(&mut hasher);
        }
    }

    hasher.finish()
}
