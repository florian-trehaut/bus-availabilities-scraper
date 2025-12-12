use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use crate::{api_impl, db, scraper::BusScraper};

/// Get the `BusScraper` from Leptos context
#[cfg(feature = "ssr")]
pub fn get_scraper_from_context() -> Result<Arc<BusScraper>, ServerFnError> {
    use leptos::prelude::expect_context;
    Ok(expect_context::<Arc<BusScraper>>())
}

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
    pub route_id: String,
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
    pub route_id: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRouteWithPassengersDto {
    pub id: String,
    pub user_id: String,
    pub area_id: i32,
    pub route_id: String,
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

#[server]
pub async fn get_users() -> Result<Vec<UserDto>, ServerFnError> {
    let db = db::get_db_from_context()?;
    api_impl::get_users_impl(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_user(form: UserFormDto) -> Result<UserDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    api_impl::create_user_impl(&db, form)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_user(id: String, form: UserFormDto) -> Result<UserDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    let uuid = api_impl::parse_uuid(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    api_impl::update_user_impl(&db, uuid, form)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_user(id: String) -> Result<(), ServerFnError> {
    let db = db::get_db_from_context()?;
    let uuid = api_impl::parse_uuid(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    api_impl::delete_user_impl(&db, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetch routes from Highway Bus API for a given area
#[server]
pub async fn get_routes(area_id: i32) -> Result<Vec<RouteDto>, ServerFnError> {
    let scraper = get_scraper_from_context()?;
    api_impl::fetch_and_translate_routes(&scraper, area_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetch departure stations from Highway Bus API for a given route
#[server]
pub async fn get_departure_stations(route_id: String) -> Result<Vec<StationDto>, ServerFnError> {
    let scraper = get_scraper_from_context()?;
    api_impl::fetch_and_translate_departure_stations(&scraper, &route_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetch arrival stations from Highway Bus API for a given route and departure station
#[server]
pub async fn get_arrival_stations(
    route_id: String,
    departure_station_id: String,
) -> Result<Vec<StationDto>, ServerFnError> {
    let scraper = get_scraper_from_context()?;
    api_impl::fetch_and_translate_arrival_stations(&scraper, &route_id, &departure_station_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_user_route(form: UserRouteFormDto) -> Result<UserRouteDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    api_impl::create_user_route_impl(&db, form)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_user_routes(
    user_id: String,
) -> Result<Vec<UserRouteWithPassengersDto>, ServerFnError> {
    let db = db::get_db_from_context()?;
    let uuid = api_impl::parse_uuid(&user_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    api_impl::get_user_routes_impl(&db, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_user_route(
    id: String,
    form: UserRouteFormDto,
) -> Result<UserRouteDto, ServerFnError> {
    let db = db::get_db_from_context()?;
    let uuid = api_impl::parse_uuid(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    api_impl::update_user_route_impl(&db, uuid, form)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_user_route(id: String) -> Result<(), ServerFnError> {
    let db = db::get_db_from_context()?;
    let uuid = api_impl::parse_uuid(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    api_impl::delete_user_route_impl(&db, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
