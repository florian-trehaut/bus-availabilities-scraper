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
        Self::from_env_internal()
    }

    /// Create config from environment variables without loading .env file.
    /// Used for testing with controlled environment.
    pub fn from_env_with_dotenv(load_dotenv: bool) -> Result<Self> {
        if load_dotenv {
            dotenvy::dotenv().ok();
        }
        Self::from_env_internal()
    }

    fn from_env_internal() -> Result<Self> {
        let scrape_interval_secs = std::env::var("SCRAPE_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| ScraperError::Config("Invalid SCRAPE_INTERVAL_SECS".to_string()))?;

        let area_id = std::env::var("AREA_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid AREA_ID".to_string()))?;

        let route_id = std::env::var("ROUTE_ID")
            .map_err(|_| ScraperError::Config("ROUTE_ID is required".to_string()))?
            .parse::<u32>()
            .map_err(|_| ScraperError::Config("Invalid ROUTE_ID".to_string()))?;

        let departure_station = std::env::var("DEPARTURE_STATION")
            .map_err(|_| ScraperError::Config("DEPARTURE_STATION is required".to_string()))?;

        let arrival_station = std::env::var("ARRIVAL_STATION")
            .map_err(|_| ScraperError::Config("ARRIVAL_STATION is required".to_string()))?;

        let date_start = std::env::var("DATE_START")
            .unwrap_or_else(|_| Local::now().format("%Y%m%d").to_string());

        let date_end = std::env::var("DATE_END").unwrap_or_else(|_| {
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

        let time_min = std::env::var("DEPARTURE_TIME_MIN")
            .ok()
            .filter(|s| !s.is_empty());
        let time_max = std::env::var("DEPARTURE_TIME_MAX")
            .ok()
            .filter(|s| !s.is_empty());

        let time_filter = match (time_min, time_max) {
            (None, None) => None,
            (min, max) => Some(TimeFilter {
                departure_min: min,
                departure_max: max,
            }),
        };

        let discord_webhook_url = std::env::var("DISCORD_WEBHOOK_URL")
            .ok()
            .filter(|s| !s.is_empty());

        let notify_on_change_only = std::env::var("NOTIFY_ON_CHANGE_ONLY")
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
    std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u8>()
        .map_err(|_| ScraperError::Config(format!("Invalid {key}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serial_test::serial;

    // === parse_env_u8 TESTS ===

    #[test]
    #[serial]
    fn test_parse_env_u8_returns_default_when_not_set() {
        temp_env::with_var_unset("TEST_PARSE_U8_VAR", || {
            let result = parse_env_u8("TEST_PARSE_U8_VAR", 5);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 5);
        });
    }

    #[test]
    #[serial]
    fn test_parse_env_u8_parses_valid_value() {
        temp_env::with_var("TEST_PARSE_U8_VAR", Some("42"), || {
            let result = parse_env_u8("TEST_PARSE_U8_VAR", 0);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        });
    }

    #[test]
    #[serial]
    fn test_parse_env_u8_returns_error_for_invalid_value() {
        temp_env::with_var("TEST_PARSE_U8_VAR", Some("not_a_number"), || {
            let result = parse_env_u8("TEST_PARSE_U8_VAR", 0);
            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn test_parse_env_u8_returns_error_for_negative_value() {
        temp_env::with_var("TEST_PARSE_U8_VAR", Some("-1"), || {
            let result = parse_env_u8("TEST_PARSE_U8_VAR", 0);
            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn test_parse_env_u8_returns_error_for_overflow() {
        temp_env::with_var("TEST_PARSE_U8_VAR", Some("256"), || {
            let result = parse_env_u8("TEST_PARSE_U8_VAR", 0);
            assert!(result.is_err());
        });
    }

    // === Config::from_env TESTS ===

    // Helper to clear all config-related env vars
    fn all_config_vars_cleared() -> Vec<(&'static str, Option<&'static str>)> {
        vec![
            ("ROUTE_ID", None),
            ("DEPARTURE_STATION", None),
            ("ARRIVAL_STATION", None),
            ("AREA_ID", None),
            ("SCRAPE_INTERVAL_SECS", None),
            ("DATE_START", None),
            ("DATE_END", None),
            ("ADULT_MEN", None),
            ("ADULT_WOMEN", None),
            ("CHILD_MEN", None),
            ("CHILD_WOMEN", None),
            ("HANDICAP_ADULT_MEN", None),
            ("HANDICAP_ADULT_WOMEN", None),
            ("HANDICAP_CHILD_MEN", None),
            ("HANDICAP_CHILD_WOMEN", None),
            ("DEPARTURE_TIME_MIN", None),
            ("DEPARTURE_TIME_MAX", None),
            ("DISCORD_WEBHOOK_URL", None),
            ("NOTIFY_ON_CHANGE_ONLY", None),
        ]
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_route_id_returns_error() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("002")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("ROUTE_ID"));
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_departure_station_returns_error() {
        let mut vars = all_config_vars_cleared();
        vars.extend([("ROUTE_ID", Some("155")), ("ARRIVAL_STATION", Some("002"))]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("DEPARTURE_STATION"));
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_arrival_station_returns_error() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("ARRIVAL_STATION"));
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_minimal_required_vars() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("498")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_ok());
            let config = result.unwrap();

            // Verify defaults
            assert_eq!(config.scrape_interval_secs, 300);
            assert_eq!(config.request.area_id, 1);
            assert_eq!(config.request.route_id, 155);
            assert_eq!(config.request.departure_station, "001");
            assert_eq!(config.request.arrival_station, "498");
            assert_eq!(config.request.passengers.adult_men, 1);
            assert_eq!(config.request.passengers.adult_women, 0);
            assert!(config.request.time_filter.is_none());
            assert!(config.discord_webhook_url.is_none());
            assert!(config.notify_on_change_only);
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_all_vars_set() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("110")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("064")),
            ("AREA_ID", Some("2")),
            ("SCRAPE_INTERVAL_SECS", Some("600")),
            ("DATE_START", Some("2025-01-01")),
            ("DATE_END", Some("2025-01-07")),
            ("ADULT_MEN", Some("2")),
            ("ADULT_WOMEN", Some("1")),
            ("CHILD_MEN", Some("0")),
            ("CHILD_WOMEN", Some("0")),
            ("HANDICAP_ADULT_MEN", Some("0")),
            ("HANDICAP_ADULT_WOMEN", Some("0")),
            ("HANDICAP_CHILD_MEN", Some("0")),
            ("HANDICAP_CHILD_WOMEN", Some("0")),
            ("DEPARTURE_TIME_MIN", Some("08:00")),
            ("DEPARTURE_TIME_MAX", Some("12:00")),
            ("DISCORD_WEBHOOK_URL", Some("https://discord.com/webhook")),
            ("NOTIFY_ON_CHANGE_ONLY", Some("false")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_ok());
            let config = result.unwrap();

            assert_eq!(config.scrape_interval_secs, 600);
            assert_eq!(config.request.area_id, 2);
            assert_eq!(config.request.route_id, 110);
            assert_eq!(config.request.passengers.adult_men, 2);
            assert_eq!(config.request.passengers.adult_women, 1);
            assert_eq!(config.request.passengers.total(), 3);

            let filter = config.request.time_filter.unwrap();
            assert_eq!(filter.departure_min, Some("08:00".to_string()));
            assert_eq!(filter.departure_max, Some("12:00".to_string()));

            assert_eq!(
                config.discord_webhook_url,
                Some("https://discord.com/webhook".to_string())
            );
            assert!(!config.notify_on_change_only);
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_time_filter_min_only() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("498")),
            ("DEPARTURE_TIME_MIN", Some("06:00")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_ok());
            let config = result.unwrap();

            let filter = config.request.time_filter.unwrap();
            assert_eq!(filter.departure_min, Some("06:00".to_string()));
            assert!(filter.departure_max.is_none());
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_time_filter_max_only() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("498")),
            ("DEPARTURE_TIME_MAX", Some("10:00")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_ok());
            let config = result.unwrap();

            let filter = config.request.time_filter.unwrap();
            assert!(filter.departure_min.is_none());
            assert_eq!(filter.departure_max, Some("10:00".to_string()));
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_scrape_interval_returns_error() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("498")),
            ("SCRAPE_INTERVAL_SECS", Some("not_a_number")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_err());
        });
    }

    #[test]
    #[serial]
    fn test_config_from_env_too_many_passengers_returns_error() {
        let mut vars = all_config_vars_cleared();
        vars.extend([
            ("ROUTE_ID", Some("155")),
            ("DEPARTURE_STATION", Some("001")),
            ("ARRIVAL_STATION", Some("498")),
            ("ADULT_MEN", Some("10")),
            ("ADULT_WOMEN", Some("5")),
        ]);
        temp_env::with_vars(vars, || {
            let result = Config::from_env_with_dotenv(false);
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Maximum 12 passengers"));
        });
    }
}
