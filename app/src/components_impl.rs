//! Business logic extracted from Leptos components for testability.
//!
//! This module contains pure functions that were previously embedded
//! in `#[component]` macro functions. By extracting this logic,
//! we can achieve better test coverage since tarpaulin cannot measure
//! code inside procedural macros.

use crate::api::{UserDto, UserFormDto, UserRouteFormDto, UserRouteWithPassengersDto};

// === Passenger Calculations ===

/// Calculate the total number of passengers from individual counts.
#[allow(clippy::too_many_arguments)]
pub fn calculate_total_passengers(
    adult_men: i16,
    adult_women: i16,
    child_men: i16,
    child_women: i16,
    handicap_adult_men: i16,
    handicap_adult_women: i16,
    handicap_child_men: i16,
    handicap_child_women: i16,
) -> i16 {
    adult_men
        + adult_women
        + child_men
        + child_women
        + handicap_adult_men
        + handicap_adult_women
        + handicap_child_men
        + handicap_child_women
}

// === String Helpers ===

/// Convert an empty string to None, otherwise Some(string).
/// Used for optional form fields like webhook URLs and time filters.
pub fn optional_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

/// Parse an interval string to u64, with a default fallback.
/// Used for `scrape_interval_secs` parsing.
pub fn parse_interval(s: &str, default: i64) -> i64 {
    s.parse().unwrap_or(default)
}

// === UI Status Helpers ===

/// Get the CSS badge class for user enabled status.
pub fn user_status_badge_class(enabled: bool) -> &'static str {
    if enabled {
        "badge-success"
    } else {
        "badge-danger"
    }
}

/// Get the display text for user enabled status.
pub fn user_status_text(enabled: bool) -> &'static str {
    if enabled { "Active" } else { "Inactive" }
}

/// Get the display text for notify mode.
pub fn notify_mode_text(notify_on_change_only: bool) -> &'static str {
    if notify_on_change_only {
        "Changes Only"
    } else {
        "All"
    }
}

/// Get the CSS badge class for notify mode.
pub fn notify_mode_badge_class(notify_on_change_only: bool) -> &'static str {
    if notify_on_change_only {
        "badge-info"
    } else {
        "badge-neutral"
    }
}

// === Edit Mode Detection ===

/// Check if we're in edit mode (item exists).
pub fn is_edit_mode<T>(item: &Option<T>) -> bool {
    item.is_some()
}

// === Form Data Builders ===

/// Build a [`UserFormDto`] from form field values.
pub fn build_user_form_dto(
    email: String,
    enabled: bool,
    notify_on_change_only: bool,
    interval_str: &str,
    webhook: String,
) -> UserFormDto {
    UserFormDto {
        email,
        enabled,
        notify_on_change_only,
        scrape_interval_secs: parse_interval(interval_str, 300),
        discord_webhook_url: optional_string(webhook),
    }
}

/// Passenger count data structure for form building.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PassengerCountData {
    pub adult_men: i16,
    pub adult_women: i16,
    pub child_men: i16,
    pub child_women: i16,
    pub handicap_adult_men: i16,
    pub handicap_adult_women: i16,
    pub handicap_child_men: i16,
    pub handicap_child_women: i16,
}

impl PassengerCountData {
    /// Calculate total passengers.
    pub fn total(&self) -> i16 {
        calculate_total_passengers(
            self.adult_men,
            self.adult_women,
            self.child_men,
            self.child_women,
            self.handicap_adult_men,
            self.handicap_adult_women,
            self.handicap_child_men,
            self.handicap_child_women,
        )
    }
}

/// Build a [`UserRouteFormDto`] from form field values.
#[allow(clippy::too_many_arguments)]
pub fn build_user_route_form_dto(
    user_id: String,
    area_id: i32,
    route_id: String,
    departure_station: String,
    arrival_station: String,
    date_start: String,
    date_end: String,
    time_min: String,
    time_max: String,
    passengers: PassengerCountData,
) -> UserRouteFormDto {
    UserRouteFormDto {
        user_id,
        area_id,
        route_id,
        departure_station,
        arrival_station,
        date_start,
        date_end,
        departure_time_min: optional_string(time_min),
        departure_time_max: optional_string(time_max),
        adult_men: passengers.adult_men,
        adult_women: passengers.adult_women,
        child_men: passengers.child_men,
        child_women: passengers.child_women,
        handicap_adult_men: passengers.handicap_adult_men,
        handicap_adult_women: passengers.handicap_adult_women,
        handicap_child_men: passengers.handicap_child_men,
        handicap_child_women: passengers.handicap_child_women,
    }
}

// === Date Formatting ===

