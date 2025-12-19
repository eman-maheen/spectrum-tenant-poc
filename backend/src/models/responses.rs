use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct PropertyResponse {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct BuildingResponse {
    pub id: Uuid,
    pub property_id: Uuid,
    pub name: String,
    pub floors: i32,
    pub year_built: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct UnitResponse {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub square_feet: Option<i32>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}
