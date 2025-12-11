use std::fmt;

#[derive(Debug)]
pub enum ScraperError {
    #[cfg(feature = "ssr")]
    Http(reqwest::Error),
    Parse(String),
    Config(String),
    #[cfg(feature = "ssr")]
    Database(sea_orm::DbErr),
    ServiceUnavailable,
    InvalidResponse(String),
}

impl fmt::Display for ScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "ssr")]
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Parse(e) => write!(f, "XML parse error: {e}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            #[cfg(feature = "ssr")]
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::ServiceUnavailable => write!(f, "Service temporarily unavailable (503)"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
        }
    }
}

impl std::error::Error for ScraperError {}

#[cfg(feature = "ssr")]
impl From<reqwest::Error> for ScraperError {
    fn from(e: reqwest::Error) -> Self {
        if e.status() == Some(reqwest::StatusCode::SERVICE_UNAVAILABLE) {
            Self::ServiceUnavailable
        } else {
            Self::Http(e)
        }
    }
}

#[cfg(feature = "ssr")]
impl From<sea_orm::DbErr> for ScraperError {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::Database(e)
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
