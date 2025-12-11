use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::{
    db,
    entities::{prelude::*, routes, stations, user_passengers, user_routes, users},
};

#[cfg(feature = "ssr")]
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub enabled: bool,
    pub notify_on_change_only: bool,
    pub scrape_interval_secs: i64,
    pub discord_webhook_url: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserFormDto {
    pub email: String,
    pub enabled: bool,
    pub notify_on_change_only: bool,
    pub scrape_interval_secs: i64,
    pub discord_webhook_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRouteDto {
    pub id: String,
    pub user_id: String,
    pub area_id: i32,
    pub route_id: i32,
    pub departure_station: String,
    pub arrival_station: String,
    pub date_start: String,
    pub date_end: String,
    pub departure_time_min: Option<String>,
    pub departure_time_max: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRouteFormDto {
    pub user_id: String,
    pub area_id: i32,
    pub route_id: i32,
    pub departure_station: String,
    pub arrival_station: String,
    pub date_start: String,
    pub date_end: String,
    pub departure_time_min: Option<String>,
    pub departure_time_max: Option<String>,
    pub adult_men: i16,
    pub adult_women: i16,
    pub child_men: i16,
    pub child_women: i16,
    pub handicap_adult_men: i16,
    pub handicap_adult_women: i16,
    pub handicap_child_men: i16,
    pub handicap_child_women: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteDto {
    pub route_id: String,
    pub area_id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationDto {
    pub station_id: String,
    pub name: String,
    pub area_id: i32,
}

#[server]
pub async fn get_users() -> Result<Vec<UserDto>, ServerFnError> {
    let db = db::get_db_from_context()?;

    let users = Users::find()
        .all(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(users
        .into_iter()
        .map(|u| UserDto {
            id: u.id.to_string(),
            email: u.email,
            enabled: u.enabled,
            notify_on_change_only: u.notify_on_change_only,
            scrape_interval_secs: u.scrape_interval_secs,
            discord_webhook_url: u.discord_webhook_url,
            created_at: u.created_at.to_string(),
        })
        .collect())
}

#[server]
pub async fn create_user(form: UserFormDto) -> Result<UserDto, ServerFnError> {
    let db = db::get_db_from_context()?;

    let new_user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(form.email.clone()),
        enabled: Set(form.enabled),
        notify_on_change_only: Set(form.notify_on_change_only),
        scrape_interval_secs: Set(form.scrape_interval_secs),
        discord_webhook_url: Set(form.discord_webhook_url.clone()),
        created_at: Set(chrono::Utc::now()),
    };

    let user = new_user
        .insert(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {e}")))?;

    Ok(UserDto {
        id: user.id.to_string(),
        email: user.email,
        enabled: user.enabled,
        notify_on_change_only: user.notify_on_change_only,
        scrape_interval_secs: user.scrape_interval_secs,
        discord_webhook_url: user.discord_webhook_url,
        created_at: user.created_at.to_string(),
    })
}

#[server]
pub async fn update_user(id: String, form: UserFormDto) -> Result<UserDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    let user_id =
        Uuid::parse_str(&id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {e}")))?;

    let user = Users::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new("User not found".to_string()))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.email = Set(form.email.clone());
    active_user.enabled = Set(form.enabled);
    active_user.notify_on_change_only = Set(form.notify_on_change_only);
    active_user.scrape_interval_secs = Set(form.scrape_interval_secs);
    active_user.discord_webhook_url = Set(form.discord_webhook_url.clone());

    let updated_user = active_user
        .update(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update user: {e}")))?;

    Ok(UserDto {
        id: updated_user.id.to_string(),
        email: updated_user.email,
        enabled: updated_user.enabled,
        notify_on_change_only: updated_user.notify_on_change_only,
        scrape_interval_secs: updated_user.scrape_interval_secs,
        discord_webhook_url: updated_user.discord_webhook_url,
        created_at: updated_user.created_at.to_string(),
    })
}

#[server]
pub async fn delete_user(id: String) -> Result<(), ServerFnError> {
    let db = db::get_db_from_context()?;
    let user_id =
        Uuid::parse_str(&id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {e}")))?;

    Users::delete_by_id(user_id)
        .exec(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete user: {e}")))?;

    Ok(())
}

#[server]
pub async fn get_routes(area_id: i32) -> Result<Vec<RouteDto>, ServerFnError> {
    let db = db::get_db_from_context()?;

    let routes = Routes::find()
        .filter(routes::Column::AreaId.eq(area_id))
        .all(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(routes
        .into_iter()
        .map(|r| RouteDto {
            route_id: r.route_id,
            area_id: r.area_id,
            name: r.name,
        })
        .collect())
}

#[server]
pub async fn get_stations_for_route(
    route_id: i32,
    area_id: i32,
) -> Result<Vec<StationDto>, ServerFnError> {
    let db = db::get_db_from_context()?;

    let stations = Stations::find()
        .filter(
            stations::Column::AreaId
                .eq(area_id)
                .and(stations::Column::RouteId.eq(route_id)),
        )
        .all(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(stations
        .into_iter()
        .map(|s| StationDto {
            station_id: s.station_id,
            name: s.name,
            area_id: s.area_id,
        })
        .collect())
}

#[server]
pub async fn create_user_route(form: UserRouteFormDto) -> Result<UserRouteDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    let user_id = Uuid::parse_str(&form.user_id)
        .map_err(|e| ServerFnError::new(format!("Invalid user UUID: {e}")))?;

    let route_id = Uuid::new_v4();

    let new_route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(form.area_id),
        route_id: Set(form.route_id),
        departure_station: Set(form.departure_station.clone()),
        arrival_station: Set(form.arrival_station.clone()),
        date_start: Set(form.date_start.clone()),
        date_end: Set(form.date_end.clone()),
        departure_time_min: Set(form.departure_time_min.clone()),
        departure_time_max: Set(form.departure_time_max.clone()),
        created_at: Set(chrono::Utc::now()),
    };

    let route = new_route
        .insert(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create route: {e}")))?;

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
        .insert(&db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create passengers: {e}")))?;

    Ok(UserRouteDto {
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
    })
}
