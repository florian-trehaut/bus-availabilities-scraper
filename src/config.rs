use crate::error::{Result, ScraperError};
use crate::types::{DateRange, PassengerCount, ScrapeRequest, TimeFilter};
use chrono::Local;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub scrape_interval_secs: u64,
    pub request: ScrapeRequest,
}

impl Config {
    #[allow(clippy::disallowed_methods)] // env::var is used with proper error handling
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let base_url =
            env::var("BASE_URL").unwrap_or_else(|_| "https://www.highwaybus.com/gp".to_string());

        let scrape_interval_secs = env::var("SCRAPE_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| ScraperError::Config("Invalid SCRAPE_INTERVAL_SECS".to_string()))?;

        let area_id = env::var("AREA_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid AREA_ID".to_string()))?;

        let route_id = env::var("ROUTE_ID")
            .map_err(|_| ScraperError::Config("ROUTE_ID is required".to_string()))?
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid ROUTE_ID".to_string()))?;

        let departure_station = env::var("DEPARTURE_STATION")
            .map_err(|_| ScraperError::Config("DEPARTURE_STATION is required".to_string()))?;

        let arrival_station = env::var("ARRIVAL_STATION")
            .map_err(|_| ScraperError::Config("ARRIVAL_STATION is required".to_string()))?;

        let date_start =
            env::var("DATE_START").unwrap_or_else(|_| Local::now().format("%Y%m%d").to_string());

        let date_end = env::var("DATE_END").unwrap_or_else(|_| {
            Local::now()
                .checked_add_signed(chrono::Duration::days(7))
                .unwrap()
                .format("%Y%m%d")
                .to_string()
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

        let time_min = env::var("DEPARTURE_TIME_MIN")
            .ok()
            .filter(|s| !s.is_empty());
        let time_max = env::var("DEPARTURE_TIME_MAX")
            .ok()
            .filter(|s| !s.is_empty());

        let time_filter = match (time_min, time_max) {
            (None, None) => None,
            (min, max) => Some(TimeFilter {
                departure_min: min,
                departure_max: max,
            }),
        };

        Ok(Config {
            base_url,
            scrape_interval_secs,
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

#[allow(clippy::disallowed_methods)] // env::var is used with proper error handling
fn parse_env_u8(key: &str, default: u8) -> Result<u8> {
    env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u8>()
        .map_err(|_| ScraperError::Config(format!("Invalid {}", key)))
}
