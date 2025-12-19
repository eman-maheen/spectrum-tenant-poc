use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{requests::*, responses::*, AppError, AppResult};

pub async fn list_units(pool: &PgPool, schema_name: &str) -> AppResult<Vec<UnitResponse>> {
    let query = format!(
        "SELECT id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms, created_at, updated_at, version 
         FROM {}.units 
         WHERE is_deleted = false 
         ORDER BY unit_number",
        schema_name
    );

    let units = sqlx::query_as::<_, UnitResponse>(&query)
        .fetch_all(pool)
        .await?;

    Ok(units)
}

pub async fn get_unit(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<UnitResponse> {
    let query = format!(
        "SELECT id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms, created_at, updated_at, version 
         FROM {}.units 
         WHERE id = $1 AND is_deleted = false",
        schema_name
    );

    let unit = sqlx::query_as::<_, UnitResponse>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Unit not found".to_string()))?;

    Ok(unit)
}

pub async fn create_unit(
    pool: &PgPool,
    schema_name: &str,
    req: CreateUnitRequest,
) -> AppResult<UnitResponse> {
    let query = format!(
        "INSERT INTO {}.units (building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms, created_at, updated_at, version",
        schema_name
    );

    let unit = sqlx::query_as::<_, UnitResponse>(&query)
        .bind(&req.building_id)
        .bind(&req.unit_number)
        .bind(&req.unit_type)
        .bind(&req.square_feet)
        .bind(&req.bedrooms)
        .bind(&req.bathrooms)
        .fetch_one(pool)
        .await?;

    Ok(unit)
}

pub async fn update_unit(
    pool: &PgPool,
    schema_name: &str,
    id: Uuid,
    req: UpdateUnitRequest,
) -> AppResult<UnitResponse> {
    let _ = get_unit(pool, schema_name, id).await?;

    let query = format!(
        "UPDATE {}.units 
         SET unit_number = COALESCE($1, unit_number),
             unit_type = COALESCE($2, unit_type),
             square_feet = COALESCE($3, square_feet),
             bedrooms = COALESCE($4, bedrooms),
             bathrooms = COALESCE($5, bathrooms),
             version = version + 1
         WHERE id = $6 AND is_deleted = false
         RETURNING id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms, created_at, updated_at, version",
        schema_name
    );

    let unit = sqlx::query_as::<_, UnitResponse>(&query)
        .bind(&req.unit_number)
        .bind(&req.unit_type)
        .bind(&req.square_feet)
        .bind(&req.bedrooms)
        .bind(&req.bathrooms)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(unit)
}

pub async fn delete_unit(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<()> {
    let query = format!(
        "UPDATE {}.units 
         SET is_deleted = true, version = version + 1 
         WHERE id = $1",
        schema_name
    );

    let result = sqlx::query(&query)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Unit not found".to_string()));
    }

    Ok(())
}
