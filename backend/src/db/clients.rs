use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{requests::*, responses::*, AppError, AppResult};

pub async fn list_clients(pool: &PgPool, schema_name: &str) -> AppResult<Vec<ClientResponse>> {
    let query = format!(
        "SELECT id, name, email, phone, created_at, updated_at, version 
         FROM {}.clients 
         WHERE is_deleted = false 
         ORDER BY created_at DESC",
        schema_name
    );

    let clients = sqlx::query_as::<_, ClientResponse>(&query)
        .fetch_all(pool)
        .await?;

    Ok(clients)
}

pub async fn get_client(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<ClientResponse> {
    let query = format!(
        "SELECT id, name, email, phone, created_at, updated_at, version 
         FROM {}.clients 
         WHERE id = $1 AND is_deleted = false",
        schema_name
    );

    let client = sqlx::query_as::<_, ClientResponse>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

    Ok(client)
}

pub async fn create_client(
    pool: &PgPool,
    schema_name: &str,
    req: CreateClientRequest,
) -> AppResult<ClientResponse> {
    let query = format!(
        "INSERT INTO {}.clients (name, email, phone) 
         VALUES ($1, $2, $3) 
         RETURNING id, name, email, phone, created_at, updated_at, version",
        schema_name
    );

    let client = sqlx::query_as::<_, ClientResponse>(&query)
        .bind(&req.name)
        .bind(&req.email)
        .bind(&req.phone)
        .fetch_one(pool)
        .await?;

    Ok(client)
}

pub async fn update_client(
    pool: &PgPool,
    schema_name: &str,
    id: Uuid,
    req: UpdateClientRequest,
) -> AppResult<ClientResponse> {
    // First check if client exists
    let _ = get_client(pool, schema_name, id).await?;

    let query = format!(
        "UPDATE {}.clients 
         SET name = COALESCE($1, name),
             email = COALESCE($2, email),
             phone = COALESCE($3, phone),
             version = version + 1
         WHERE id = $4 AND is_deleted = false
         RETURNING id, name, email, phone, created_at, updated_at, version",
        schema_name
    );

    let client = sqlx::query_as::<_, ClientResponse>(&query)
        .bind(&req.name)
        .bind(&req.email)
        .bind(&req.phone)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(client)
}

pub async fn delete_client(pool: &PgPool, schema_name: &str, id: Uuid) -> AppResult<()> {
    let query = format!(
        "UPDATE {}.clients 
         SET is_deleted = true, version = version + 1 
         WHERE id = $1",
        schema_name
    );

    let result = sqlx::query(&query)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Client not found".to_string()));
    }

    Ok(())
}
