//! Business logic extracted from tracker for testability.
//!
//! This module contains pure functions that were previously embedded
//! in the `UserTracker` struct. By extracting this logic, we can achieve
//! better test coverage since these functions don't depend on async runtime
//! or external services.

use app::types::{BusSchedule, SeatAvailability};

/// Determine if a notification should be sent based on:
/// - `notify_on_change_only`: user preference for notification strategy
/// - `state_changed`: whether the availability hash has changed since last check
/// - `has_available_seats`: whether any schedules have seats available
///
/// Decision matrix:
/// | notify_on_change_only | state_changed | has_seats | Result |
/// |-----------------------|---------------|-----------|--------|
/// | true                  | true          | true      | true   |
/// | true                  | true          | false     | false  |
/// | true                  | false         | true      | false  |
/// | true                  | false         | false     | false  |
/// | false                 | *             | true      | true   |
/// | false                 | *             | false     | false  |
pub fn should_send_notification(
    notify_on_change_only: bool,
    state_changed: bool,
    has_available_seats: bool,
) -> bool {
    if notify_on_change_only {
        state_changed && has_available_seats
    } else {
        has_available_seats
    }
}

/// Filter schedules to only include those with at least one available seat.
/// A schedule has available seats if any of its plans have `SeatAvailability::Available` with `remaining_seats.is_some()`.
#[allow(dead_code)]
pub fn filter_schedules_with_seats(schedules: Vec<BusSchedule>) -> Vec<BusSchedule> {
    schedules
        .into_iter()
        .filter(|s| {
            s.available_plans.iter().any(|p| {
                matches!(&p.availability, SeatAvailability::Available { remaining_seats } if remaining_seats.is_some())
            })
        })
        .collect()
}

/// Check if the state has changed by comparing the current hash with the stored hash.
/// Returns `true` if:
/// - No previous state exists (first check)
/// - The stored hash differs from the current hash
pub fn has_state_changed(last_hash: Option<&str>, current_hash: &str) -> bool {
    match last_hash {
        Some(hash) => hash != current_hash,
        None => true, // First time = always considered "changed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::types::PricingPlan;

    // === should_send_notification tests ===

    #[test]
    fn test_notify_on_change_only_true_state_changed_seats_available() {
        // User wants change-only notifications, state changed, seats available -> NOTIFY
        assert!(should_send_notification(true, true, true));
    }

    #[test]
    fn test_notify_on_change_only_true_state_unchanged_seats_available() {
        // User wants change-only, but state hasn't changed -> NO NOTIFY
        assert!(!should_send_notification(true, false, true));
    }

    #[test]
    fn test_notify_on_change_only_true_state_changed_no_seats() {
        // User wants change-only, state changed but no seats -> NO NOTIFY
        assert!(!should_send_notification(true, true, false));
    }

    #[test]
    fn test_notify_on_change_only_true_state_unchanged_no_seats() {
        // User wants change-only, no change, no seats -> NO NOTIFY
        assert!(!should_send_notification(true, false, false));
    }

    #[test]
    fn test_notify_on_change_only_false_state_changed_seats_available() {
        // User wants all notifications, seats available -> NOTIFY
        assert!(should_send_notification(false, true, true));
    }

    #[test]
    fn test_notify_on_change_only_false_state_unchanged_seats_available() {
        // User wants all notifications, state unchanged but seats available -> NOTIFY
        assert!(should_send_notification(false, false, true));
    }

    #[test]
    fn test_notify_on_change_only_false_state_changed_no_seats() {
        // User wants all notifications, but no seats -> NO NOTIFY
        assert!(!should_send_notification(false, true, false));
    }

    #[test]
    fn test_notify_on_change_only_false_state_unchanged_no_seats() {
        // User wants all notifications, but no seats -> NO NOTIFY
        assert!(!should_send_notification(false, false, false));
    }

    // === has_state_changed tests ===

    #[test]
    fn test_has_state_changed_none_always_true() {
        // First time checking (no previous state) -> always changed
        assert!(has_state_changed(None, "12345"));
    }

    #[test]
    fn test_has_state_changed_same_hash() {
        // Same hash -> not changed
        assert!(!has_state_changed(Some("12345"), "12345"));
    }

    #[test]
    fn test_has_state_changed_different_hash() {
        // Different hash -> changed
        assert!(has_state_changed(Some("12345"), "99999"));
    }

    #[test]
    fn test_has_state_changed_empty_strings() {
        // Empty strings are still valid comparisons
        assert!(!has_state_changed(Some(""), ""));
        assert!(has_state_changed(Some(""), "12345"));
        assert!(has_state_changed(Some("12345"), ""));
    }

    // === filter_schedules_with_seats tests ===

    fn create_schedule_with_seats(remaining: Option<u32>) -> BusSchedule {
        BusSchedule {
            bus_number: "Bus_1".to_string(),
            route_name: "Test Route".to_string(),
            departure_station: "001".to_string(),
            departure_date: "20250115".to_string(),
            departure_time: "08:30".to_string(),
            arrival_station: "064".to_string(),
            arrival_date: "20250115".to_string(),
            arrival_time: "10:00".to_string(),
            way_no: 1,
            available_plans: vec![PricingPlan {
                plan_id: 12345,
                plan_index: 0,
                plan_name: "Standard".to_string(),
                price: 2100,
                display_price: "2100円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: remaining,
                },
            }],
        }
    }

    #[test]
    fn test_filter_schedules_empty_list() {
        let schedules: Vec<BusSchedule> = vec![];
        let result = filter_schedules_with_seats(schedules);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_schedules_all_with_seats() {
        let schedules = vec![
            create_schedule_with_seats(Some(5)),
            create_schedule_with_seats(Some(3)),
        ];
        let result = filter_schedules_with_seats(schedules);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_schedules_none_with_seats() {
        let schedules = vec![
            create_schedule_with_seats(None),
            create_schedule_with_seats(None),
        ];
        let result = filter_schedules_with_seats(schedules);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_schedules_mixed() {
        let schedules = vec![
            create_schedule_with_seats(Some(5)),
            create_schedule_with_seats(None),
            create_schedule_with_seats(Some(3)),
        ];
        let result = filter_schedules_with_seats(schedules);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_schedules_zero_seats_still_available() {
        // remaining_seats = Some(0) still means "available" (just none left)
        let schedules = vec![create_schedule_with_seats(Some(0))];
        let result = filter_schedules_with_seats(schedules);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_schedules_no_plans() {
        let mut schedule = create_schedule_with_seats(Some(5));
        schedule.available_plans.clear();
        let schedules = vec![schedule];
        let result = filter_schedules_with_seats(schedules);
        // No plans means no seats available
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_schedules_multiple_plans_one_with_seats() {
        let mut schedule = create_schedule_with_seats(None);
        schedule.available_plans.push(PricingPlan {
            plan_id: 99999,
            plan_index: 1,
            plan_name: "Premium".to_string(),
            price: 3500,
            display_price: "3500円".to_string(),
            availability: SeatAvailability::Available {
                remaining_seats: Some(2),
            },
        });
        let schedules = vec![schedule];
        let result = filter_schedules_with_seats(schedules);
        // At least one plan has seats -> schedule is included
        assert_eq!(result.len(), 1);
    }
}
