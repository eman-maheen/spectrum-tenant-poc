use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use crate::handlers;

pub fn create_routes(pool: PgPool) -> Router {
    let protected_routes = Router::new()
        // Clients routes
        .route("/clients", get(handlers::clients::list_clients))
        .route("/clients/:id", get(handlers::clients::get_client))
        .route("/clients", post(handlers::clients::create_client))
        .route("/clients/:id", put(handlers::clients::update_client))
        .route("/clients/:id", delete(handlers::clients::delete_client))
        
        // Properties routes
        .route("/properties", get(handlers::properties::list_properties))
        .route("/properties/:id", get(handlers::properties::get_property))
        .route("/properties", post(handlers::properties::create_property))
        .route("/properties/:id", put(handlers::properties::update_property))
        .route("/properties/:id", delete(handlers::properties::delete_property))
        
        // Buildings routes
        .route("/buildings", get(handlers::buildings::list_buildings))
        .route("/buildings/:id", get(handlers::buildings::get_building))
        .route("/buildings", post(handlers::buildings::create_building))
        .route("/buildings/:id", put(handlers::buildings::update_building))
        .route("/buildings/:id", delete(handlers::buildings::delete_building))
        
        // Units routes
        .route("/units", get(handlers::units::list_units))
        .route("/units/:id", get(handlers::units::get_unit))
        .route("/units", post(handlers::units::create_unit))
        .route("/units/:id", put(handlers::units::update_unit))
        .route("/units/:id", delete(handlers::units::delete_unit))
        
        // Apply tenant middleware to all protected routes
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            crate::middleware::tenant_resolver,
        ));

    Router::new()
        .route("/health", get(handlers::health::health_check))
        .nest("/api", protected_routes)
        .with_state(pool)
}
