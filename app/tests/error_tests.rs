use app::error::ScraperError;

// Test Display implementation for Parse variant
#[test]
fn test_parse_error_display() {
    let err = ScraperError::Parse("invalid XML".to_string());
    assert_eq!(err.to_string(), "XML parse error: invalid XML");
}

// Test Display implementation for Config variant
#[test]
fn test_config_error_display() {
    let err = ScraperError::Config("missing URL".to_string());
    assert_eq!(err.to_string(), "Configuration error: missing URL");
}

// Test Display implementation for ServiceUnavailable variant
#[test]
fn test_service_unavailable_display() {
    let err = ScraperError::ServiceUnavailable;
    assert_eq!(err.to_string(), "Service temporarily unavailable (503)");
}

// Test Display implementation for InvalidResponse variant
#[test]
fn test_invalid_response_display() {
    let err = ScraperError::InvalidResponse("unexpected format".to_string());
    assert_eq!(err.to_string(), "Invalid response: unexpected format");
}

// SSR-only tests
#[cfg(feature = "ssr")]
mod ssr_tests {
    use super::*;
    use sea_orm::DbErr;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Test Display implementation for Http variant using real reqwest error
    #[tokio::test]
    async fn test_http_error_display() {
        // Create a connection error by using an invalid URL
        let client = reqwest::Client::new();
        let result = client
            .get("http://invalid-domain-that-does-not-exist-12345.com")
            .send()
            .await;

        // Convert to ScraperError through Http variant
        if let Err(err) = result {
            let scraper_err = ScraperError::Http(err);
            assert!(scraper_err.to_string().starts_with("HTTP error:"));
        } else {
            panic!("Expected reqwest error");
        }
    }

    // Test Display implementation for Database variant
    #[test]
    fn test_database_error_display() {
        let err = ScraperError::Database("connection failed".to_string());
        assert!(err.to_string().contains("Database error:"));
        assert!(err.to_string().contains("connection failed"));
    }

    // Test Display implementation for NotFound variant
    #[test]
    fn test_not_found_error_display() {
        let err = ScraperError::NotFound("User not found".to_string());
        assert!(err.to_string().contains("Not found:"));
        assert!(err.to_string().contains("User not found"));
    }

