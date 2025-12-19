use axum::{extract::State, Json};
use sqlx::PgPool;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

pub async fn health_check(State(pool): State<PgPool>) -> Json<HealthResponse> {
    let db_status = match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
    })
}