/// Format a date string for display (YYYYMMDD → YYYY-MM-DD).
pub fn format_date_for_display(date: &str) -> String {
    if date.len() == 8 {
        format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
    } else {
        date.to_string()
    }
}

/// Parse a display date back to storage format (YYYY-MM-DD → YYYYMMDD).
pub fn parse_date_from_display(date: &str) -> String {
    date.replace('-', "")
}

// === Form State Extraction ===

/// Initial state for a User form (create or edit).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserFormState {
    pub email: String,
    pub enabled: bool,
    pub notify_on_change_only: bool,
    pub interval: String,
    pub webhook: String,
}

/// Extract the initial form state from an optional [`UserDto`].
/// Returns defaults for new user creation, or populated values for editing.
pub fn extract_user_form_state(user: Option<&UserDto>) -> UserFormState {
    match user {
        Some(u) => UserFormState {
            email: u.email.clone(),
            enabled: u.enabled,
            notify_on_change_only: u.notify_on_change_only,
            interval: u.scrape_interval_secs.to_string(),
            webhook: u.discord_webhook_url.clone().unwrap_or_default(),
        },
        None => UserFormState {
            email: String::new(),
            enabled: true,
            notify_on_change_only: true,
            interval: "300".to_string(),
            webhook: String::new(),
        },
    }
}

/// Initial state for a [`UserRoute`] form (create or edit).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserRouteFormState {
    pub area_id: i32,
    pub route_id: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub date_start: String,
    pub date_end: String,
    pub time_min: String,
    pub time_max: String,
    pub passengers: PassengerCountData,
}

