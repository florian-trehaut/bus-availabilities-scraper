use crate::entities::{prelude::*, route_states, user_passengers, user_routes, users};
use crate::error::{Result, ScraperError};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRouteWithDetails {
    pub user_route_id: Uuid,
    pub email: String,
    pub notify_on_change_only: bool,
    pub scrape_interval_secs: i64,
    pub discord_webhook_url: Option<String>,
    pub area_id: i32,
    pub route_id: i32,
    pub departure_station: String,
    pub arrival_station: String,
    pub date_start: String,
    pub date_end: String,
    pub departure_time_min: Option<String>,
    pub departure_time_max: Option<String>,
    pub passengers: PassengerDetails,
}

#[derive(Debug, Clone)]
pub struct PassengerDetails {
    pub adult_men: i16,
    pub adult_women: i16,
    pub child_men: i16,
    pub child_women: i16,
    pub handicap_adult_men: i16,
    pub handicap_adult_women: i16,
    pub handicap_child_men: i16,
    pub handicap_child_women: i16,
}

impl PassengerDetails {
    pub const fn total(&self) -> i16 {
        self.adult_men
            + self.adult_women
            + self.child_men
            + self.child_women
            + self.handicap_adult_men
            + self.handicap_adult_women
            + self.handicap_child_men
            + self.handicap_child_women
    }
}

#[derive(Debug, Clone)]
pub struct RouteStateDetails {
    pub last_seen_hash: String,
}

pub async fn get_all_active_user_routes(
    db: &DatabaseConnection,
) -> Result<Vec<UserRouteWithDetails>> {
    // Query 1: Get all enabled users with their routes (uses JOIN internally)
    let users_with_routes: Vec<(users::Model, Vec<user_routes::Model>)> = Users::find()
        .filter(users::Column::Enabled.eq(true))
        .find_with_related(UserRoutes)
        .all(db)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to fetch users with routes: {}", e)))?;

    // Collect all route IDs for batch passenger query
    let route_ids: Vec<Uuid> = users_with_routes
        .iter()
        .flat_map(|(_, routes)| routes.iter().map(|r| r.id))
        .collect();

    if route_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Query 2: Get all passengers in one query
    let all_passengers: Vec<user_passengers::Model> = UserPassengers::find()
        .filter(user_passengers::Column::UserRouteId.is_in(route_ids))
        .all(db)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to fetch passengers: {}", e)))?;

    // Index passengers by route_id for O(1) lookup
    let passengers_map: HashMap<Uuid, user_passengers::Model> = all_passengers
        .into_iter()
        .map(|p| (p.user_route_id, p))
        .collect();

    // Build result
    let mut result = Vec::new();
    for (user, routes) in users_with_routes {
        for route in routes {
            let passengers = passengers_map.get(&route.id).ok_or_else(|| {
                ScraperError::Config(format!("No passengers found for route {}", route.id))
            })?;

            result.push(UserRouteWithDetails {
                user_route_id: route.id,
                email: user.email.clone(),
                notify_on_change_only: user.notify_on_change_only,
                scrape_interval_secs: user.scrape_interval_secs,
                discord_webhook_url: user.discord_webhook_url.clone(),
                area_id: route.area_id,
                route_id: route.route_id,
                departure_station: route.departure_station,
                arrival_station: route.arrival_station,
                date_start: route.date_start,
                date_end: route.date_end,
                departure_time_min: route.departure_time_min,
                departure_time_max: route.departure_time_max,
                passengers: PassengerDetails {
                    adult_men: passengers.adult_men,
                    adult_women: passengers.adult_women,
                    child_men: passengers.child_men,
                    child_women: passengers.child_women,
                    handicap_adult_men: passengers.handicap_adult_men,
                    handicap_adult_women: passengers.handicap_adult_women,
                    handicap_child_men: passengers.handicap_child_men,
                    handicap_child_women: passengers.handicap_child_women,
                },
            });
        }
    }

    Ok(result)
}

