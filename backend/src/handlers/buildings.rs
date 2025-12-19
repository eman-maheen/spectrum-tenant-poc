use axum::{extract::{Path, State}, Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{db, middleware::TenantContext, models::{requests::*, responses::*, ApiResponse, AppResult}};

pub async fn list_buildings(
    tenant: TenantContext,
    State(pool): State<PgPool>,
) -> AppResult<Json<ApiResponse<Vec<BuildingResponse>>>> {
    let buildings = db::buildings::list_buildings(&pool, &tenant.schema_name).await?;
    Ok(Json(ApiResponse::success(buildings)))
}

pub async fn get_building(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<BuildingResponse>>> {
    let building = db::buildings::get_building(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(building)))
}

pub async fn create_building(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateBuildingRequest>,
) -> AppResult<Json<ApiResponse<BuildingResponse>>> {
    let building = db::buildings::create_building(&pool, &tenant.schema_name, req).await?;
    Ok(Json(ApiResponse::success(building)))
}

pub async fn update_building(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBuildingRequest>,
) -> AppResult<Json<ApiResponse<BuildingResponse>>> {
    let building = db::buildings::update_building(&pool, &tenant.schema_name, id, req).await?;
    Ok(Json(ApiResponse::success(building)))
}

pub async fn delete_building(
    tenant: TenantContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    db::buildings::delete_building(&pool, &tenant.schema_name, id).await?;
    Ok(Json(ApiResponse::success(())))
}
