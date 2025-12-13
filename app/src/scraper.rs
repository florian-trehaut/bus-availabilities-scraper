use crate::error::{Result, ScraperError};
use crate::html_parser;
use crate::types::{BusSchedule, Route, ScrapeRequest, Station};
use quick_xml::Reader;
use quick_xml::events::Event;
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
            .fetch_with_retry(
                &url,
                &[
                    ("mode", "line:full"),
                    ("id", &area_id.to_string()),
                    ("lang", "EN"),
                ],
            )
            .await?;

        parse_routes(&xml)
    }

    pub async fn fetch_departure_stations(&self, route_id: &str) -> Result<Vec<Station>> {
        let url = format!("{}/ajaxPulldown", self.base_url);
        let xml = self
            .fetch_with_retry(
                &url,
                &[("mode", "station_geton"), ("id", route_id), ("lang", "EN")],
            )
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
                    ("lang", "EN"),
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

        #[cfg(debug_assertions)]
        {
            let _ = std::fs::write("/tmp/schedules.html", &html);
            debug!("Saved HTML to /tmp/schedules.html");
        }

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
                        RETRY_DELAY_MS * u64::from(attempts)
                    );
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * u64::from(attempts)))
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
            Err(e) => return Err(ScraperError::Parse(format!("XML error: {e}"))),
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
            Err(e) => return Err(ScraperError::Parse(format!("XML error: {e}"))),
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
            .map_err(|e| ScraperError::Parse(format!("Text unescape error: {e}"))),
        _ => Ok(String::new()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // === parse_routes TESTS ===

    #[test]
    fn test_parse_routes_valid_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<routes>
    <id>110</id>
    <name>新宿～富士五湖線</name>
    <switchChangeableFlg>1</switchChangeableFlg>
    <id>155</id>
    <name>新宿～上高地線</name>
    <switchChangeableFlg>0</switchChangeableFlg>
</routes>"#;

        let routes = parse_routes(xml).unwrap();
        assert_eq!(routes.len(), 2);

        assert_eq!(routes[0].id, "110");
        assert_eq!(routes[0].name, "新宿～富士五湖線");
        assert_eq!(routes[0].switch_changeable_flg, Some("1".to_string()));

        assert_eq!(routes[1].id, "155");
        assert_eq!(routes[1].name, "新宿～上高地線");
        assert_eq!(routes[1].switch_changeable_flg, Some("0".to_string()));
    }

    #[test]
    fn test_parse_routes_without_flag() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<routes>
    <id>200</id>
    <name>名古屋～福岡線</name>
</routes>"#;

        let routes = parse_routes(xml).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "200");
        assert_eq!(routes[0].name, "名古屋～福岡線");
        assert!(routes[0].switch_changeable_flg.is_none());
    }

    #[test]
    fn test_parse_routes_empty_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><routes></routes>"#;

        let routes = parse_routes(xml).unwrap();
        assert!(routes.is_empty());
    }

    #[test]
    fn test_parse_routes_single_route() {
        let xml = r"<id>123</id><name>Test Route</name>";

        let routes = parse_routes(xml).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "123");
        assert_eq!(routes[0].name, "Test Route");
    }

    #[test]
    fn test_parse_routes_with_unicode() {
        let xml = r"<id>123</id><name>日本語ルート名 テスト</name>";

        let routes = parse_routes(xml).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].name, "日本語ルート名 テスト");
    }

    // === parse_stations TESTS ===

    #[test]
    fn test_parse_stations_valid_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<stations>
    <id>001</id>
    <name>バスタ新宿（南口）</name>
    <id>064</id>
    <name>河口湖駅</name>
    <id>498</id>
    <name>上高地バスターミナル</name>
</stations>"#;

        let stations = parse_stations(xml).unwrap();
        assert_eq!(stations.len(), 3);

        assert_eq!(stations[0].id, "001");
        assert_eq!(stations[0].name, "バスタ新宿（南口）");

        assert_eq!(stations[1].id, "064");
        assert_eq!(stations[1].name, "河口湖駅");

        assert_eq!(stations[2].id, "498");
        assert_eq!(stations[2].name, "上高地バスターミナル");
    }

    #[test]
    fn test_parse_stations_empty_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><stations></stations>"#;

        let stations = parse_stations(xml).unwrap();
        assert!(stations.is_empty());
    }

    #[test]
    fn test_parse_stations_single_station() {
        let xml = r"<id>001</id><name>Test Station</name>";

        let stations = parse_stations(xml).unwrap();
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].id, "001");
        assert_eq!(stations[0].name, "Test Station");
    }

    #[test]
    fn test_parse_stations_with_unicode() {
        let xml = r"<id>001</id><name>東京駅 八重洲口</name>";

        let stations = parse_stations(xml).unwrap();
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].name, "東京駅 八重洲口");
    }

    // === read_text TESTS ===

    #[test]
    fn test_read_text_with_content() {
        let xml = "<root>Hello World</root>";
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        // Skip the opening tag
        let mut buf = Vec::new();
        let _ = reader.read_event_into(&mut buf);

        let text = read_text(&mut reader).unwrap();
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_read_text_with_special_chars() {
        let xml = "<root>Test &amp; Special &lt;chars&gt;</root>";
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let _ = reader.read_event_into(&mut buf);

        let text = read_text(&mut reader).unwrap();
        assert_eq!(text, "Test & Special <chars>");
    }

    #[test]
    fn test_read_text_empty_element() {
        let xml = "<root></root>";
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let _ = reader.read_event_into(&mut buf);

        let text = read_text(&mut reader).unwrap();
        assert_eq!(text, "");
    }

    // === BusScraper TESTS ===

    #[test]
    fn test_bus_scraper_new_success() {
        let scraper = BusScraper::new("https://example.com".to_string());
        assert!(scraper.is_ok());
    }

    #[test]
    fn test_bus_scraper_stores_base_url() {
        let scraper = BusScraper::new("https://test.example.com".to_string()).unwrap();
        assert_eq!(scraper.base_url, "https://test.example.com");
    }
}
