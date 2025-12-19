use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

pub mod tenants;
pub mod clients;
pub mod properties;
pub mod buildings;
pub mod units;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
