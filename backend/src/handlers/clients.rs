use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    db,
    middleware::TenantContext,
    models::{requests::*, responses::*, ApiResponse, AppResult},
};

pub async fn list_clients(
    tenant: TenantContext,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<Vec<ClientResponse>>>> {
    let clients = db::clients::list_clients(&pool, &tenant.schema_name).await?;
    Ok(Json(ApiResponse::success(clients)))
}

pub async fn get_client(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<ClientResponse>>> {
    let client = db::clients::get_client(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(client)))
}

pub async fn create_client(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateClientRequest>,
) -> AppResult<Json<ApiResponse<ClientResponse>>> {
    let client = db::clients::create_client(&pool, &tenant.schema_name, req).await?;
    Ok(Json(ApiResponse::success(client)))
}

pub async fn update_client(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateClientRequest>,
) -> AppResult<Json<ApiResponse<ClientResponse>>> {
    let client = db::clients::update_client(&pool, &tenant.schema_name, id, req).await?;
    Ok(Json(ApiResponse::success(client)))
}

pub async fn delete_client(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    db::clients::delete_client(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(())))
}
