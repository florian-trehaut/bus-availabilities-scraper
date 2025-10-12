use std::fmt;

#[derive(Debug)]
pub enum ScraperError {
    Http(reqwest::Error),
    Parse(String),
    Config(String),
    ServiceUnavailable,
    InvalidResponse(String),
}

impl fmt::Display for ScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::Parse(e) => write!(f, "XML parse error: {}", e),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::ServiceUnavailable => write!(f, "Service temporarily unavailable (503)"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl std::error::Error for ScraperError {}

impl From<reqwest::Error> for ScraperError {
    fn from(e: reqwest::Error) -> Self {
        if e.status() == Some(reqwest::StatusCode::SERVICE_UNAVAILABLE) {
            Self::ServiceUnavailable
        } else {
            Self::Http(e)
        }
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
