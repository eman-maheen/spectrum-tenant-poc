# Phase 1: Repository & Project Setup - Step by Step

## Prerequisites Check

Before starting, ensure you have:
```bash
# Check Rust (need 1.75+)
rustc --version
cargo --version

# Check Node.js (need 18+)
node --version
npm --version

# Install if missing:
# Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Node: https://nodejs.org/
```

---

## Step 1: Initialize Git Repository

```bash
# Create project directory
mkdir spectrum-tenant-poc
cd spectrum-tenant-poc

# Initialize git
git init

# Create initial .gitignore
cat > .gitignore << 'EOF'
# Rust
target/
Cargo.lock
**/*.rs.bk
*.pdb

# Node
node_modules/
dist/
build/
.svelte-kit/

# Environment
.env
.env.local
.env.*.local

# Database
*.db
*.db-shm
*.db-wal

# IDE
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store

# Logs
*.log
npm-debug.log*

# OS
Thumbs.db
EOF

git add .gitignore
git commit -m "Initial commit: Add .gitignore"
```

---

## Step 2: Create Workspace Structure

```bash
# Create all directories at once
mkdir -p backend/src
mkdir -p desktop/src
mkdir -p shared/src
mkdir -p migrations
mkdir -p docs

# Verify structure
tree -L 2
# Should show:
# .
# ├── backend/
# │   └── src/
# ├── desktop/
# │   └── src/
# ├── shared/
# │   └── src/
# ├── migrations/
# └── docs/
```

---

## Step 3: Create Root Cargo Workspace

```bash
# Create workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = ["backend", "desktop", "shared"]
resolver = "2"

[workspace.dependencies]
# Shared dependencies with locked versions
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"

[profile.dev]
opt-level = 1

[profile.release]
lto = true
codegen-units = 1
strip = true
EOF

git add Cargo.toml
git commit -m "Add workspace Cargo.toml"
```

---

## Step 4: Create Shared Library

This will contain types used by both backend and desktop.

```bash
# Create shared/Cargo.toml
cat > shared/Cargo.toml << 'EOF'
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
EOF

# Create shared/src/lib.rs with basic types
cat > shared/src/lib.rs << 'EOF'
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod models;
pub mod sync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: Uuid,
    pub property_id: Uuid,
    pub name: String,
    pub floors: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub property_id: Uuid,
    pub report_type: String,
    pub pdf_url: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub version: i64,
}
EOF

# Create placeholder modules
mkdir -p shared/src/models shared/src/sync

cat > shared/src/models/mod.rs << 'EOF'
// Re-export main types
pub use crate::{Client, Property, Building, Unit, Report};
EOF

cat > shared/src/sync/mod.rs << 'EOF'
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub id: Uuid,
    pub table_name: String,
    pub operation: Operation,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub tenant_id: String,
    pub client_id: Uuid,
    pub last_version: i64,
    pub operations: Vec<SyncOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub operations: Vec<SyncOperation>,
    pub current_version: i64,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub operation_id: Uuid,
    pub reason: String,
    pub server_version: serde_json::Value,
}
EOF

git add shared/
git commit -m "Add shared library with models and sync types"
```

---

## Step 5: Create Backend Project

```bash
# Create backend/Cargo.toml
cat > backend/Cargo.toml << 'EOF'
[package]
name = "backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Workspace dependencies
shared = { path = "../shared" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }

# Backend-specific
axum = { version = "0.7", features = ["macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "trace"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio", "chrono", "uuid", "json"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
EOF

# Create backend source structure
mkdir -p backend/src/{db,handlers,middleware,models,routes}

# Create backend/src/main.rs
cat > backend/src/main.rs << 'EOF'
use axum::{
    Router,
    routing::{get, post},
};
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
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("Starting Spectrum Tenant POC Backend");

    // TODO: Initialize database pool
    
    // Build application routes
    let app = Router::new()
        .route("/", get(|| async { "Spectrum Tenant POC API" }))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
EOF

# Create placeholder modules
cat > backend/src/db/mod.rs << 'EOF'
// Database connection and operations
EOF

cat > backend/src/handlers/mod.rs << 'EOF'
// HTTP request handlers
EOF

cat > backend/src/middleware/mod.rs << 'EOF'
// Middleware (tenant resolution, auth, etc.)
EOF

cat > backend/src/models/mod.rs << 'EOF'
// Backend-specific models
EOF

cat > backend/src/routes/mod.rs << 'EOF'
// Route definitions
EOF

# Create backend .env.example
cat > backend/.env.example << 'EOF'
DATABASE_URL=postgresql://postgres:password@localhost:5432/spectrum_poc
RUST_LOG=backend=debug,tower_http=debug
SERVER_PORT=3000
EOF

cp backend/.env.example backend/.env

git add backend/
git commit -m "Add backend Axum project structure"
```