/// Extract the initial form state from an optional [`UserRouteWithPassengersDto`].
/// Returns defaults for new route creation, or populated values for editing.
pub fn extract_user_route_form_state(
    route: Option<&UserRouteWithPassengersDto>,
) -> UserRouteFormState {
    match route {
        Some(r) => UserRouteFormState {
            area_id: r.area_id,
            route_id: r.route_id.clone(),
            departure_station: r.departure_station.clone(),
            arrival_station: r.arrival_station.clone(),
            date_start: r.date_start.clone(),
            date_end: r.date_end.clone(),
            time_min: r.departure_time_min.clone().unwrap_or_default(),
            time_max: r.departure_time_max.clone().unwrap_or_default(),
            passengers: PassengerCountData {
                adult_men: r.adult_men,
                adult_women: r.adult_women,
                child_men: r.child_men,
                child_women: r.child_women,
                handicap_adult_men: r.handicap_adult_men,
                handicap_adult_women: r.handicap_adult_women,
                handicap_child_men: r.handicap_child_men,
                handicap_child_women: r.handicap_child_women,
            },
        },
        None => UserRouteFormState {
            area_id: 1,
            route_id: String::new(),
            departure_station: String::new(),
            arrival_station: String::new(),
            date_start: String::new(),
            date_end: String::new(),
            time_min: String::new(),
            time_max: String::new(),
            passengers: PassengerCountData::default(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Passenger Calculation Tests ===

    #[test]
    fn test_calculate_total_passengers_all_zeros() {
        let total = calculate_total_passengers(0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_calculate_total_passengers_adults_only() {
        let total = calculate_total_passengers(2, 1, 0, 0, 0, 0, 0, 0);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_calculate_total_passengers_mixed() {
        let total = calculate_total_passengers(2, 2, 1, 1, 0, 0, 0, 0);
        assert_eq!(total, 6);
    }

    #[test]
    fn test_calculate_total_passengers_all_types() {
        let total = calculate_total_passengers(1, 1, 1, 1, 1, 1, 1, 1);
        assert_eq!(total, 8);
    }

    // === Optional String Tests ===

    #[test]
    fn test_optional_string_empty() {
        let result = optional_string(String::new());
        assert!(result.is_none());
    }

    #[test]
    fn test_optional_string_with_value() {
        let result = optional_string("https://webhook.url".to_string());
        assert_eq!(result, Some("https://webhook.url".to_string()));
    }

    #[test]
    fn test_optional_string_whitespace_not_empty() {
        // Whitespace is NOT treated as empty
        let result = optional_string("   ".to_string());
        assert_eq!(result, Some("   ".to_string()));
    }

    // === Parse Interval Tests ===

    #[test]
    fn test_parse_interval_valid() {
        let result = parse_interval("600", 300);
        assert_eq!(result, 600);
    }

    #[test]
    fn test_parse_interval_invalid_returns_default() {
        let result = parse_interval("not_a_number", 300);
        assert_eq!(result, 300);
    }

    #[test]
    fn test_parse_interval_empty_returns_default() {
        let result = parse_interval("", 300);
        assert_eq!(result, 300);
    }

    #[test]
    fn test_parse_interval_negative() {
        let result = parse_interval("-100", 300);
        assert_eq!(result, -100);
    }

    // === UI Status Tests ===

    #[test]
    fn test_user_status_badge_class_enabled() {
        assert_eq!(user_status_badge_class(true), "badge-success");
    }

    #[test]
    fn test_user_status_badge_class_disabled() {
        assert_eq!(user_status_badge_class(false), "badge-danger");
    }

    #[test]
    fn test_user_status_text_enabled() {
        assert_eq!(user_status_text(true), "Active");
    }

    #[test]
    fn test_user_status_text_disabled() {
        assert_eq!(user_status_text(false), "Inactive");
    }

    #[test]
    fn test_notify_mode_text_changes_only() {
        assert_eq!(notify_mode_text(true), "Changes Only");
    }

    #[test]
    fn test_notify_mode_text_all() {
        assert_eq!(notify_mode_text(false), "All");
    }

    #[test]
    fn test_notify_mode_badge_class_changes_only() {
        assert_eq!(notify_mode_badge_class(true), "badge-info");
    }

    #[test]
    fn test_notify_mode_badge_class_all() {
        assert_eq!(notify_mode_badge_class(false), "badge-neutral");
    }

    // === Edit Mode Tests ===

    #[test]
    fn test_is_edit_mode_none() {
        let item: Option<String> = None;
        assert!(!is_edit_mode(&item));
    }

    #[test]
    fn test_is_edit_mode_some() {
        let item = Some("value".to_string());
        assert!(is_edit_mode(&item));
    }

    // === Form Builder Tests ===

    #[test]
    fn test_build_user_form_dto() {
        let dto = build_user_form_dto(
            "test@example.com".to_string(),
            true,
            false,
            "600",
            "https://webhook.url".to_string(),
        );

        assert_eq!(dto.email, "test@example.com");
        assert!(dto.enabled);
        assert!(!dto.notify_on_change_only);
        assert_eq!(dto.scrape_interval_secs, 600);
        assert_eq!(
            dto.discord_webhook_url,
            Some("https://webhook.url".to_string())
        );
    }

    #[test]
    fn test_build_user_form_dto_empty_webhook() {
        let dto = build_user_form_dto(
            "test@example.com".to_string(),
            true,
            true,
            "300",
            String::new(),
        );

        assert!(dto.discord_webhook_url.is_none());
    }

    #[test]
    fn test_build_user_route_form_dto() {
        let passengers = PassengerCountData {
            adult_men: 2,
            adult_women: 1,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };

        let dto = build_user_route_form_dto(
            "user-uuid".to_string(),
            100,
            "155".to_string(),
            "001".to_string(),
            "064".to_string(),
            "20250101".to_string(),
            "20250107".to_string(),
            "08:00".to_string(),
            "18:00".to_string(),
            passengers,
        );

        assert_eq!(dto.user_id, "user-uuid");
        assert_eq!(dto.area_id, 100);
        assert_eq!(dto.route_id, "155");
        assert_eq!(dto.departure_time_min, Some("08:00".to_string()));
        assert_eq!(dto.departure_time_max, Some("18:00".to_string()));
        assert_eq!(dto.adult_men, 2);
        assert_eq!(dto.adult_women, 1);
    }

    #[test]
    fn test_build_user_route_form_dto_empty_times() {
        let passengers = PassengerCountData::default();

        let dto = build_user_route_form_dto(
            "user-uuid".to_string(),
            100,
            "155".to_string(),
            "001".to_string(),
            "064".to_string(),
            "20250101".to_string(),
            "20250107".to_string(),
            String::new(),
            String::new(),
            passengers,
        );

        assert!(dto.departure_time_min.is_none());
        assert!(dto.departure_time_max.is_none());
    }

    // === Passenger Count Data Tests ===

    #[test]
    fn test_passenger_count_data_total() {
        let passengers = PassengerCountData {
            adult_men: 2,
            adult_women: 1,
            child_men: 1,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };

        assert_eq!(passengers.total(), 4);
    }

    #[test]
    fn test_passenger_count_data_default() {
        let passengers = PassengerCountData::default();
        assert_eq!(passengers.total(), 0);
    }

    // === Date Formatting Tests ===

    #[test]
    fn test_format_date_for_display() {
        assert_eq!(format_date_for_display("20250115"), "2025-01-15");
    }

    #[test]
    fn test_format_date_for_display_invalid_length() {
        assert_eq!(format_date_for_display("2025-01-15"), "2025-01-15");
    }

    #[test]
    fn test_parse_date_from_display() {
        assert_eq!(parse_date_from_display("2025-01-15"), "20250115");
    }

    #[test]
    fn test_parse_date_from_display_no_dashes() {
        assert_eq!(parse_date_from_display("20250115"), "20250115");
    }

    // === Form State Extraction Tests ===

    #[test]
    fn test_extract_user_form_state_none() {
        let state = extract_user_form_state(None);
        assert_eq!(state.email, "");
        assert!(state.enabled);
        assert!(state.notify_on_change_only);
        assert_eq!(state.interval, "300");
        assert_eq!(state.webhook, "");
    }

    #[test]
    fn test_extract_user_form_state_some() {
        let user = UserDto {
            id: "uuid".to_string(),
            email: "test@example.com".to_string(),
            enabled: false,
            notify_on_change_only: false,
            scrape_interval_secs: 600,
            discord_webhook_url: Some("https://webhook.url".to_string()),
            created_at: "2025-01-01".to_string(),
        };

        let state = extract_user_form_state(Some(&user));
        assert_eq!(state.email, "test@example.com");
        assert!(!state.enabled);
        assert!(!state.notify_on_change_only);
        assert_eq!(state.interval, "600");
        assert_eq!(state.webhook, "https://webhook.url");
    }

    #[test]
    fn test_extract_user_form_state_some_no_webhook() {
        let user = UserDto {
            id: "uuid".to_string(),
            email: "test@example.com".to_string(),
            enabled: true,
            notify_on_change_only: true,
            scrape_interval_secs: 300,
            discord_webhook_url: None,
            created_at: "2025-01-01".to_string(),
        };

        let state = extract_user_form_state(Some(&user));
        assert_eq!(state.webhook, "");
    }

    #[test]
    fn test_extract_user_route_form_state_none() {
        let state = extract_user_route_form_state(None);
        assert_eq!(state.area_id, 1);
        assert_eq!(state.route_id, "");
        assert_eq!(state.departure_station, "");
        assert_eq!(state.arrival_station, "");
        assert_eq!(state.date_start, "");
        assert_eq!(state.date_end, "");
        assert_eq!(state.time_min, "");
        assert_eq!(state.time_max, "");
        assert_eq!(state.passengers.total(), 0);
    }

    #[test]
    fn test_extract_user_route_form_state_some() {
        let route = UserRouteWithPassengersDto {
            id: "route-uuid".to_string(),
            user_id: "user-uuid".to_string(),
            area_id: 100,
            route_id: "155".to_string(),
            departure_station: "001".to_string(),
            arrival_station: "064".to_string(),
            date_start: "20250101".to_string(),
            date_end: "20250107".to_string(),
            departure_time_min: Some("08:00".to_string()),
            departure_time_max: Some("18:00".to_string()),
            adult_men: 2,
            adult_women: 1,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };

        let state = extract_user_route_form_state(Some(&route));
        assert_eq!(state.area_id, 100);
        assert_eq!(state.route_id, "155");
        assert_eq!(state.departure_station, "001");
        assert_eq!(state.arrival_station, "064");
        assert_eq!(state.date_start, "20250101");
        assert_eq!(state.date_end, "20250107");
        assert_eq!(state.time_min, "08:00");
        assert_eq!(state.time_max, "18:00");
        assert_eq!(state.passengers.adult_men, 2);
        assert_eq!(state.passengers.adult_women, 1);
        assert_eq!(state.passengers.total(), 3);
    }

    #[test]
    fn test_extract_user_route_form_state_some_no_times() {
        let route = UserRouteWithPassengersDto {
            id: "route-uuid".to_string(),
            user_id: "user-uuid".to_string(),
            area_id: 100,
            route_id: "155".to_string(),
            departure_station: "001".to_string(),
            arrival_station: "064".to_string(),
            date_start: "20250101".to_string(),
            date_end: "20250107".to_string(),
            departure_time_min: None,
            departure_time_max: None,
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };

        let state = extract_user_route_form_state(Some(&route));
        assert_eq!(state.time_min, "");
        assert_eq!(state.time_max, "");
    }

    #[test]
    fn test_user_form_state_default() {
        let state = UserFormState::default();
        assert_eq!(state.email, "");
        assert!(!state.enabled);
        assert!(!state.notify_on_change_only);
        assert_eq!(state.interval, "");
        assert_eq!(state.webhook, "");
    }

    #[test]
    fn test_user_route_form_state_default() {
        let state = UserRouteFormState::default();
        assert_eq!(state.area_id, 0);
        assert_eq!(state.route_id, "");
        assert_eq!(state.passengers.total(), 0);
    }
}
