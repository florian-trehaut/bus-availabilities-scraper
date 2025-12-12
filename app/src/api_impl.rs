//! Business logic extracted from server functions for testability.
//!
//! This module contains the core implementation logic that was previously
//! embedded in `#[server]` macro functions. By extracting this logic,
//! we can achieve better test coverage since tarpaulin cannot measure
//! code inside procedural macros.

use crate::api::{
    RouteDto, StationDto, UserDto, UserFormDto, UserRouteDto, UserRouteFormDto,
    UserRouteWithPassengersDto,
};
use crate::entities::{prelude::*, user_passengers, user_routes, users};
use crate::error::{Result, ScraperError};
use crate::scraper::BusScraper;
use crate::translations::{translate_route_name, translate_station_name};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

// === UUID Parsing ===

/// Parse a UUID string, returning a descriptive error on failure.
pub fn parse_uuid(id: &str) -> Result<Uuid> {
    Uuid::parse_str(id).map_err(|e| ScraperError::Config(format!("Invalid UUID: {e}")))
}

// === DTO Conversions ===

/// Convert a user model to a DTO.
pub fn user_to_dto(user: users::Model) -> UserDto {
    UserDto {
        id: user.id.to_string(),
        email: user.email,
        enabled: user.enabled,
        notify_on_change_only: user.notify_on_change_only,
        scrape_interval_secs: user.scrape_interval_secs,
        discord_webhook_url: user.discord_webhook_url,
        created_at: user.created_at.to_string(),
    }
}

/// Convert a user route model to a DTO.
pub fn user_route_to_dto(route: user_routes::Model) -> UserRouteDto {
    UserRouteDto {
        id: route.id.to_string(),
        user_id: route.user_id.to_string(),
        area_id: route.area_id,
        route_id: route.route_id,
        departure_station: route.departure_station,
        arrival_station: route.arrival_station,
        date_start: route.date_start,
        date_end: route.date_end,
        departure_time_min: route.departure_time_min,
        departure_time_max: route.departure_time_max,
    }
}

/// Convert a user route with passengers to a DTO.
pub fn user_route_with_passengers_to_dto(
    route: user_routes::Model,
    passengers: Option<user_passengers::Model>,
) -> UserRouteWithPassengersDto {
    let p = passengers.unwrap_or(user_passengers::Model {
        user_route_id: route.id,
        adult_men: 0,
        adult_women: 0,
        child_men: 0,
        child_women: 0,
        handicap_adult_men: 0,
        handicap_adult_women: 0,
        handicap_child_men: 0,
        handicap_child_women: 0,
    });

    UserRouteWithPassengersDto {
        id: route.id.to_string(),
        user_id: route.user_id.to_string(),
        area_id: route.area_id,
        route_id: route.route_id,
        departure_station: route.departure_station,
        arrival_station: route.arrival_station,
        date_start: route.date_start,
        date_end: route.date_end,
        departure_time_min: route.departure_time_min,
        departure_time_max: route.departure_time_max,
        adult_men: p.adult_men,
        adult_women: p.adult_women,
        child_men: p.child_men,
        child_women: p.child_women,
        handicap_adult_men: p.handicap_adult_men,
        handicap_adult_women: p.handicap_adult_women,
        handicap_child_men: p.handicap_child_men,
        handicap_child_women: p.handicap_child_women,
    }
}

// === User Operations ===

