//! Integration tests for scraper.rs using wiremock
//!
//! Tests HTTP interactions with mocked external API

use app::scraper::BusScraper;
use app::types::{DateRange, PassengerCount, ScrapeRequest, TimeFilter};
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a standard ScrapeRequest for tests
fn test_scrape_request(date: &str) -> ScrapeRequest {
    ScrapeRequest {
        area_id: 100,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_range: DateRange {
            start: date.to_string(),
            end: date.to_string(),
        },
        passengers: PassengerCount::default(),
        time_filter: None,
    }
}

// === fetch_routes TESTS ===

#[tokio::test]
async fn test_fetch_routes_success() {
    let mock_server = MockServer::start().await;

    let routes_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<routes>
    <id>110</id>
    <name>新宿～富士五湖線</name>
    <switchChangeableFlg>1</switchChangeableFlg>
    <id>155</id>
    <name>新宿～上高地線</name>
    <switchChangeableFlg>0</switchChangeableFlg>
</routes>"#;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=line%3Afull"))
        .and(body_string_contains("id=100"))
        .respond_with(ResponseTemplate::new(200).set_body_string(routes_xml))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let routes = scraper.fetch_routes(100).await.unwrap();

    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].id, "110");
    assert_eq!(routes[0].name, "新宿～富士五湖線");
    assert_eq!(routes[1].id, "155");
}

#[tokio::test]
async fn test_fetch_routes_empty_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"<?xml version="1.0"?><routes></routes>"#),
        )
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let routes = scraper.fetch_routes(100).await.unwrap();

    assert!(routes.is_empty());
}

#[tokio::test]
async fn test_fetch_routes_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let result = scraper.fetch_routes(100).await;

    assert!(result.is_err());
}

// === fetch_departure_stations TESTS ===

#[tokio::test]
async fn test_fetch_departure_stations_success() {
    let mock_server = MockServer::start().await;

    let stations_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<stations>
    <id>001</id>
    <name>バスタ新宿（南口）</name>
    <id>064</id>
    <name>河口湖駅</name>
</stations>"#;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=station_geton"))
        .and(body_string_contains("id=110"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stations_xml))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let stations = scraper.fetch_departure_stations("110").await.unwrap();

    assert_eq!(stations.len(), 2);
    assert_eq!(stations[0].id, "001");
    assert_eq!(stations[0].name, "バスタ新宿（南口）");
}

// === fetch_arrival_stations TESTS ===

#[tokio::test]
async fn test_fetch_arrival_stations_success() {
    let mock_server = MockServer::start().await;

    let stations_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<stations>
    <id>064</id>
    <name>河口湖駅</name>
    <id>065</id>
    <name>富士急ハイランド</name>
</stations>"#;

    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .and(body_string_contains("mode=station_getoff"))
        .and(body_string_contains("id=110"))
        .and(body_string_contains("stationcd=001"))
        .respond_with(ResponseTemplate::new(200).set_body_string(stations_xml))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let stations = scraper.fetch_arrival_stations("110", "001").await.unwrap();

    assert_eq!(stations.len(), 2);
    assert_eq!(stations[0].id, "064");
    assert_eq!(stations[1].name, "富士急ハイランド");
}

// === fetch_schedules TESTS ===

#[tokio::test]
async fn test_fetch_schedules_success() {
    let mock_server = MockServer::start().await;

    // Use the correct HTML structure expected by the parser:
    // section.busSvclistItem with li.dep/li.arr > p.time and form[name='selectPlan']
    let schedules_html = r#"<!DOCTYPE html>
<html><body>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">7:45 発</p></li>
            <li class="arr"><p class="time">10:00 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,100円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="12345">
                <button>残り5席</button>
            </form>
        </div>
    </section>
</body></html>"#;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .and(query_param("mode", "search"))
        .and(query_param("bordingDate", "20250115"))
        .respond_with(ResponseTemplate::new(200).set_body_string(schedules_html))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let request = test_scrape_request("2025-01-15");
    let schedules = scraper.fetch_schedules(&request, "20250115").await.unwrap();

    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].departure_time, "7:45");
    assert_eq!(schedules[0].arrival_time, "10:00");
    assert!(!schedules[0].available_plans.is_empty());
    assert_eq!(schedules[0].available_plans[0].price, 2100);
}

