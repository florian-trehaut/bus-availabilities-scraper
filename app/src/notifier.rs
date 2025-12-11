use crate::error::Result;
use crate::types::{BusSchedule, SeatAvailability};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct NotificationContext {
    pub departure_station_name: String,
    pub arrival_station_name: String,
    pub date_range: (String, String),
    pub passenger_count: u8,
    pub time_filter: Option<(String, String)>,
}

pub struct DiscordNotifier {
    client: Client,
}

impl DiscordNotifier {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn send_startup_notification(
        &self,
        webhook_url: &str,
        user_count: usize,
        route_count: usize,
    ) -> Result<()> {
        let embed = json!({
            "title": "✅ Bot démarré",
            "description": format!(
                "Monitoring actif pour **{}** utilisateur(s) et **{}** route(s)",
                user_count, route_count
            ),
            "color": 5763719,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        match self
            .client
            .post(webhook_url)
            .json(&json!({ "embeds": [embed] }))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Startup notification sent successfully");
                    Ok(())
                } else {
                    error!(
                        "Startup notification failed with status: {}",
                        response.status()
                    );
                    Ok(())
                }
            }
            Err(e) => {
                error!("Failed to send startup notification: {}", e);
                Ok(())
            }
        }
    }

    pub async fn send_availability_alert(
        &self,
        webhook_url: &str,
        schedules: &[BusSchedule],
        context: &NotificationContext,
    ) -> Result<()> {
        if schedules.is_empty() {
            return Ok(());
        }

        let embed = self.build_embed(schedules, context);

        match self
            .client
            .post(webhook_url)
            .json(&json!({ "embeds": [embed] }))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Discord notification sent successfully");
                    Ok(())
                } else {
                    error!("Discord webhook failed with status: {}", response.status());
                    Ok(())
                }
            }
            Err(e) => {
                error!("Failed to send Discord notification: {}", e);
                Ok(())
            }
        }
    }

    fn build_embed(
        &self,
        schedules: &[BusSchedule],
        context: &NotificationContext,
    ) -> serde_json::Value {
        let mut fields = Vec::new();
        let mut count_with_plans = 0;

        for schedule in schedules {
            if schedule.available_plans.is_empty() {
                continue;
            }

            count_with_plans += 1;

            let formatted_date = format_date(&schedule.departure_date);

            for plan in &schedule.available_plans {
                let seats_info = match &plan.availability {
                    SeatAvailability::Available { remaining_seats } => match remaining_seats {
                        Some(n) => format!("{n} sièges"),
                        None => "Places dispo".to_string(),
                    },
                };

                let bus_info = format!(
                    "📅 **{}** à **{}**\n🕐 Arrivée : {}\n💺 {}\n💰 {}",
                    formatted_date,
                    schedule.departure_time,
                    schedule.arrival_time,
                    seats_info,
                    plan.display_price
                );

                fields.push(json!({
                    "name": format!("🚌 Bus {} - Plan {}", schedule.bus_number, plan.plan_id),
                    "value": bus_info,
                    "inline": false
                }));
            }
        }

        let description = format!(
            "**{}** bus avec places disponibles\n📍 {} → {}\n📆 {} — {}",
            count_with_plans,
            context.departure_station_name,
            context.arrival_station_name,
            format_date(&context.date_range.0),
            format_date(&context.date_range.1)
        );

        let footer_text = if let Some((min, max)) = &context.time_filter {
            format!(
                "{} passager(s) | Horaires : {} - {}",
                context.passenger_count, min, max
            )
        } else {
            format!("{} passager(s) | Tous horaires", context.passenger_count)
        };

        json!({
            "title": "🚌 Bus disponibles !",
            "description": description,
            "color": 3066993,
            "fields": fields,
            "footer": {
                "text": footer_text
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }
}

fn format_date(date_yyyymmdd: &str) -> String {
    if date_yyyymmdd.len() == 8 {
        format!(
            "{}/{}/{}",
            &date_yyyymmdd[6..8],
            &date_yyyymmdd[4..6],
            &date_yyyymmdd[0..4]
        )
    } else {
        date_yyyymmdd.to_string()
    }
}

impl Default for DiscordNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::PricingPlan;

    #[test]
    fn test_build_embed() {
        let notifier = DiscordNotifier::new();

        let schedules = vec![BusSchedule {
            bus_number: "Bus_1".to_string(),
            route_name: String::new(),
            departure_station: String::new(),
            departure_date: "20251029".to_string(),
            departure_time: "22:25".to_string(),
            arrival_station: String::new(),
            arrival_date: "20251030".to_string(),
            arrival_time: "5:20".to_string(),
            way_no: 0,
            available_plans: vec![PricingPlan {
                plan_id: 12345,
                plan_index: 1,
                plan_name: String::new(),
                price: 12000,
                display_price: "12,000円".to_string(),
                availability: SeatAvailability::Available {
                    remaining_seats: Some(3),
                },
            }],
        }];

        let context = NotificationContext {
            departure_station_name: "Shinjuku".to_string(),
            arrival_station_name: "Kamikochi".to_string(),
            date_range: ("20251029".to_string(), "20251105".to_string()),
            passenger_count: 2,
            time_filter: Some(("20:00".to_string(), "23:59".to_string())),
        };

        let embed = notifier.build_embed(&schedules, &context);

        assert_eq!(embed["title"], "🚌 Bus disponibles !");
        assert_eq!(embed["color"], 3066993);
        assert!(embed["fields"].is_array());
        assert!(!embed["fields"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_build_embed_empty() {
        let notifier = DiscordNotifier::new();
        let schedules = vec![];

        let context = NotificationContext {
            departure_station_name: "Shinjuku".to_string(),
            arrival_station_name: "Kamikochi".to_string(),
            date_range: ("20251029".to_string(), "20251105".to_string()),
            passenger_count: 2,
            time_filter: None,
        };

        let embed = notifier.build_embed(&schedules, &context);

        assert_eq!(embed["title"], "🚌 Bus disponibles !");
        let desc = embed["description"].as_str().unwrap();
        assert!(desc.contains("Shinjuku"));
        assert!(desc.contains("Kamikochi"));
    }

    #[test]
    fn test_format_date() {
        assert_eq!(format_date("20251029"), "29/10/2025");
        assert_eq!(format_date("20250101"), "01/01/2025");
        assert_eq!(format_date("invalid"), "invalid");
    }
}
