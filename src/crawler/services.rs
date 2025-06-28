use crate::pg::conn::establish_connection;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppServices {
    pub http: Arc<reqwest::Client>,
    pub db: Arc<sqlx::PgPool>,
}

impl AppServices {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            http: Arc::new(reqwest::Client::new()),
            db: Arc::new(establish_connection().await?),
        })
    }
}