#[tokio::test]
async fn test_fetch_schedules_no_buses() {
    let mock_server = MockServer::start().await;

    let html = r#"<!DOCTYPE html><html><body><div>No buses available</div></body></html>"#;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let request = test_scrape_request("2025-01-20");
    let schedules = scraper.fetch_schedules(&request, "20250120").await.unwrap();

    assert!(schedules.is_empty());
}

#[tokio::test]
async fn test_fetch_schedules_with_time_filter() {
    let mock_server = MockServer::start().await;

    let schedules_html = r#"<!DOCTYPE html>
<html><body>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">6:00 発</p></li>
            <li class="arr"><p class="time">8:00 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,000円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="1">
                <button>残り5席</button>
            </form>
        </div>
    </section>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">10:30 発</p></li>
            <li class="arr"><p class="time">12:30 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,000円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="2">
                <button>残り5席</button>
            </form>
        </div>
    </section>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">15:00 発</p></li>
            <li class="arr"><p class="time">17:00 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,000円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="3">
                <button>残り5席</button>
            </form>
        </div>
    </section>
</body></html>"#;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(ResponseTemplate::new(200).set_body_string(schedules_html))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let mut request = test_scrape_request("2025-01-15");
    request.time_filter = Some(TimeFilter {
        departure_min: Some("08:00".to_string()),
        departure_max: Some("14:00".to_string()),
    });

    let schedules = scraper.fetch_schedules(&request, "20250115").await.unwrap();

    // Only 10:30 should match (after 08:00 and before 14:00)
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].departure_time, "10:30");
}

#[tokio::test]
async fn test_fetch_schedules_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let request = test_scrape_request("2025-01-15");
    let result = scraper.fetch_schedules(&request, "20250115").await;

    assert!(result.is_err());
}

// === Retry logic TESTS ===