pub async fn get_route_state(
    db: &DatabaseConnection,
    user_route_id: Uuid,
) -> Result<Option<RouteStateDetails>> {
    let state = RouteStates::find_by_id(user_route_id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to fetch route state: {}", e)))?;

    Ok(state.map(|s| RouteStateDetails {
        last_seen_hash: s.last_seen_hash,
    }))
}

pub async fn update_route_state(
    db: &DatabaseConnection,
    user_route_id: Uuid,
    hash: String,
    increment_alerts: bool,
) -> Result<()> {
    let existing = RouteStates::find_by_id(user_route_id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to fetch route state: {}", e)))?;

    match existing {
        Some(state) => {
            let new_total_checks = state.total_checks + 1;
            let new_total_alerts = if increment_alerts {
                state.total_alerts + 1
            } else {
                state.total_alerts
            };

            let active_model = route_states::ActiveModel {
                user_route_id: Set(user_route_id),
                last_seen_hash: Set(hash),
                last_check: Set(Some(Utc::now())),
                total_checks: Set(new_total_checks),
                total_alerts: Set(new_total_alerts),
            };
            active_model
                .update(db)
                .await
                .map_err(|e| ScraperError::Config(format!("Failed to update route state: {e}")))?;
        }
        None => {
            let new_state = route_states::ActiveModel {
                user_route_id: Set(user_route_id),
                last_seen_hash: Set(hash),
                last_check: Set(Some(Utc::now())),
                total_checks: Set(1),
                total_alerts: Set(i64::from(increment_alerts)),
            };
            new_state
                .insert(db)
                .await
                .map_err(|e| ScraperError::Config(format!("Failed to insert route state: {e}")))?;
        }
    }

    Ok(())
}

pub async fn get_station_name(db: &DatabaseConnection, station_id: &str) -> Result<Option<String>> {
    let station = Stations::find_by_id(station_id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to fetch station: {}", e)))?;

    Ok(station.map(|s| s.name))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::db::init_database;
    use migration::{Migrator, MigratorTrait};

    async fn setup_test_db() -> DatabaseConnection {
        let db = init_database("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_get_all_active_user_routes_empty() {
        let db = setup_test_db().await;
        let routes = get_all_active_user_routes(&db).await.unwrap();
        assert!(routes.is_empty());
    }

    #[tokio::test]
    async fn test_get_station_name() {
        let db = setup_test_db().await;
        let name = get_station_name(&db, "001").await.unwrap();
        assert_eq!(name, Some("Busta Shinjuku".to_string()));
    }

    #[tokio::test]
    async fn test_route_state_lifecycle() {
        use crate::entities::{user_passengers, user_routes, users};
        use sea_orm::{ActiveModelTrait, Set};

        let db = setup_test_db().await;

        let user_id = Uuid::new_v4();
        let route_id = Uuid::new_v4();

        let user = users::ActiveModel {
            id: Set(user_id),
            email: Set("test@test.com".to_string()),
            enabled: Set(true),
            notify_on_change_only: Set(true),
            scrape_interval_secs: Set(300),
            discord_webhook_url: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        user.insert(&db).await.unwrap();

        let route = user_routes::ActiveModel {
            id: Set(route_id),
            user_id: Set(user_id),
            area_id: Set(1),
            route_id: Set(155),
            departure_station: Set("001".to_string()),
            arrival_station: Set("498".to_string()),
            date_start: Set("2025-10-12".to_string()),
            date_end: Set("2025-10-19".to_string()),
            departure_time_min: Set(None),
            departure_time_max: Set(None),
            created_at: Set(chrono::Utc::now()),
        };
        route.insert(&db).await.unwrap();

        let passengers = user_passengers::ActiveModel {
            user_route_id: Set(route_id),
            adult_men: Set(1),
            adult_women: Set(0),
            child_men: Set(0),
            child_women: Set(0),
            handicap_adult_men: Set(0),
            handicap_adult_women: Set(0),
            handicap_child_men: Set(0),
            handicap_child_women: Set(0),
        };
        passengers.insert(&db).await.unwrap();

        let state = get_route_state(&db, route_id).await.unwrap();
        assert!(state.is_none());

        update_route_state(&db, route_id, "hash1".to_string(), false)
            .await
            .unwrap();

        let state = get_route_state(&db, route_id).await.unwrap();
        assert!(state.is_some());
        assert_eq!(state.as_ref().unwrap().last_seen_hash, "hash1");

        update_route_state(&db, route_id, "hash2".to_string(), true)
            .await
            .unwrap();

        let state = get_route_state(&db, route_id).await.unwrap();
        assert_eq!(state.as_ref().unwrap().last_seen_hash, "hash2");
    }
}
