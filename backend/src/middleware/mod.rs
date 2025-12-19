use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

pub mod tenant;

// Tenant context that gets added to request extensions
#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: uuid::Uuid,
    pub schema_name: String,
}

pub async fn tenant_resolver(
    State(pool): State<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, crate::models::AppError> {
    // Extract tenant from header (in production, this would come from JWT or session)
    let tenant_header = req
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| crate::models::AppError::BadRequest("Missing x-tenant-id header".to_string()))?;

    // Parse tenant ID
    let tenant_id = uuid::Uuid::parse_str(tenant_header)
        .map_err(|_| crate::models::AppError::BadRequest("Invalid tenant ID format".to_string()))?;

    // Get tenant from database
    let tenant = crate::db::tenants::get_tenant_by_id(&pool, tenant_id)
        .await?
        .ok_or_else(|| crate::models::AppError::NotFound("Tenant not found".to_string()))?;

    // Add tenant context to request extensions
    let context = TenantContext {
        tenant_id: tenant.id,
        schema_name: tenant.schema_name,
    };
    
    req.extensions_mut().insert(context);

    Ok(next.run(req).await)
}