    // Test From<reqwest::Error> for 503 status
    #[tokio::test]
    async fn test_reqwest_503_becomes_service_unavailable() {
        let mock_server = MockServer::start().await;

        // Mount a 503 response
        Mock::given(wiremock::matchers::method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        // Make a request that returns 503
        let client = reqwest::Client::new();
        let result = client
            .get(format!("{}/service-down", mock_server.uri()))
            .send()
            .await
            .and_then(|r| r.error_for_status());

        if let Err(reqwest_err) = result {
            let scraper_err: ScraperError = reqwest_err.into();

            match scraper_err {
                ScraperError::ServiceUnavailable => {
                    assert_eq!(
                        scraper_err.to_string(),
                        "Service temporarily unavailable (503)"
                    );
                }
                _ => panic!(
                    "Expected ServiceUnavailable variant, got: {:?}",
                    scraper_err
                ),
            }
        } else {
            panic!("Expected reqwest error");
        }
    }

    // Test From<reqwest::Error> for non-503 status
    #[tokio::test]
    async fn test_reqwest_non_503_becomes_http() {
        let mock_server = MockServer::start().await;

        // Mount a 404 response
        Mock::given(wiremock::matchers::method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        // Make a request that returns 404
        let client = reqwest::Client::new();
        let result = client
            .get(format!("{}/not-found", mock_server.uri()))
            .send()
            .await
            .and_then(|r| r.error_for_status());

        if let Err(reqwest_err) = result {
            let scraper_err: ScraperError = reqwest_err.into();

            match scraper_err {
                ScraperError::Http(_) => {
                    assert!(scraper_err.to_string().starts_with("HTTP error:"));
                }
                _ => panic!("Expected Http variant"),
            }
        } else {
            panic!("Expected reqwest error");
        }
    }

    // Test From<reqwest::Error> for 500 status
    #[tokio::test]
    async fn test_reqwest_500_becomes_http() {
        let mock_server = MockServer::start().await;

        // Mount a 500 response
        Mock::given(wiremock::matchers::method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        // Make a request that returns 500
        let client = reqwest::Client::new();
        let result = client
            .get(format!("{}/error", mock_server.uri()))
            .send()
            .await
            .and_then(|r| r.error_for_status());

        if let Err(reqwest_err) = result {
            let scraper_err: ScraperError = reqwest_err.into();

            match scraper_err {
                ScraperError::Http(_) => {
                    assert!(scraper_err.to_string().starts_with("HTTP error:"));
                }
                _ => panic!("Expected Http variant, got ServiceUnavailable"),
            }
        } else {
            panic!("Expected reqwest error");
        }
    }

    // Test From<sea_orm::DbErr> implementation
    #[test]
    fn test_database_error_from_conversion() {
        let db_err = DbErr::Custom("query failed".to_string());
        let scraper_err: ScraperError = db_err.into();

        match scraper_err {
            ScraperError::Database(_) => {
                assert!(scraper_err.to_string().contains("Database error:"));
                assert!(scraper_err.to_string().contains("query failed"));
            }
            _ => panic!("Expected Database variant"),
        }
    }

    // Test From<sea_orm::DbErr> with different error types
    #[test]
    fn test_database_error_conn_acquire() {
        let db_err = DbErr::ConnectionAcquire(sea_orm::ConnAcquireErr::Timeout);
        let scraper_err: ScraperError = db_err.into();

        match scraper_err {
            ScraperError::Database(_) => {
                assert!(scraper_err.to_string().contains("Database error:"));
            }
            _ => panic!("Expected Database variant"),
        }
    }

    // Test error chaining - From conversion followed by Result propagation
    #[tokio::test]
    async fn test_error_chaining_reqwest() {
        async fn simulate_http_call(uri: String) -> app::error::Result<String> {
            let client = reqwest::Client::new();
            let response = client.get(uri).send().await?.error_for_status()?;
            Ok(response.text().await?)
        }

        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .respond_with(ResponseTemplate::new(502))
            .mount(&mock_server)
            .await;

        let result = simulate_http_call(format!("{}/bad-gateway", mock_server.uri())).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            ScraperError::Http(_) => {
                assert!(err.to_string().starts_with("HTTP error:"));
            }
            _ => panic!("Expected Http variant"),
        }
    }

    // Test error chaining - From conversion with 503
    #[tokio::test]
    async fn test_error_chaining_service_unavailable() {
        async fn simulate_503(uri: String) -> app::error::Result<String> {
            let client = reqwest::Client::new();
            let response = client.get(uri).send().await?.error_for_status()?;
            Ok(response.text().await?)
        }

        let mock_server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let result = simulate_503(format!("{}/unavailable", mock_server.uri())).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            ScraperError::ServiceUnavailable => {
                assert_eq!(err.to_string(), "Service temporarily unavailable (503)");
            }
            _ => panic!("Expected ServiceUnavailable variant"),
        }
    }

    // Test error chaining - Database errors
    #[test]
    fn test_error_chaining_database() {
        fn simulate_db_query() -> app::error::Result<String> {
            let db_err = DbErr::RecordNotFound("user not found".to_string());
            Err(db_err.into())
        }

        let result = simulate_db_query();
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            ScraperError::Database(_) => {
                assert!(err.to_string().contains("Database error:"));
                assert!(err.to_string().contains("user not found"));
            }
            _ => panic!("Expected Database variant"),
        }
    }
}

// Test error propagation with ? operator
#[test]
fn test_error_propagation_parse() {
    fn parse_operation() -> app::error::Result<i32> {
        Err(ScraperError::Parse("bad format".to_string()))
    }

    fn caller() -> app::error::Result<i32> {
        let _value = parse_operation()?;
        Ok(42)
    }

    let result = caller();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "XML parse error: bad format"
    );
}

// Test error propagation with multiple levels
#[test]
fn test_nested_error_propagation() {
    fn level_1() -> app::error::Result<()> {
        Err(ScraperError::Config("invalid setting".to_string()))
    }

    fn level_2() -> app::error::Result<()> {
        level_1()?;
        Ok(())
    }

    fn level_3() -> app::error::Result<()> {
        level_2()?;
        Ok(())
    }

    let result = level_3();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Configuration error: invalid setting"
    );
}

// Test that ScraperError implements std::error::Error
#[test]
fn test_implements_std_error() {
    fn is_std_error<T: std::error::Error>(_: &T) {}

    let err = ScraperError::Parse("test".to_string());
    is_std_error(&err);
}

// Test Debug implementation
#[test]
fn test_debug_format() {
    let err = ScraperError::Parse("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("Parse"));
    assert!(debug_str.contains("test"));
}

// Test Result type alias
#[test]
fn test_result_type_alias() {
    fn returns_result() -> app::error::Result<String> {
        Ok("success".to_string())
    }

    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

// Test Result type alias with error
#[test]
fn test_result_type_alias_error() {
    fn returns_error() -> app::error::Result<String> {
        Err(ScraperError::InvalidResponse("bad data".to_string()))
    }

    let result = returns_error();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid response: bad data"
    );
}
