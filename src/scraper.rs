use crate::error::{Result, ScraperError};
use crate::html_parser;
use crate::types::{
    AvailabilityResult, AvailableDate, BusSchedule, DateSlot, Route, ScrapeRequest, Station,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

#[allow(dead_code)]
const MAX_RETRIES: u32 = 3;
#[allow(dead_code)]
const RETRY_DELAY_MS: u64 = 1000;
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

pub struct BusScraper {
    client: Client,
    base_url: String,
}

impl BusScraper {
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(ScraperError::Http)?;

        Ok(Self { client, base_url })
    }

    #[allow(dead_code)]
    pub async fn check_availability(&self, request: &ScrapeRequest) -> Result<AvailabilityResult> {
        let routes = self.fetch_routes(request.area_id).await?;
        let route = routes
            .iter()
            .find(|r| r.id == request.route_id.to_string())
            .ok_or_else(|| ScraperError::InvalidResponse("Route not found".to_string()))?;

        let departure_stations = self.fetch_departure_stations(&route.id).await?;
        let departure = departure_stations
            .iter()
            .find(|s| s.id == request.departure_station)
            .ok_or_else(|| {
                ScraperError::InvalidResponse("Departure station not found".to_string())
            })?;

        let arrival_stations = self
            .fetch_arrival_stations(&route.id, &departure.id)
            .await?;
        let arrival = arrival_stations
            .iter()
            .find(|s| s.id == request.arrival_station)
            .ok_or_else(|| {
                ScraperError::InvalidResponse("Arrival station not found".to_string())
            })?;

        let available_dates = self
            .fetch_available_dates(&route.id, &departure.id, &arrival.id)
            .await?;

        debug!(
            "Found {} available dates for route {} ({} -> {})",
            available_dates.len(),
            route.name,
            departure.name,
            arrival.name
        );

        let date_slots: Vec<DateSlot> = available_dates
            .into_iter()
            .map(|d| DateSlot {
                id: d.id,
                name: d.name,
            })
            .collect();

        Ok(AvailabilityResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            route_id: route.id.clone(),
            route_name: route.name.clone(),
            departure_id: departure.id.clone(),
            departure_name: departure.name.clone(),
            arrival_id: arrival.id.clone(),
            arrival_name: arrival.name.clone(),
            date: "N/A".to_string(),
            available_dates: date_slots,
        })
    }

    pub async fn check_availability_full(
        &self,
        request: &ScrapeRequest,
    ) -> Result<Vec<BusSchedule>> {
        let dates = request.date_range.dates()?;
        let mut all_schedules = Vec::new();

        for date in dates {
            debug!("Fetching schedules for date: {}", date);

            match self.fetch_schedules(request, &date).await {
                Ok(schedules) => {
                    debug!("Found {} schedules for date {}", schedules.len(), date);
                    all_schedules.extend(schedules);
                }
                Err(e) => {
                    warn!("Failed to fetch schedules for date {}: {}", date, e);
                }
            }
        }

        Ok(all_schedules)
    }

    #[allow(dead_code)]
    pub async fn fetch_routes(&self, area_id: u32) -> Result<Vec<Route>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(&url, &[("mode", "line:full"), ("id", &area_id.to_string())])
            .await?;

        parse_routes(&xml)
    }

    #[allow(dead_code)]
    pub async fn fetch_departure_stations(&self, route_id: &str) -> Result<Vec<Station>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(&url, &[("mode", "station_geton"), ("id", route_id)])
            .await?;

        parse_stations(&xml)
    }

    #[allow(dead_code)]
    pub async fn fetch_arrival_stations(
        &self,
        route_id: &str,
        departure_station: &str,
    ) -> Result<Vec<Station>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(
                &url,
                &[
                    ("mode", "station_getoff"),
                    ("id", route_id),
                    ("stationcd", departure_station),
                ],
            )
            .await?;

        parse_stations(&xml)
    }

    #[allow(dead_code)]
    pub async fn fetch_available_dates(
        &self,
        route_id: &str,
        departure_station: &str,
        arrival_station: &str,
    ) -> Result<Vec<AvailableDate>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(
                &url,
                &[
                    ("mode", "date"),
                    ("id", route_id),
                    ("onStation", departure_station),
                    ("offStation", arrival_station),
                ],
            )
            .await?;

        parse_dates(&xml)
    }

    pub async fn fetch_schedules(
        &self,
        request: &ScrapeRequest,
        date: &str,
    ) -> Result<Vec<BusSchedule>> {
        let url = format!("{}/reservation/rsvPlanList", self.base_url);

        let params = [
            ("mode", "search".to_string()),
            ("route", request.area_id.to_string()),
            ("lineId", request.route_id.to_string()),
            ("onStationCd", request.departure_station.clone()),
            ("offStationCd", request.arrival_station.clone()),
            ("bordingDate", date.to_string()),
            ("danseiNum", request.passengers.total_male().to_string()),
            ("zyoseiNum", request.passengers.total_female().to_string()),
            ("adultMen", request.passengers.adult_men.to_string()),
            ("adultWomen", request.passengers.adult_women.to_string()),
            ("childMen", request.passengers.child_men.to_string()),
            ("childWomen", request.passengers.child_women.to_string()),
            (
                "handicapAdultMen",
                request.passengers.handicap_adult_men.to_string(),
            ),
            (
                "handicapAdultWomen",
                request.passengers.handicap_adult_women.to_string(),
            ),
            (
                "handicapChildMen",
                request.passengers.handicap_child_men.to_string(),
            ),
            (
                "handicapChildWomen",
                request.passengers.handicap_child_women.to_string(),
            ),
        ];

        let html = self.fetch_schedules_html(&url, &params).await?;
        let mut schedules = html_parser::parse_schedules_html(&html, date)?;

        if let Some(ref filter) = request.time_filter {
            schedules.retain(|s| filter.matches(&s.departure_time));
        }

        Ok(schedules)
    }

    async fn fetch_schedules_html(&self, url: &str, params: &[(&str, String)]) -> Result<String> {
        let query_params: Vec<(&str, &str)> =
            params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", format!("{}/", self.base_url))
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ScraperError::InvalidResponse(format!(
                "HTTP {} for url={}",
                response.status(),
                url
            )));
        }

        let html = response.text().await?;
        debug!("Fetched schedules HTML, length: {}", html.len());

        #[cfg(debug_assertions)]
        {
            #[allow(clippy::disallowed_methods)]
            if std::env::var("SAVE_HTML").is_ok() {
                let _ = std::fs::write("/tmp/schedules.html", &html);
                debug!("Saved HTML to /tmp/schedules.html");
            }
        }

        Ok(html)
    }

    #[allow(dead_code)]
    async fn fetch_with_retry(&self, url: &str, params: &[(&str, &str)]) -> Result<String> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self.fetch_data(url, params).await {
                Ok(response) => return Ok(response),
                Err(ScraperError::ServiceUnavailable) if attempts < MAX_RETRIES => {
                    warn!(
                        "Service unavailable (attempt {}/{}), retrying in {}ms",
                        attempts,
                        MAX_RETRIES,
                        RETRY_DELAY_MS * attempts as u64
                    );
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempts as u64))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    #[allow(dead_code)]
    async fn fetch_data(&self, url: &str, params: &[(&str, &str)]) -> Result<String> {
        let response = self
            .client
            .post(url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", format!("{}/index", self.base_url))
            .form(params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ScraperError::InvalidResponse(format!(
                "HTTP {} for url={}",
                response.status(),
                url
            )));
        }

        let body = response.text().await?;
        debug!("Response body: {}", body);

        Ok(body)
    }
}

