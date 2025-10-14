use chrono::NaiveDate;
use serde::Serialize;

use crate::error::{Result, ScraperError};

#[derive(Debug, Clone)]
pub struct ScrapeRequest {
    pub area_id: u32,
    pub route_id: u32,
    pub departure_station: String,
    pub arrival_station: String,
    pub date_range: DateRange,
    pub passengers: PassengerCount,
    pub time_filter: Option<TimeFilter>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassengerCount {
    pub adult_men: u8,
    pub adult_women: u8,
    pub child_men: u8,
    pub child_women: u8,
    pub handicap_adult_men: u8,
    pub handicap_adult_women: u8,
    pub handicap_child_men: u8,
    pub handicap_child_women: u8,
}

impl PassengerCount {
    pub const fn total_male(&self) -> u8 {
        self.adult_men + self.child_men + self.handicap_adult_men + self.handicap_child_men
    }

    pub const fn total_female(&self) -> u8 {
        self.adult_women + self.child_women + self.handicap_adult_women + self.handicap_child_women
    }

    pub const fn total(&self) -> u8 {
        self.total_male() + self.total_female()
    }

    pub fn validate(&self) -> Result<()> {
        if self.total() == 0 {
            return Err(ScraperError::Config(
                "At least 1 passenger required".to_string(),
            ));
        }
        if self.total() > 12 {
            return Err(ScraperError::Config(
                "Maximum 12 passengers allowed".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for PassengerCount {
    fn default() -> Self {
        Self {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

impl DateRange {
    pub fn dates(&self) -> Result<Vec<String>> {
        let start = Self::parse_date(&self.start)?;
        let end = Self::parse_date(&self.end)?;

        if start > end {
            return Err(ScraperError::Config(
                "Start date must be before end date".to_string(),
            ));
        }

        let mut dates = Vec::new();
        let mut current = start;
        while current <= end {
            dates.push(current.format("%Y%m%d").to_string());
            current = current
                .succ_opt()
                .ok_or_else(|| ScraperError::Config("Date overflow".to_string()))?;
        }

        Ok(dates)
    }

    fn parse_date(date_str: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%Y%m%d"))
            .map_err(|e| {
                ScraperError::Config(format!(
                    "Invalid date '{}' (expected YYYY-MM-DD or YYYYMMDD): {}",
                    date_str, e
                ))
            })
    }
}

#[derive(Debug, Clone)]
pub struct TimeFilter {
    pub departure_min: Option<String>,
    pub departure_max: Option<String>,
}

impl TimeFilter {
    pub fn matches(&self, time: &str) -> bool {
        if let Some(ref min) = self.departure_min {
            if time < min.as_str() {
                return false;
            }
        }
        if let Some(ref max) = self.departure_max {
            if time > max.as_str() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Route {
    pub id: String,
    pub name: String,
    pub switch_changeable_flg: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Station {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AvailableDate {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResult {
    pub timestamp: String,
    pub route_id: String,
    pub route_name: String,
    pub departure_id: String,
    pub departure_name: String,
    pub arrival_id: String,
    pub arrival_name: String,
    pub date: String,
    pub available_dates: Vec<DateSlot>,
}

#[derive(Debug, Serialize)]
pub struct DateSlot {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusSchedule {
    pub bus_number: String,
    pub route_name: String,
    pub departure_station: String,
    pub departure_date: String,
    pub departure_time: String,
    pub arrival_station: String,
    pub arrival_date: String,
    pub arrival_time: String,
    pub way_no: u32,
    pub available_plans: Vec<PricingPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PricingPlan {
    pub plan_id: u32,
    pub plan_index: u32,
    pub plan_name: String,
    pub price: u32,
    pub display_price: String,
    pub availability: SeatAvailability,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum SeatAvailability {
    Available { remaining_seats: Option<u32> },
    SoldOut,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_iso_format() {
        let range = DateRange {
            start: "2025-10-29".to_string(),
            end: "2025-11-02".to_string(),
        };

        let dates = range.dates().unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], "20251029");
        assert_eq!(dates[4], "20251102");
    }

    #[test]
    fn test_date_range_yyyymmdd_format() {
        let range = DateRange {
            start: "20251029".to_string(),
            end: "20251102".to_string(),
        };

        let dates = range.dates().unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], "20251029");
        assert_eq!(dates[4], "20251102");
    }

    #[test]
    fn test_date_range_mixed_formats() {
        let range = DateRange {
            start: "2025-10-29".to_string(),
            end: "20251102".to_string(),
        };

        let dates = range.dates().unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], "20251029");
        assert_eq!(dates[4], "20251102");
    }

    #[test]
    fn test_date_range_single_day() {
        let range = DateRange {
            start: "2025-10-29".to_string(),
            end: "2025-10-29".to_string(),
        };

        let dates = range.dates().unwrap();
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], "20251029");
    }

    #[test]
    fn test_date_range_invalid_format() {
        let range = DateRange {
            start: "2025/10/29".to_string(),
            end: "2025-10-30".to_string(),
        };

        assert!(range.dates().is_err());
    }

    #[test]
    fn test_date_range_start_after_end() {
        let range = DateRange {
            start: "2025-11-02".to_string(),
            end: "2025-10-29".to_string(),
        };

        let result = range.dates();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Start date must be before end date"));
    }

    #[test]
    fn test_passenger_count_validation() {
        let valid = PassengerCount {
            adult_men: 1,
            ..Default::default()
        };
        assert!(valid.validate().is_ok());

        let too_many = PassengerCount {
            adult_men: 10,
            adult_women: 3,
            ..Default::default()
        };
        assert!(too_many.validate().is_err());

        let zero = PassengerCount {
            adult_men: 0,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };
        assert!(zero.validate().is_err());
    }

    #[test]
    fn test_time_filter_matching() {
        let filter = TimeFilter {
            departure_min: Some("08:00".to_string()),
            departure_max: Some("10:00".to_string()),
        };

        assert!(filter.matches("09:30"));
        assert!(filter.matches("08:00"));
        assert!(filter.matches("10:00"));
        assert!(!filter.matches("07:59"));
        assert!(!filter.matches("10:01"));
    }

    #[test]
    fn test_time_filter_no_min() {
        let filter = TimeFilter {
            departure_min: None,
            departure_max: Some("10:00".to_string()),
        };

        assert!(filter.matches("00:00"));
        assert!(filter.matches("09:59"));
        assert!(!filter.matches("10:01"));
    }

    #[test]
    fn test_time_filter_no_max() {
        let filter = TimeFilter {
            departure_min: Some("08:00".to_string()),
            departure_max: None,
        };

        assert!(filter.matches("08:00"));
        assert!(filter.matches("23:59"));
        assert!(!filter.matches("07:59"));
    }
}