#[tokio::test]
async fn test_retry_on_503_returns_error() {
    let mock_server = MockServer::start().await;

    // 503 is converted to InvalidResponse, not ServiceUnavailable
    // So it errors immediately without retrying
    Mock::given(method("POST"))
        .and(path("/ajaxPulldown"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();
    let result = scraper.fetch_routes(100).await;

    assert!(result.is_err());
}

// === Network error TESTS ===

#[tokio::test]
async fn test_network_error_invalid_url() {
    let scraper = BusScraper::new("http://localhost:1".to_string()).unwrap();
    let result = scraper.fetch_routes(100).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_scraper_with_different_base_urls() {
    let mock_server1 = MockServer::start().await;
    let mock_server2 = MockServer::start().await;

    let routes_xml1 = r#"<id>111</id><name>Route 1</name>"#;
    let routes_xml2 = r#"<id>222</id><name>Route 2</name>"#;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(routes_xml1))
        .mount(&mock_server1)
        .await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(routes_xml2))
        .mount(&mock_server2)
        .await;

    let scraper1 = BusScraper::new(mock_server1.uri()).unwrap();
    let scraper2 = BusScraper::new(mock_server2.uri()).unwrap();

    let routes1 = scraper1.fetch_routes(100).await.unwrap();
    let routes2 = scraper2.fetch_routes(100).await.unwrap();

    assert_eq!(routes1[0].id, "111");
    assert_eq!(routes2[0].id, "222");
}

// === check_availability_full TESTS ===

#[tokio::test]
async fn test_check_availability_full_multiple_dates() {
    let mock_server = MockServer::start().await;

    // Mock that returns different schedules - will be called for each date
    let schedules_html = r#"<!DOCTYPE html>
<html><body>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">9:00 発</p></li>
            <li class="arr"><p class="time">12:00 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">3,000円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="100">
                <button>残り3席</button>
            </form>
        </div>
    </section>
</body></html>"#;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(ResponseTemplate::new(200).set_body_string(schedules_html))
        .expect(3) // 3 dates = 3 requests
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();

    // Request spanning 3 days
    let request = ScrapeRequest {
        area_id: 100,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_range: DateRange {
            start: "2025-01-15".to_string(),
            end: "2025-01-17".to_string(), // 3 days: 15, 16, 17
        },
        passengers: PassengerCount::default(),
        time_filter: None,
    };

    let schedules = scraper.check_availability_full(&request).await.unwrap();

    // Should have schedules from all 3 days
    assert_eq!(schedules.len(), 3);
}

#[tokio::test]
async fn test_check_availability_full_with_some_failures() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mock_server = MockServer::start().await;

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    // Respond with success for some requests, 500 error for others
    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(move |_: &wiremock::Request| {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            if count == 1 {
                // Fail on second request
                ResponseTemplate::new(500)
            } else {
                ResponseTemplate::new(200).set_body_string(
                    r#"<!DOCTYPE html>
<html><body>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">10:00 発</p></li>
            <li class="arr"><p class="time">13:00 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,500円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="200">
                <button>残り5席</button>
            </form>
        </div>
    </section>
</body></html>"#,
                )
            }
        })
        .expect(3)
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();

    let request = ScrapeRequest {
        area_id: 100,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_range: DateRange {
            start: "2025-01-15".to_string(),
            end: "2025-01-17".to_string(),
        },
        passengers: PassengerCount::default(),
        time_filter: None,
    };

    // Should succeed even with one failure - should have 2 schedules (days 1 and 3)
    let schedules = scraper.check_availability_full(&request).await.unwrap();

    // 2 successful days out of 3
    assert_eq!(schedules.len(), 2);
}

#[tokio::test]
async fn test_check_availability_full_empty_results() {
    let mock_server = MockServer::start().await;

    // Mock returns empty HTML (no schedules)
    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<!DOCTYPE html><html><body></body></html>"#),
        )
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();

    let request = ScrapeRequest {
        area_id: 100,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_range: DateRange {
            start: "2025-01-15".to_string(),
            end: "2025-01-15".to_string(),
        },
        passengers: PassengerCount::default(),
        time_filter: None,
    };

    let schedules = scraper.check_availability_full(&request).await.unwrap();

    assert!(schedules.is_empty());
}

#[tokio::test]
async fn test_check_availability_full_single_date() {
    let mock_server = MockServer::start().await;

    let schedules_html = r#"<!DOCTYPE html>
<html><body>
    <section class="busSvclistItem">
        <ul>
            <li class="dep"><p class="time">8:30 発</p></li>
            <li class="arr"><p class="time">11:30 着</p></li>
        </ul>
        <div class="planArea">
            <p class="price">2,800円</p>
            <form name="selectPlan">
                <input type="hidden" class="seat_0" value="1" data-index="0">
                <input type="hidden" name="discntPlanNo" value="300">
                <button>残り8席</button>
            </form>
        </div>
    </section>
</body></html>"#;

    Mock::given(method("GET"))
        .and(path("/reservation/rsvPlanList"))
        .respond_with(ResponseTemplate::new(200).set_body_string(schedules_html))
        .expect(1)
        .mount(&mock_server)
        .await;

    let scraper = BusScraper::new(mock_server.uri()).unwrap();

    // Single date (start == end)
    let request = ScrapeRequest {
        area_id: 100,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date_range: DateRange {
            start: "2025-01-20".to_string(),
            end: "2025-01-20".to_string(),
        },
        passengers: PassengerCount::default(),
        time_filter: None,
    };

    let schedules = scraper.check_availability_full(&request).await.unwrap();

    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].departure_time, "8:30");
}
