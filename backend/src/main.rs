use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handlers;
mod middleware;
mod models;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    tracing::info!("Starting Spectrum Tenant POC Backend");
    tracing::info!("Connecting to database...");

    // Create database pool
    let pool = db::create_pool(&database_url).await?;
    
    tracing::info!("Database connected successfully");

    // Build application with routes
    let app = Router::new()
        .merge(routes::create_routes(pool))
        .layer(CorsLayer::permissive());

    // Start server
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");
        
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Listening on {}", addr);
    tracing::info!("
===========================================
API Endpoints:
  GET    /health
  
  GET    /api/clients
  GET    /api/clients/:id
  POST   /api/clients
  PUT    /api/clients/:id
  DELETE /api/clients/:id
  
  GET    /api/properties
  GET    /api/properties/:id
  POST   /api/properties
  PUT    /api/properties/:id
  DELETE /api/properties/:id
  
  GET    /api/buildings
  GET    /api/buildings/:id
  POST   /api/buildings
  PUT    /api/buildings/:id
  DELETE /api/buildings/:id
  
  GET    /api/units
  GET    /api/units/:id
  POST   /api/units
  PUT    /api/units/:id
  DELETE /api/units/:id
===========================================
");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
