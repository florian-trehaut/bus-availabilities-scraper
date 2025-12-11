use crate::error::{Result, ScraperError};
use crate::types::{DateRange, PassengerCount, ScrapeRequest, TimeFilter};
use chrono::Local;

#[derive(Debug, Clone)]
pub struct Config {
    pub scrape_interval_secs: u64,
    pub request: ScrapeRequest,
    pub discord_webhook_url: Option<String>,
    pub notify_on_change_only: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let scrape_interval_secs = dotenvy::var("SCRAPE_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| ScraperError::Config("Invalid SCRAPE_INTERVAL_SECS".to_string()))?;

        let area_id = dotenvy::var("AREA_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid AREA_ID".to_string()))?;

        let route_id = dotenvy::var("ROUTE_ID")
            .map_err(|_| ScraperError::Config("ROUTE_ID is required".to_string()))?
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid ROUTE_ID".to_string()))?;

        let departure_station = dotenvy::var("DEPARTURE_STATION")
            .map_err(|_| ScraperError::Config("DEPARTURE_STATION is required".to_string()))?;

        let arrival_station = dotenvy::var("ARRIVAL_STATION")
            .map_err(|_| ScraperError::Config("ARRIVAL_STATION is required".to_string()))?;

        let date_start = dotenvy::var("DATE_START")
            .unwrap_or_else(|_| Local::now().format("%Y%m%d").to_string());

        let date_end = dotenvy::var("DATE_END").unwrap_or_else(|_| {
            Local::now()
                .checked_add_signed(chrono::Duration::days(7))
                .map_or_else(
                    || Local::now().format("%Y%m%d").to_string(),
                    |d| d.format("%Y%m%d").to_string(),
                )
        });

        let date_range = DateRange {
            start: date_start,
            end: date_end,
        };

        let passengers = PassengerCount {
            adult_men: parse_env_u8("ADULT_MEN", 1)?,
            adult_women: parse_env_u8("ADULT_WOMEN", 0)?,
            child_men: parse_env_u8("CHILD_MEN", 0)?,
            child_women: parse_env_u8("CHILD_WOMEN", 0)?,
            handicap_adult_men: parse_env_u8("HANDICAP_ADULT_MEN", 0)?,
            handicap_adult_women: parse_env_u8("HANDICAP_ADULT_WOMEN", 0)?,
            handicap_child_men: parse_env_u8("HANDICAP_CHILD_MEN", 0)?,
            handicap_child_women: parse_env_u8("HANDICAP_CHILD_WOMEN", 0)?,
        };

        passengers.validate()?;

        let time_min = dotenvy::var("DEPARTURE_TIME_MIN")
            .ok()
            .filter(|s| !s.is_empty());
        let time_max = dotenvy::var("DEPARTURE_TIME_MAX")
            .ok()
            .filter(|s| !s.is_empty());

        let time_filter = match (time_min, time_max) {
            (None, None) => None,
            (min, max) => Some(TimeFilter {
                departure_min: min,
                departure_max: max,
            }),
        };

        let discord_webhook_url = dotenvy::var("DISCORD_WEBHOOK_URL")
            .ok()
            .filter(|s| !s.is_empty());

        let notify_on_change_only = dotenvy::var("NOTIFY_ON_CHANGE_ONLY")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        Ok(Config {
            scrape_interval_secs,
            discord_webhook_url,
            notify_on_change_only,
            request: ScrapeRequest {
                area_id,
                route_id,
                departure_station,
                arrival_station,
                date_range,
                passengers,
                time_filter,
            },
        })
    }
}

fn parse_env_u8(key: &str, default: u8) -> Result<u8> {
    dotenvy::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u8>()
        .map_err(|_| ScraperError::Config(format!("Invalid {key}")))
}
