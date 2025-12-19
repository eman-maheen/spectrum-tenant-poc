use axum::{extract::{Path, State}, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{db, middleware::TenantContext, models::{requests::*, responses::*, ApiResponse, AppResult}};

pub async fn list_properties(
    tenant: TenantContext,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<Vec<PropertyResponse>>>> {
    let properties = db::properties::list_properties(&pool, &tenant.schema_name).await?;
    Ok(Json(ApiResponse::success(properties)))
}

pub async fn get_property(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<PropertyResponse>>> {
    let property = db::properties::get_property(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(property)))
}

pub async fn create_property(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreatePropertyRequest>,
) -> AppResult<Json<ApiResponse<PropertyResponse>>> {
    let property = db::properties::create_property(&pool, &tenant.schema_name, req).await?;
    Ok(Json(ApiResponse::success(property)))
}

pub async fn update_property(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePropertyRequest>,
) -> AppResult<Json<ApiResponse<PropertyResponse>>> {
    let property = db::properties::update_property(&pool, &tenant.schema_name, id, req).await?;
    Ok(Json(ApiResponse::success(property)))
}

pub async fn delete_property(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    db::properties::delete_property(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(())))
}