---

## Step 6: Create Desktop (Tauri) Project

```bash
# Create desktop/Cargo.toml
cat > desktop/Cargo.toml << 'EOF'
[package]
name = "desktop"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
# Workspace dependencies
shared = { path = "../shared" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
anyhow = { workspace = true }

# Desktop-specific
tauri = { version = "2", features = [] }
rusqlite = { version = "0.31", features = ["bundled", "chrono", "uuid"] }
reqwest = { version = "0.11", features = ["json"] }
EOF

# Create desktop/src/main.rs
cat > desktop/src/main.rs << 'EOF'
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod db;
mod sync;
mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize local database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF

# Create desktop/src/commands.rs
cat > desktop/src/commands.rs << 'EOF'
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Spectrum Tenant POC.", name)
}
EOF

# Create placeholder modules
cat > desktop/src/db/mod.rs << 'EOF'
use tauri::AppHandle;
use anyhow::Result;

pub async fn init_db(app: &AppHandle) -> Result<()> {
    // TODO: Initialize SQLite database
    println!("Database initialized");
    Ok(())
}
EOF

cat > desktop/src/sync/mod.rs << 'EOF'
// Sync logic
EOF

# Create desktop/build.rs
cat > desktop/build.rs << 'EOF'
fn main() {
    tauri_build::build()
}
EOF

# Create Tauri configuration
cat > desktop/tauri.conf.json << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Spectrum Tenant POC",
  "version": "0.1.0",
  "identifier": "com.spectrum.tenant.poc",
  "build": {
    "frontendDist": "../frontend/build",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "windows": [
      {
        "title": "Spectrum Tenant POC",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
EOF

# Create icons directory (you'll need to add actual icons later)
mkdir -p desktop/icons
touch desktop/icons/.gitkeep

git add desktop/
git commit -m "Add Tauri desktop project structure"
```

---

## Step 7: Create SvelteKit Frontend

```bash
# Initialize SvelteKit (this will prompt for options)
npm create svelte@latest frontend

# When prompted, choose:
# - Skeleton project
# - Yes, using TypeScript syntax
# - Add ESLint, Prettier
# - No Playwright, Vitest (add later if needed)

cd frontend

# Install dependencies
npm install

# Install Tauri API
npm install @tauri-apps/api @tauri-apps/plugin-shell

# Update svelte.config.js for Tauri
cat > svelte.config.js << 'EOF'
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    })
  }
};

export default config;
EOF

# Install static adapter
npm install -D @sveltejs/adapter-static

# Update vite.config.ts
cat > vite.config.ts << 'EOF'
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  
  // Tauri expects a fixed port for development
  server: {
    port: 5173,
    strictPort: true,
  },
  
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  
  // Tauri expects environment variables to be prefixed with VITE_
  envPrefix: ['VITE_', 'TAURI_'],
});
EOF

cd ..

git add frontend/
git commit -m "Add SvelteKit frontend with Tauri configuration"
```

---

## Step 8: Create Documentation Structure

