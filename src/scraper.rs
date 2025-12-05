use crate::error::{Result, ScraperError};
use crate::html_parser;
use crate::types::{BusSchedule, Route, ScrapeRequest, Station};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

const MAX_RETRIES: u32 = 3;
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

    pub async fn fetch_routes(&self, area_id: u32) -> Result<Vec<Route>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(&url, &[("mode", "line:full"), ("id", &area_id.to_string())])
            .await?;

        parse_routes(&xml)
    }

    pub async fn fetch_departure_stations(&self, route_id: &str) -> Result<Vec<Station>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(&url, &[("mode", "station_geton"), ("id", route_id)])
            .await?;

        parse_stations(&xml)
    }

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

        Ok(html)
    }

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_fetch_routes_success() {
        let mock_server = MockServer::start().await;

        let xml_response = r#"
            <id>110</id>
            <name>Shinjuku - Fuji Five Lakes</name>
            <switchChangeableFlg>1</switchChangeableFlg>
            <id>155</id>
            <name>Shinjuku - Kamikochi</name>
        "#;

        Mock::given(method("POST"))
            .and(path("/ajaxPulldown"))
            .and(body_string_contains("mode=line"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml_response))
            .mount(&mock_server)
            .await;

        let scraper = BusScraper::new(mock_server.uri()).expect("Failed to create scraper");
        let routes = scraper
            .fetch_routes(1)
            .await
            .expect("Failed to fetch routes");

        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].id, "110");
        assert_eq!(routes[0].name, "Shinjuku - Fuji Five Lakes");
        assert_eq!(routes[1].id, "155");
        assert_eq!(routes[1].name, "Shinjuku - Kamikochi");
    }

    #[tokio::test]
    async fn test_fetch_departure_stations() {
        let mock_server = MockServer::start().await;

        let xml_response = r#"
            <id>001</id>
            <name>Busta Shinjuku</name>
            <id>002</id>
            <name>Shibuya Mark City</name>
        "#;

        Mock::given(method("POST"))
            .and(path("/ajaxPulldown"))
            .and(body_string_contains("mode=station_geton"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml_response))
            .mount(&mock_server)
            .await;

        let scraper = BusScraper::new(mock_server.uri()).expect("Failed to create scraper");
        let stations = scraper
            .fetch_departure_stations("110")
            .await
            .expect("Failed to fetch stations");

        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].id, "001");
        assert_eq!(stations[0].name, "Busta Shinjuku");
    }

    #[tokio::test]
    async fn test_fetch_routes_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let scraper = BusScraper::new(mock_server.uri()).expect("Failed to create scraper");
        let result = scraper.fetch_routes(1).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_routes_empty_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&mock_server)
            .await;

        let scraper = BusScraper::new(mock_server.uri()).expect("Failed to create scraper");
        let routes = scraper
            .fetch_routes(1)
            .await
            .expect("Failed to fetch routes");

        assert!(routes.is_empty());
    }

    #[tokio::test]
    async fn test_parse_routes_xml() {
        let xml = r#"
            <id>110</id>
            <name>Route A</name>
            <switchChangeableFlg>1</switchChangeableFlg>
            <id>120</id>
            <name>Route B</name>
        "#;

        let routes = parse_routes(xml).expect("Failed to parse routes");
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].id, "110");
        assert_eq!(routes[0].name, "Route A");
        assert_eq!(routes[0].switch_changeable_flg, Some("1".to_string()));
        assert_eq!(routes[1].id, "120");
        assert_eq!(routes[1].name, "Route B");
    }

    #[tokio::test]
    async fn test_parse_stations_xml() {
        let xml = r#"
            <id>001</id>
            <name>Station A</name>
            <id>002</id>
            <name>Station B</name>
        "#;

        let stations = parse_stations(xml).expect("Failed to parse stations");
        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].id, "001");
        assert_eq!(stations[0].name, "Station A");
    }
}