/// Fetch all users from the database.
pub async fn get_users_impl(db: &DatabaseConnection) -> Result<Vec<UserDto>> {
    let users = Users::find()
        .all(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Database error: {e}")))?;

    Ok(users.into_iter().map(user_to_dto).collect())
}

/// Create a new user in the database.
pub async fn create_user_impl(db: &DatabaseConnection, form: UserFormDto) -> Result<UserDto> {
    let new_user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(form.email),
        enabled: Set(form.enabled),
        notify_on_change_only: Set(form.notify_on_change_only),
        scrape_interval_secs: Set(form.scrape_interval_secs),
        discord_webhook_url: Set(form.discord_webhook_url),
        created_at: Set(chrono::Utc::now()),
    };

    let user = new_user
        .insert(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to create user: {e}")))?;

    Ok(user_to_dto(user))
}

/// Update an existing user in the database.
pub async fn update_user_impl(
    db: &DatabaseConnection,
    id: Uuid,
    form: UserFormDto,
) -> Result<UserDto> {
    let user = Users::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Database error: {e}")))?
        .ok_or_else(|| ScraperError::NotFound("User not found".to_string()))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.email = Set(form.email);
    active_user.enabled = Set(form.enabled);
    active_user.notify_on_change_only = Set(form.notify_on_change_only);
    active_user.scrape_interval_secs = Set(form.scrape_interval_secs);
    active_user.discord_webhook_url = Set(form.discord_webhook_url);

    let updated_user = active_user
        .update(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to update user: {e}")))?;

    Ok(user_to_dto(updated_user))
}

/// Delete a user from the database.
pub async fn delete_user_impl(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    Users::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to delete user: {e}")))?;

    Ok(())
}

// === User Route Operations ===

/// Fetch all routes for a user from the database.
pub async fn get_user_routes_impl(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<UserRouteWithPassengersDto>> {
    let routes = UserRoutes::find()
        .filter(user_routes::Column::UserId.eq(user_id))
        .find_also_related(UserPassengers)
        .all(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Database error: {e}")))?;

    Ok(routes
        .into_iter()
        .map(|(route, passengers)| user_route_with_passengers_to_dto(route, passengers))
        .collect())
}

/// Create a new user route with passengers in the database.
pub async fn create_user_route_impl(
    db: &DatabaseConnection,
    form: UserRouteFormDto,
) -> Result<UserRouteDto> {
    let user_id =
        parse_uuid(&form.user_id).map_err(|_| ScraperError::Config("Invalid user UUID".into()))?;
    let route_id = Uuid::new_v4();

    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(form.area_id),
        route_id: Set(form.route_id),
        departure_station: Set(form.departure_station),
        arrival_station: Set(form.arrival_station),
        date_start: Set(form.date_start),
        date_end: Set(form.date_end),
        departure_time_min: Set(form.departure_time_min),
        departure_time_max: Set(form.departure_time_max),
        created_at: Set(chrono::Utc::now()),
    };

    let route = new_route
        .insert(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to create route: {e}")))?;

    let new_passengers = user_passengers::ActiveModel {
        user_route_id: Set(route_id),
        adult_men: Set(form.adult_men),
        adult_women: Set(form.adult_women),
        child_men: Set(form.child_men),
        child_women: Set(form.child_women),
        handicap_adult_men: Set(form.handicap_adult_men),
        handicap_adult_women: Set(form.handicap_adult_women),
        handicap_child_men: Set(form.handicap_child_men),
        handicap_child_women: Set(form.handicap_child_women),
    };

    new_passengers
        .insert(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to create passengers: {e}")))?;

    Ok(user_route_to_dto(route))
}

/// Update an existing user route with passengers in the database.
pub async fn update_user_route_impl(
    db: &DatabaseConnection,
    id: Uuid,
    form: UserRouteFormDto,
) -> Result<UserRouteDto> {
    let route = UserRoutes::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Database error: {e}")))?
        .ok_or_else(|| ScraperError::NotFound("Route not found".to_string()))?;

    let mut active_route: user_routes::ActiveModel = route.into();
    active_route.area_id = Set(form.area_id);
    active_route.route_id = Set(form.route_id);
    active_route.departure_station = Set(form.departure_station);
    active_route.arrival_station = Set(form.arrival_station);
    active_route.date_start = Set(form.date_start);
    active_route.date_end = Set(form.date_end);
    active_route.departure_time_min = Set(form.departure_time_min);
    active_route.departure_time_max = Set(form.departure_time_max);

    let updated_route = active_route
        .update(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to update route: {e}")))?;

    // Update passengers if they exist
    let passengers = UserPassengers::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Database error: {e}")))?;

    if let Some(p) = passengers {
        let mut active_passengers: user_passengers::ActiveModel = p.into();
        active_passengers.adult_men = Set(form.adult_men);
        active_passengers.adult_women = Set(form.adult_women);
        active_passengers.child_men = Set(form.child_men);
        active_passengers.child_women = Set(form.child_women);
        active_passengers.handicap_adult_men = Set(form.handicap_adult_men);
        active_passengers.handicap_adult_women = Set(form.handicap_adult_women);
        active_passengers.handicap_child_men = Set(form.handicap_child_men);
        active_passengers.handicap_child_women = Set(form.handicap_child_women);

        active_passengers
            .update(db)
            .await
            .map_err(|e| ScraperError::Database(format!("Failed to update passengers: {e}")))?;
    }

    Ok(user_route_to_dto(updated_route))
}

/// Delete a user route from the database.
pub async fn delete_user_route_impl(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    UserRoutes::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| ScraperError::Database(format!("Failed to delete route: {e}")))?;

    Ok(())
}

// === Scraper Operations ===

/// Fetch routes from the Highway Bus API and translate names.
pub async fn fetch_and_translate_routes(
    scraper: &BusScraper,
    area_id: i32,
) -> Result<Vec<RouteDto>> {
    let routes = scraper.fetch_routes(area_id as u32).await?;

    Ok(routes
        .into_iter()
        .map(|r| RouteDto {
            route_id: r.id,
            area_id,
            name: translate_route_name(&r.name),
        })
        .collect())
}

/// Fetch departure stations from the Highway Bus API and translate names.
pub async fn fetch_and_translate_departure_stations(
    scraper: &BusScraper,
    route_id: &str,
) -> Result<Vec<StationDto>> {
    let stations = scraper.fetch_departure_stations(route_id).await?;

    Ok(stations
        .into_iter()
        .map(|s| StationDto {
            station_id: s.id,
            name: translate_station_name(&s.name),
            area_id: 0,
        })
        .collect())
}

/// Fetch arrival stations from the Highway Bus API and translate names.
pub async fn fetch_and_translate_arrival_stations(
    scraper: &BusScraper,
    route_id: &str,
    departure_station_id: &str,
) -> Result<Vec<StationDto>> {
    let stations = scraper
        .fetch_arrival_stations(route_id, departure_station_id)
        .await?;

    Ok(stations
        .into_iter()
        .map(|s| StationDto {
            station_id: s.id,
            name: translate_station_name(&s.name),
            area_id: 0,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_route_with_passengers_to_dto_with_none_passengers() {
        let route = user_routes::Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            area_id: 1,
            route_id: "155".to_string(),
            departure_station: "001".to_string(),
            arrival_station: "064".to_string(),
            date_start: "20250101".to_string(),
            date_end: "20250107".to_string(),
            departure_time_min: None,
            departure_time_max: None,
            created_at: chrono::Utc::now(),
        };

        let dto = user_route_with_passengers_to_dto(route.clone(), None);

        assert_eq!(dto.route_id, "155");
        assert_eq!(dto.adult_men, 0);
        assert_eq!(dto.adult_women, 0);
    }

    #[test]
    fn test_user_route_with_passengers_to_dto_with_passengers() {
        let route_uuid = Uuid::new_v4();
        let route = user_routes::Model {
            id: route_uuid,
            user_id: Uuid::new_v4(),
            area_id: 1,
            route_id: "155".to_string(),
            departure_station: "001".to_string(),
            arrival_station: "064".to_string(),
            date_start: "20250101".to_string(),
            date_end: "20250107".to_string(),
            departure_time_min: Some("08:00".to_string()),
            departure_time_max: Some("12:00".to_string()),
            created_at: chrono::Utc::now(),
        };

        let passengers = user_passengers::Model {
            user_route_id: route_uuid,
            adult_men: 2,
            adult_women: 1,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        };

        let dto = user_route_with_passengers_to_dto(route, Some(passengers));

        assert_eq!(dto.adult_men, 2);
        assert_eq!(dto.adult_women, 1);
        assert_eq!(dto.departure_time_min, Some("08:00".to_string()));
    }
}
