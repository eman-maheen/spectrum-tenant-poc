use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{requests::*, responses::*, AppError, AppResult};

pub async fn list_buildings(pool: &PgPool, schema_name: &str) -> AppResult<Vec<BuildingResponse>> {
    let query = format!(
        "SELECT id, property_id, name, floors, year_built, created_at, updated_at, version 
         FROM {}.buildings 
         WHERE is_deleted = false 
         ORDER BY created_at DESC",
        schema_name
    );

    let buildings = sqlx::query_as::<_, BuildingResponse>(&query)
        .fetch_all(pool)
        .await?;

    Ok(buildings)
}

pub async fn get_building(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<BuildingResponse> {
    let query = format!(
        "SELECT id, property_id, name, floors, year_built, created_at, updated_at, version 
         FROM {}.buildings 
         WHERE id = $1 AND is_deleted = false",
        schema_name
    );

    let building = sqlx::query_as::<_, BuildingResponse>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Building not found".to_string()))?;

    Ok(building)
}

pub async fn create_building(
    pool: &PgPool,
    schema_name: &str,
    req: CreateBuildingRequest,
) -> AppResult<BuildingResponse> {
    let query = format!(
        "INSERT INTO {}.buildings (property_id, name, floors, year_built) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, property_id, name, floors, year_built, created_at, updated_at, version",
        schema_name
    );

    let building = sqlx::query_as::<_, BuildingResponse>(&query)
        .bind(&req.property_id)
        .bind(&req.name)
        .bind(&req.floors)
        .bind(&req.year_built)
        .fetch_one(pool)
        .await?;

    Ok(building)
}

pub async fn update_building(
    pool: &PgPool,
    schema_name: &str,
    id: Uuid,
    req: UpdateBuildingRequest,
) -> AppResult<BuildingResponse> {
    let _ = get_building(pool, schema_name, id).await?;

    let query = format!(
        "UPDATE {}.buildings 
         SET name = COALESCE($1, name),
             floors = COALESCE($2, floors),
             year_built = COALESCE($3, year_built),
             version = version + 1
         WHERE id = $4 AND is_deleted = false
         RETURNING id, property_id, name, floors, year_built, created_at, updated_at, version",
        schema_name
    );

    let building = sqlx::query_as::<_, BuildingResponse>(&query)
        .bind(&req.name)
        .bind(&req.floors)
        .bind(&req.year_built)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(building)
}

pub async fn delete_building(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<()> {
    let query = format!(
        "UPDATE {}.buildings 
         SET is_deleted = true, version = version + 1 
         WHERE id = $1",
        schema_name
    );

    let result = sqlx::query(&query)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Building not found".to_string()));
    }

    Ok(())
}
