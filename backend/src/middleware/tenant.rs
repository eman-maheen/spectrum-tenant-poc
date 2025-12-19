use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use super::TenantContext;
use crate::models::AppError;

// Extractor for tenant context
#[axum::async_trait]
impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TenantContext>()
            .cloned()
            .ok_or_else(|| AppError::InternalError("Tenant context not found".to_string()))
    }
}
