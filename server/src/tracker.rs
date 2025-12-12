use app::{
    error,
    notifier::{DiscordNotifier, NotificationContext},
    repositories::{
        UserRouteWithDetails, get_all_active_user_routes, get_route_state, update_route_state,
    },
    scraper::BusScraper,
    types::{self, DateRange, PassengerCount, ScrapeRequest, TimeFilter},
};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Station name cache: `station_id` -> `station_name`
pub type StationCache = Arc<tokio::sync::RwLock<HashMap<String, String>>>;

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

    // Build station cache at startup by fetching all stations for user routes
    let station_cache: StationCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    info!("Building station name cache from API...");
    for user_route in &user_routes {
        if let Err(e) = populate_station_cache(&scraper, &station_cache, &user_route.route_id).await
        {
            warn!(
                "Failed to cache stations for route {}: {}",
                user_route.route_id, e
            );
        }
    }
    info!(
        "Station cache built with {} entries",
        station_cache.read().await.len()
    );

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
            station_cache: Arc::clone(&station_cache),
            notifier: DiscordNotifier::new(),
        };

        tokio::spawn(async move {
            tracker.run().await;
        });
    }

    Ok(())
}

async fn populate_station_cache(
    scraper: &BusScraper,
    cache: &StationCache,
    route_id: &str,
) -> anyhow::Result<()> {
    let stations = scraper.fetch_departure_stations(route_id).await?;
    let mut cache_lock = cache.write().await;
    for station in stations {
        cache_lock.insert(station.id, station.name);
    }
    Ok(())
}

struct UserTracker {
    user_route: UserRouteWithDetails,
    scraper: Arc<BusScraper>,
    db: Arc<DatabaseConnection>,
    station_cache: StationCache,
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
            route_id: self.user_route.route_id.parse().unwrap_or(0),
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
        let cache = self.station_cache.read().await;

        let departure_name = cache
            .get(&self.user_route.departure_station)
            .cloned()
            .unwrap_or_else(|| format!("Station {}", self.user_route.departure_station));

        let arrival_name = cache
            .get(&self.user_route.arrival_station)
            .cloned()
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

pub fn calculate_state_hash(schedules: &[types::BusSchedule]) -> u64 {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use app::types::{BusSchedule, PricingPlan, SeatAvailability};

    fn create_test_schedule(
        departure_date: &str,
        departure_time: &str,
        plan_id: u32,
        price: u32,
        remaining_seats: Option<u32>,
    ) -> BusSchedule {
        BusSchedule {
            bus_number: "Bus_1".to_string(),
            route_name: "Test Route".to_string(),
            departure_station: "001".to_string(),
            departure_date: departure_date.to_string(),
            departure_time: departure_time.to_string(),
            arrival_station: "064".to_string(),
            arrival_date: departure_date.to_string(),
            arrival_time: "10:00".to_string(),
            way_no: 1,
            available_plans: vec![PricingPlan {
                plan_id,
                plan_index: 0,
                plan_name: "Standard".to_string(),
                price,
                display_price: format!("{price}円"),
                availability: SeatAvailability::Available { remaining_seats },
            }],
        }
    }

    #[test]
    fn test_calculate_state_hash_empty_schedules() {
        let schedules: Vec<BusSchedule> = vec![];
        let hash = calculate_state_hash(&schedules);

        // Empty schedules should produce a consistent hash
        let hash2 = calculate_state_hash(&schedules);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_calculate_state_hash_single_schedule() {
        let schedules = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let hash = calculate_state_hash(&schedules);

        // Same schedule should produce same hash
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let hash2 = calculate_state_hash(&schedules2);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_calculate_state_hash_different_dates() {
        let schedules1 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let schedules2 = vec![create_test_schedule(
            "20250116",
            "08:30",
            12345,
            2100,
            Some(5),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_different_times() {
        let schedules1 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "09:00",
            12345,
            2100,
            Some(5),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_different_prices() {
        let schedules1 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2500,
            Some(5),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_different_remaining_seats() {
        let schedules1 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(3),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_none_vs_some_seats() {
        let schedules1 = vec![create_test_schedule("20250115", "08:30", 12345, 2100, None)];
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_multiple_schedules() {
        let schedules = vec![
            create_test_schedule("20250115", "08:30", 12345, 2100, Some(5)),
            create_test_schedule("20250115", "10:00", 12346, 2200, Some(3)),
        ];
        let hash = calculate_state_hash(&schedules);

        // Same schedules in same order should produce same hash
        let schedules2 = vec![
            create_test_schedule("20250115", "08:30", 12345, 2100, Some(5)),
            create_test_schedule("20250115", "10:00", 12346, 2200, Some(3)),
        ];
        let hash2 = calculate_state_hash(&schedules2);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_calculate_state_hash_order_matters() {
        let schedules1 = vec![
            create_test_schedule("20250115", "08:30", 12345, 2100, Some(5)),
            create_test_schedule("20250115", "10:00", 12346, 2200, Some(3)),
        ];
        let schedules2 = vec![
            create_test_schedule("20250115", "10:00", 12346, 2200, Some(3)),
            create_test_schedule("20250115", "08:30", 12345, 2100, Some(5)),
        ];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        // Order matters in hash calculation
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_state_hash_no_plans() {
        let mut schedule = create_test_schedule("20250115", "08:30", 12345, 2100, Some(5));
        schedule.available_plans.clear();
        let schedules = vec![schedule];

        let hash = calculate_state_hash(&schedules);

        // Should still produce a valid hash based on date/time
        assert!(hash > 0);
    }

    #[test]
    fn test_calculate_state_hash_different_plan_ids() {
        let schedules1 = vec![create_test_schedule(
            "20250115",
            "08:30",
            12345,
            2100,
            Some(5),
        )];
        let schedules2 = vec![create_test_schedule(
            "20250115",
            "08:30",
            99999,
            2100,
            Some(5),
        )];

        let hash1 = calculate_state_hash(&schedules1);
        let hash2 = calculate_state_hash(&schedules2);

        assert_ne!(hash1, hash2);
    }
}
