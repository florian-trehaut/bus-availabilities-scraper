use crate::error::Result;
use crate::scraper::BusScraper;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::collections::HashSet;
use tracing::{error, info, warn};

pub async fn seed_routes_catalog(db: &DatabaseConnection, scraper: &BusScraper) -> Result<()> {
    info!("Starting routes catalog seeding...");

    let area_ids = vec![1];

    for area_id in area_ids {
        info!("Fetching routes for area_id={}", area_id);

        let routes = match scraper.fetch_routes(area_id).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to fetch routes for area {}: {}", area_id, e);
                continue;
            }
        };

        info!("Found {} routes for area {}", routes.len(), area_id);

        for route in routes {
            let route_id_str = route.id.clone();
            let route_name = route.name.clone();

            if let Err(e) = seed_single_route(db, scraper, area_id, &route).await {
                error!(
                    "Failed to seed route {} ({}): {}",
                    route_id_str, route_name, e
                );
            }
        }
    }

    info!("Routes catalog seeding completed");
    Ok(())
}

async fn seed_single_route(
    db: &DatabaseConnection,
    scraper: &BusScraper,
    area_id: u32,
    route: &crate::types::Route,
) -> Result<()> {
    use crate::entities::{prelude::Routes, routes};

    let existing = Routes::find_by_id(route.id.clone()).one(db).await?;

    if existing.is_none() {
        let route_model = routes::ActiveModel {
            route_id: Set(route.id.clone()),
            area_id: Set(area_id as i32),
            name: Set(route.name.clone()),
            switch_changeable_flg: Set(route.switch_changeable_flg.clone()),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        route_model.insert(db).await?;
        info!("Inserted route: {} - {}", route.id, route.name);
    } else {
        info!("Route {} already exists, skipping", route.id);
    }

    let departure_stations = match scraper.fetch_departure_stations(&route.id).await {
        Ok(s) => s,
        Err(e) => {
            warn!(
                "Failed to fetch departure stations for route {}: {}",
                route.id, e
            );
            return Ok(());
        }
    };

    info!(
        "Found {} departure stations for route {}",
        departure_stations.len(),
        route.id
    );

    let mut all_stations = HashSet::new();
    for station in &departure_stations {
        all_stations.insert((station.id.clone(), station.name.clone()));
    }

    for departure in &departure_stations {
        let arrival_stations = match scraper
            .fetch_arrival_stations(&route.id, &departure.id)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Failed to fetch arrival stations for route {} from {}: {}",
                    route.id, departure.id, e
                );
                continue;
            }
        };

        for station in &arrival_stations {
            all_stations.insert((station.id.clone(), station.name.clone()));
        }
    }

    info!(
        "Total unique stations for route {}: {}",
        route.id,
        all_stations.len()
    );

    for (station_id, station_name) in all_stations {
        if let Err(e) = seed_station(db, &station_id, &station_name, area_id, &route.id).await {
            error!(
                "Failed to insert station {} ({}): {}",
                station_id, station_name, e
            );
        }
    }

    Ok(())
}

async fn seed_station(
    db: &DatabaseConnection,
    station_id: &str,
    station_name: &str,
    area_id: u32,
    route_id: &str,
) -> Result<()> {
    use crate::entities::{prelude::*, stations};

    let existing = Stations::find_by_id(station_id).one(db).await?;

    if existing.is_none() {
        let station_model = stations::ActiveModel {
            station_id: Set(station_id.to_string()),
            name: Set(station_name.to_string()),
            area_id: Set(area_id as i32),
            route_id: Set(route_id.parse::<i32>().ok()),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        station_model.insert(db).await?;
        info!("Inserted station: {} - {}", station_id, station_name);
    }

    Ok(())
}
