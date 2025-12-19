use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{requests::*, responses::*, AppError, AppResult};

pub async fn list_properties(pool: &PgPool, schema_name: &str) -> AppResult<Vec<PropertyResponse>> {
    let query = format!(
        "SELECT id, client_id, name, address, city, state, zip_code, created_at, updated_at, version 
         FROM {}.properties 
         WHERE is_deleted = false 
         ORDER BY created_at DESC",
        schema_name
    );

    let properties = sqlx::query_as::<_, PropertyResponse>(&query)
        .fetch_all(pool)
        .await?;

    Ok(properties)
}

pub async fn get_property(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<PropertyResponse> {
    let query = format!(
        "SELECT id, client_id, name, address, city, state, zip_code, created_at, updated_at, version 
         FROM {}.properties 
         WHERE id = $1 AND is_deleted = false",
        schema_name
    );

    let property = sqlx::query_as::<_, PropertyResponse>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Property not found".to_string()))?;

    Ok(property)
}

pub async fn create_property(
    pool: &PgPool,
    schema_name: &str,
    req: CreatePropertyRequest,
) -> AppResult<PropertyResponse> {
    let query = format!(
        "INSERT INTO {}.properties (client_id, name, address, city, state, zip_code) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, client_id, name, address, city, state, zip_code, created_at, updated_at, version",
        schema_name
    );

    let property = sqlx::query_as::<_, PropertyResponse>(&query)
        .bind(&req.client_id)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.city)
        .bind(&req.state)
        .bind(&req.zip_code)
        .fetch_one(pool)
        .await?;

    Ok(property)
}

pub async fn update_property(
    pool: &PgPool,
    schema_name: &str,
    id: Uuid,
    req: UpdatePropertyRequest,
) -> AppResult<PropertyResponse> {
    let _ = get_property(pool, schema_name, id).await?;

    let query = format!(
        "UPDATE {}.properties 
         SET name = COALESCE($1, name),
             address = COALESCE($2, address),
             city = COALESCE($3, city),
             state = COALESCE($4, state),
             zip_code = COALESCE($5, zip_code),
             version = version + 1
         WHERE id = $6 AND is_deleted = false
         RETURNING id, client_id, name, address, city, state, zip_code, created_at, updated_at, version",
        schema_name
    );

    let property = sqlx::query_as::<_, PropertyResponse>(&query)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.city)
        .bind(&req.state)
        .bind(&req.zip_code)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(property)
}

pub async fn delete_property(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<()> {
    let query = format!(
        "UPDATE {}.properties 
         SET is_deleted = true, version = version + 1 
         WHERE id = $1",
        schema_name
    );

    let result = sqlx::query(&query)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Property not found".to_string()));
    }

    Ok(())
}