```bash
# Create README.md
cat > README.md << 'EOF'
# Spectrum Tenant POC

Offline-first multi-tenant system with bidirectional sync.

## Architecture

- **Backend**: Axum + PostgreSQL (schema-based tenancy)
- **Desktop**: Tauri + SvelteKit + SQLite
- **Sync**: PowerSync / Custom CRDT

## Project Structure

```
spectrum-tenant-poc/
├── backend/          # Axum REST API
├── desktop/          # Tauri application
├── frontend/         # SvelteKit UI
├── shared/           # Shared Rust types
├── migrations/       # Database migrations
└── docs/             # Documentation
```

## Getting Started

See [docs/SETUP.md](docs/SETUP.md) for detailed setup instructions.

## Development

### Backend
```bash
cd backend
cargo run
```

### Desktop + Frontend
```bash
cd desktop
cargo tauri dev
```

## License

MIT
EOF

# Create setup documentation
cat > docs/SETUP.md << 'EOF'
# Setup Guide

## Prerequisites

- Rust 1.75+
- Node.js 18+
- PostgreSQL 16+

## Database Setup

```bash
createdb spectrum_poc
psql spectrum_poc < migrations/001_initial.sql
```

## Backend Setup

```bash
cd backend
cp .env.example .env
# Edit .env with your database credentials
cargo run
```

## Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

## Desktop Setup

```bash
cd desktop
cargo tauri dev
```
EOF

git add README.md docs/
git commit -m "Add documentation structure"
```

---

## Step 9: Create Development Scripts

```bash
# Create root package.json for convenience scripts
cat > package.json << 'EOF'
{
  "name": "spectrum-tenant-poc",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev:backend": "cd backend && cargo run",
    "dev:desktop": "cd desktop && cargo tauri dev",
    "dev:frontend": "cd frontend && npm run dev",
    "build:all": "cargo build --release && cd frontend && npm run build",
    "test:all": "cargo test --all"
  }
}
EOF

# Create Makefile for common tasks
cat > Makefile << 'EOF'
.PHONY: help dev-backend dev-desktop clean test

help:
	@echo "Available commands:"
	@echo "  make dev-backend   - Run backend server"
	@echo "  make dev-desktop   - Run desktop app"
	@echo "  make clean         - Clean all build artifacts"
	@echo "  make test          - Run all tests"

dev-backend:
	cd backend && cargo run

dev-desktop:
	cd desktop && cargo tauri dev

clean:
	cargo clean
	cd frontend && rm -rf node_modules build .svelte-kit

test:
	cargo test --all
EOF

git add package.json Makefile
git commit -m "Add development scripts"
```

---

## Step 10: Verify Setup

```bash
# Test backend compilation
cd backend
cargo check
cd ..

# Test desktop compilation
cd desktop
cargo check
cd ..

# Test frontend
cd frontend
npm run check
cd ..

# Test shared library
cd shared
cargo test
cd ..

# Verify workspace
cargo build --all
```

---

## Step 11: Create Initial Tag and Push

```bash
# Create initial tag
git tag -a v0.1.0-setup -m "Phase 1: Project setup complete"

# View commit history
git log --oneline --graph

# If pushing to remote (GitHub/GitLab)
# git remote add origin <your-repo-url>
# git push -u origin main
# git push --tags
```

---

## Final Verification Checklist

- [ ] All directories created
- [ ] Workspace compiles: `cargo build --all`
- [ ] Backend runs: `cd backend && cargo run`
- [ ] Frontend builds: `cd frontend && npm run build`
- [ ] Shared types accessible from both backend and desktop
- [ ] Git history clean with logical commits
- [ ] Documentation files present

---

## Troubleshooting

### Issue: Cargo version conflicts
```bash
cargo update
cargo clean
cargo build --all
```

### Issue: Frontend fails to build
```bash
cd frontend
rm -rf node_modules package-lock.json
npm install
```

### Issue: Tauri dev command not found
```bash
cargo install tauri-cli --version "^2.0.0"
```

---

## Next Steps

After completing Phase 1:
1. Move to Phase 2: Database Setup
2. Create PostgreSQL migrations
3. Set up SQLite schema for desktop
4. Implement basic CRUD operations

Your foundation is now solid and ready for building! 🚀