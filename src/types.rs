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
        let start = NaiveDate::parse_from_str(&self.start, "%Y%m%d")
            .map_err(|e| ScraperError::Config(format!("Invalid start date: {}", e)))?;
        let end = NaiveDate::parse_from_str(&self.end, "%Y%m%d")
            .map_err(|e| ScraperError::Config(format!("Invalid end date: {}", e)))?;

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
