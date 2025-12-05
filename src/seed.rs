use crate::config::Config;
use crate::entities::{prelude::*, user_passengers, user_routes, users};
use crate::error::{Result, ScraperError};
use crate::types::PassengerCount;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use tracing::{info, warn};
use uuid::Uuid;

const SEED_USER_EMAIL: &str = "beta@bus-scraper.local";

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

    let existing_user = Users::find()
        .filter(users::Column::Email.eq(SEED_USER_EMAIL))
        .one(db)
        .await?;

    let user_id = if let Some(existing) = existing_user {
        info!("Found existing user with email: {}", SEED_USER_EMAIL);

        let mut user_active: users::ActiveModel = existing.into_active_model();
        user_active.enabled = Set(true);
        user_active.notify_on_change_only = Set(config.notify_on_change_only);
        user_active.scrape_interval_secs = Set(config.scrape_interval_secs as i64);
        user_active.discord_webhook_url = Set(config.discord_webhook_url.clone());

        let updated_user = user_active.update(db).await?;
        info!("Updated user configuration for: {}", SEED_USER_EMAIL);

        updated_user.id
    } else {
        let user_id = Uuid::new_v4();
        let user = users::ActiveModel {
            id: Set(user_id),
            email: Set(SEED_USER_EMAIL.to_string()),
            enabled: Set(true),
            notify_on_change_only: Set(config.notify_on_change_only),
            scrape_interval_secs: Set(config.scrape_interval_secs as i64),
            discord_webhook_url: Set(config.discord_webhook_url.clone()),
            created_at: Set(chrono::Utc::now()),
        };
        user.insert(db).await?;
        info!("Created user with ID: {}", user_id);

        user_id
    };

    let existing_route = UserRoutes::find()
        .filter(user_routes::Column::UserId.eq(user_id))
        .filter(user_routes::Column::AreaId.eq(config.request.area_id as i32))
        .filter(user_routes::Column::RouteId.eq(config.request.route_id as i32))
        .filter(user_routes::Column::DepartureStation.eq(&config.request.departure_station))
        .filter(user_routes::Column::ArrivalStation.eq(&config.request.arrival_station))
        .one(db)
        .await?;

    let route_id = if let Some(existing) = existing_route {
        info!("Found existing route with ID: {}", existing.id);

        let mut route_active: user_routes::ActiveModel = existing.into_active_model();
        route_active.date_start = Set(config.request.date_range.start.clone());
        route_active.date_end = Set(config.request.date_range.end.clone());
        route_active.departure_time_min = Set(config
            .request
            .time_filter
            .as_ref()
            .and_then(|f| f.departure_min.clone()));
        route_active.departure_time_max = Set(config
            .request
            .time_filter
            .as_ref()
            .and_then(|f| f.departure_max.clone()));

        let updated_route = route_active.update(db).await?;
        info!("Updated route with ID: {}", updated_route.id);

        updated_route.id
    } else {
        let route_id = Uuid::new_v4();
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

        route_id
    };

    let existing_passengers = UserPassengers::find_by_id(route_id).one(db).await?;

    if let Some(existing) = existing_passengers {
        let mut passengers_active: user_passengers::ActiveModel = existing.into_active_model();
        apply_passenger_fields(&mut passengers_active, &config.request.passengers);

        passengers_active.update(db).await?;
        info!("Updated passenger configuration for route {}", route_id);
    } else {
        let mut passengers = user_passengers::ActiveModel {
            user_route_id: Set(route_id),
            ..Default::default()
        };
        apply_passenger_fields(&mut passengers, &config.request.passengers);

        passengers.insert(db).await?;
        info!("Created passenger configuration for route {}", route_id);
    }

    Ok(())
}

fn apply_passenger_fields(model: &mut user_passengers::ActiveModel, passengers: &PassengerCount) {
    model.adult_men = Set(passengers.adult_men as i16);
    model.adult_women = Set(passengers.adult_women as i16);
    model.child_men = Set(passengers.child_men as i16);
    model.child_women = Set(passengers.child_women as i16);
    model.handicap_adult_men = Set(passengers.handicap_adult_men as i16);
    model.handicap_adult_women = Set(passengers.handicap_adult_women as i16);
    model.handicap_child_men = Set(passengers.handicap_child_men as i16);
    model.handicap_child_women = Set(passengers.handicap_child_women as i16);
}
