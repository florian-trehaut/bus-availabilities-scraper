use crate::error::{Result, ScraperError};
use sea_orm::{Database, DatabaseConnection};

pub async fn init_database(database_url: &str) -> Result<DatabaseConnection> {
    Database::connect(database_url)
        .await
        .map_err(|e| ScraperError::Config(format!("Failed to connect to database: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_database() {
        let db = init_database("sqlite::memory:").await;
        assert!(db.is_ok());
    }
}
