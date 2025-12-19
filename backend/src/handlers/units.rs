use axum::{extract::{Path, State}, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{db, middleware::TenantContext, models::{requests::*, responses::*, ApiResponse, AppResult}};

pub async fn list_units(
    tenant: TenantContext,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<Vec<UnitResponse>>>> {
    let units = db::units::list_units(&pool, &tenant.schema_name).await?;
    Ok(Json(ApiResponse::success(units)))
}

pub async fn get_unit(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    let unit = db::units::get_unit(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(unit)))
}

pub async fn create_unit(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    let unit = db::units::create_unit(&pool, &tenant.schema_name, req).await?;
    Ok(Json(ApiResponse::success(unit)))
}

pub async fn update_unit(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    let unit = db::units::update_unit(&pool, &tenant.schema_name, id, req).await?;
    Ok(Json(ApiResponse::success(unit)))
}

pub async fn delete_unit(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    db::units::delete_unit(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(())))
}
