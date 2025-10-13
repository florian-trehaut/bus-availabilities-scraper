mod config;
mod error;
mod html_parser;
mod scraper;
mod types;

use config::Config;
use scraper::BusScraper;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    info!(
        "Configuration loaded: base_url={}, interval={}s, area_id={}",
        config.base_url, config.scrape_interval_secs, config.request.area_id
    );

    let scraper = BusScraper::new(config.base_url)?;
    info!("Scraper initialized, starting periodic checks...");

    let mut interval = tokio::time::interval(Duration::from_secs(config.scrape_interval_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        match scraper.check_availability_full(&config.request).await {
            Ok(schedules) => {
                info!(
                    "Availability check completed. Found {} buses with available seats",
                    schedules.len()
                );

                for schedule in &schedules {
                    info!(
                        "Bus {} | {} -> {} | Departs: {}",
                        schedule.bus_number,
                        schedule.departure_station,
                        schedule.arrival_station,
                        schedule.departure_time
                    );

                    for plan in &schedule.available_plans {
                        let seats_info = match &plan.availability {
                            crate::types::SeatAvailability::Available { remaining_seats } => {
                                match remaining_seats {
                                    Some(n) => format!("{} seats remaining", n),
                                    None => "Seats available".to_string(),
                                }
                            }
                            crate::types::SeatAvailability::SoldOut => "Sold out".to_string(),
                            crate::types::SeatAvailability::Unknown => "Unknown".to_string(),
                        };

                        info!(
                            "  Plan {}: {} yen - {}",
                            plan.plan_id, plan.price, seats_info
                        );
                    }
                }

                if let Ok(json) = serde_json::to_string_pretty(&schedules) {
                    info!("Full schedules JSON:\n{}", json);
                }
            }
            Err(e) => {
                error!("Failed to check availability: {}", e);
            }
        }

        interval.tick().await;
    }
}