#[allow(dead_code)]
fn parse_routes(xml: &str) -> Result<Vec<Route>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut routes = Vec::new();
    let mut current_id = None;
    let mut current_name = None;
    let mut current_flag = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"id" => {
                    if let (Some(id), Some(name)) = (current_id.take(), current_name.take()) {
                        routes.push(Route {
                            id,
                            name,
                            switch_changeable_flg: current_flag.take(),
                        });
                    }
                    current_id = Some(read_text(&mut reader)?);
                }
                b"name" => {
                    current_name = Some(read_text(&mut reader)?);
                }
                b"switchChangeableFlg" => {
                    current_flag = Some(read_text(&mut reader)?);
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScraperError::Parse(format!("XML error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if let (Some(id), Some(name)) = (current_id, current_name) {
        routes.push(Route {
            id,
            name,
            switch_changeable_flg: current_flag,
        });
    }

    Ok(routes)
}

#[allow(dead_code)]
fn parse_stations(xml: &str) -> Result<Vec<Station>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut stations = Vec::new();
    let mut current_id = None;
    let mut current_name = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"id" => {
                    if let (Some(id), Some(name)) = (current_id.take(), current_name.take()) {
                        stations.push(Station { id, name });
                    }
                    current_id = Some(read_text(&mut reader)?);
                }
                b"name" => {
                    current_name = Some(read_text(&mut reader)?);
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScraperError::Parse(format!("XML error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if let (Some(id), Some(name)) = (current_id, current_name) {
        stations.push(Station { id, name });
    }

    Ok(stations)
}

#[allow(dead_code)]
fn parse_dates(xml: &str) -> Result<Vec<AvailableDate>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut dates = Vec::new();
    let mut current_id = None;
    let mut current_name = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"id" => {
                    if let (Some(id), Some(name)) = (current_id.take(), current_name.take()) {
                        dates.push(AvailableDate { id, name });
                    }
                    current_id = Some(read_text(&mut reader)?);
                }
                b"name" => {
                    current_name = Some(read_text(&mut reader)?);
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScraperError::Parse(format!("XML error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if let (Some(id), Some(name)) = (current_id, current_name) {
        dates.push(AvailableDate { id, name });
    }

    Ok(dates)
}

#[allow(dead_code)]
fn read_text(reader: &mut Reader<&[u8]>) -> Result<String> {
    let mut buf = Vec::new();
    match reader.read_event_into(&mut buf) {
        Ok(Event::Text(e)) => e
            .unescape()
            .map(|s| s.to_string())
            .map_err(|e| ScraperError::Parse(format!("Text unescape error: {}", e))),
        _ => Ok(String::new()),
    }
}
