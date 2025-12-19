use serde::Deserialize;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePropertyRequest {
    pub client_id: Uuid,
    pub name: String,
    pub address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePropertyRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBuildingRequest {
    pub property_id: Uuid,
    pub name: String,
    pub floors: i32,
    pub year_built: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBuildingRequest {
    pub name: Option<String>,
    pub floors: Option<i32>,
    pub year_built: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUnitRequest {
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub square_feet: Option<i32>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<Decimal>,  // Changed from f32
}

#[derive(Debug, Deserialize)]
pub struct UpdateUnitRequest {
    pub unit_number: Option<String>,
    pub unit_type: Option<String>,
    pub square_feet: Option<i32>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<Decimal>,  // Changed from f32
}
