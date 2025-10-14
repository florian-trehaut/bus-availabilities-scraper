use crate::config::Config;
use crate::entities::{prelude::*, user_passengers, user_routes, users};
use crate::error::{Result, ScraperError};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tracing::{info, warn};
use uuid::Uuid;

pub async fn seed_from_env(db: &DatabaseConnection) -> Result<()> {
    let config = Config::from_env()?;

    let route_id_str = config.request.route_id.to_string();
    let route_exists = Routes::find_by_id(route_id_str.clone())
        .one(db)
        .await?
        .is_some();

    if !route_exists {
        warn!(
            "Route {} not found in catalog. Consider running SEED_ROUTES_CATALOG=true first.",
            route_id_str
        );
    }

    let departure_exists = Stations::find_by_id(config.request.departure_station.clone())
        .one(db)
        .await?
        .is_some();

    if !departure_exists {
        return Err(ScraperError::Config(format!(
            "Departure station {} not found in catalog",
            config.request.departure_station
        )));
    }

    let arrival_exists = Stations::find_by_id(config.request.arrival_station.clone())
        .one(db)
        .await?
        .is_some();

    if !arrival_exists {
        return Err(ScraperError::Config(format!(
            "Arrival station {} not found in catalog",
            config.request.arrival_station
        )));
    }

    let user_id = Uuid::new_v4();
    let route_id = Uuid::new_v4();

    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set("beta@bus-scraper.local".to_string()),
        enabled: Set(true),
        notify_on_change_only: Set(config.notify_on_change_only),
        scrape_interval_secs: Set(config.scrape_interval_secs as i64),
        discord_webhook_url: Set(config.discord_webhook_url.clone()),
        created_at: Set(chrono::Utc::now()),
    };
    user.insert(db).await?;
    info!("Created user with ID: {}", user_id);

    let route = user_routes::ActiveModel {
        id: Set(route_id),
        user_id: Set(user_id),
        area_id: Set(config.request.area_id as i32),
        route_id: Set(config.request.route_id as i32),
        departure_station: Set(config.request.departure_station.clone()),
        arrival_station: Set(config.request.arrival_station.clone()),
        date_start: Set(config.request.date_range.start.clone()),
        date_end: Set(config.request.date_range.end.clone()),
        departure_time_min: Set(config
            .request
            .time_filter
            .as_ref()
            .and_then(|f| f.departure_min.clone())),
        departure_time_max: Set(config
            .request
            .time_filter
            .as_ref()
            .and_then(|f| f.departure_max.clone())),
        created_at: Set(chrono::Utc::now()),
    };
    route.insert(db).await?;
    info!("Created route with ID: {}", route_id);

    let passengers = user_passengers::ActiveModel {
        user_route_id: Set(route_id),
        adult_men: Set(config.request.passengers.adult_men as i16),
        adult_women: Set(config.request.passengers.adult_women as i16),
        child_men: Set(config.request.passengers.child_men as i16),
        child_women: Set(config.request.passengers.child_women as i16),
        handicap_adult_men: Set(config.request.passengers.handicap_adult_men as i16),
        handicap_adult_women: Set(config.request.passengers.handicap_adult_women as i16),
        handicap_child_men: Set(config.request.passengers.handicap_child_men as i16),
        handicap_child_women: Set(config.request.passengers.handicap_child_women as i16),
    };
    passengers.insert(db).await?;
    info!("Created passenger configuration for route {}", route_id);

    Ok(())
}
