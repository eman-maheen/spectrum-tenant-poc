# Phase 2: Database Setup - Step by Step

## Prerequisites

Ensure PostgreSQL 16+ is installed and running:

```bash
# Check PostgreSQL version
psql --version

# Check if PostgreSQL is running
sudo systemctl status postgresql

# If not running, start it
sudo systemctl start postgresql

# For Arch/CachyOS
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
```

---

## Step 1: Install SQLx CLI

SQLx CLI manages migrations and compile-time query verification.

```bash
# Install sqlx-cli with only postgres support
cargo install sqlx-cli --no-default-features --features postgres

# Verify installation
sqlx --version
```

---

## Step 2: Create PostgreSQL Database

```bash
# Switch to postgres user and create database
sudo -u postgres psql << 'EOF'
CREATE DATABASE spectrum_poc;
CREATE USER spectrum_user WITH PASSWORD 'spectrum_pass_123';
GRANT ALL PRIVILEGES ON DATABASE spectrum_poc TO spectrum_user;
\q
EOF

# Test connection
psql -U spectrum_user -d spectrum_poc -h localhost
# Type your password: spectrum_pass_123
# Then: \q to quit
```

**Note**: For production, use stronger passwords and proper authentication!

---

## Step 3: Configure Backend Database Connection

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/backend

# Update .env file
cat > .env << 'EOF'
DATABASE_URL=postgresql://spectrum_user:spectrum_pass_123@localhost:5432/spectrum_poc
RUST_LOG=backend=debug,tower_http=debug,sqlx=debug
SERVER_PORT=3000
EOF
```

---

## Step 4: Create Migration Files

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc

# Create migrations directory
mkdir -p migrations

# Create migration 1: Base schema and tenants table
cat > migrations/20241219000001_create_base_schema.sql << 'EOF'
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants table (shared schema)
CREATE TABLE IF NOT EXISTS public.tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    schema_name VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sync state tracking (shared schema)
CREATE TABLE IF NOT EXISTS public.sync_state (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    client_id UUID NOT NULL,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_checkpoint BIGINT NOT NULL DEFAULT 0,
    UNIQUE(tenant_id, client_id)
);

-- Operation log for sync (shared schema)
CREATE TABLE IF NOT EXISTS public.operation_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    table_name VARCHAR(100) NOT NULL,
    operation VARCHAR(20) NOT NULL CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
    record_id UUID NOT NULL,
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version BIGINT NOT NULL,
    client_id UUID
);

-- Index for efficient sync queries
CREATE INDEX idx_operation_log_tenant_version ON public.operation_log(tenant_id, version);
CREATE INDEX idx_operation_log_timestamp ON public.operation_log(timestamp);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for tenants table
CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON public.tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
EOF

# Create migration 2: Tenant schema template function
cat > migrations/20241219000002_create_tenant_schema_function.sql << 'EOF'
-- Function to create a new tenant schema with all tables
CREATE OR REPLACE FUNCTION create_tenant_schema(p_tenant_id UUID, p_schema_name VARCHAR)
RETURNS void AS $$
BEGIN
    -- Create schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', p_schema_name);
    
    -- Create clients table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.clients (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            phone VARCHAR(50),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name);
    
    -- Create properties table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.properties (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            client_id UUID NOT NULL REFERENCES %I.clients(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            address TEXT NOT NULL,
            city VARCHAR(100),
            state VARCHAR(50),
            zip_code VARCHAR(20),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create buildings table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.buildings (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            property_id UUID NOT NULL REFERENCES %I.properties(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            floors INTEGER NOT NULL DEFAULT 1,
            year_built INTEGER,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create units table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.units (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            building_id UUID NOT NULL REFERENCES %I.buildings(id) ON DELETE CASCADE,
            unit_number VARCHAR(50) NOT NULL,
            unit_type VARCHAR(50) NOT NULL,
            square_feet INTEGER,
            bedrooms INTEGER,
            bathrooms DECIMAL(3,1),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create reports table (source of truth: SERVER)
    EXECUTE format('
        CREATE TABLE %I.reports (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            property_id UUID NOT NULL REFERENCES %I.properties(id) ON DELETE CASCADE,
            report_type VARCHAR(100) NOT NULL,
            title VARCHAR(255) NOT NULL,
            pdf_url TEXT,
            status VARCHAR(50) NOT NULL DEFAULT ''pending'',
            generated_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1
        )', p_schema_name, p_schema_name);
    
    -- Create triggers for updated_at on all tables
    EXECUTE format('
        CREATE TRIGGER update_clients_updated_at
            BEFORE UPDATE ON %I.clients
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_properties_updated_at
            BEFORE UPDATE ON %I.properties
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_buildings_updated_at
            BEFORE UPDATE ON %I.buildings
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_units_updated_at
            BEFORE UPDATE ON %I.units
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    -- Create indexes for foreign keys and common queries
    EXECUTE format('CREATE INDEX idx_%s_properties_client_id ON %I.properties(client_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_buildings_property_id ON %I.buildings(property_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_units_building_id ON %I.units(building_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_reports_property_id ON %I.reports(property_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    
    -- Grant permissions
    EXECUTE format('GRANT ALL PRIVILEGES ON SCHEMA %I TO spectrum_user', p_schema_name);
    EXECUTE format('GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA %I TO spectrum_user', p_schema_name);
    EXECUTE format('GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA %I TO spectrum_user', p_schema_name);
END;
$$ LANGUAGE plpgsql;

-- Function to drop tenant schema
CREATE OR REPLACE FUNCTION drop_tenant_schema(p_schema_name VARCHAR)
RETURNS void AS $$
BEGIN
    EXECUTE format('DROP SCHEMA IF EXISTS %I CASCADE', p_schema_name);
END;
$$ LANGUAGE plpgsql;
EOF

# Create migration 3: Create demo tenant
cat > migrations/20241219000003_create_demo_tenant.sql << 'EOF'
-- Insert demo tenant
INSERT INTO public.tenants (id, schema_name, name)
VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'tenant_demo',
    'Demo Tenant'
)
ON CONFLICT (schema_name) DO NOTHING;

-- Create schema for demo tenant
SELECT create_tenant_schema(
    '00000000-0000-0000-0000-000000000001'::UUID,
    'tenant_demo'
);

-- Insert sample data for testing
INSERT INTO tenant_demo.clients (id, name, email, phone)
VALUES 
    ('10000000-0000-0000-0000-000000000001'::UUID, 'John Doe', 'john@example.com', '555-0100'),
    ('10000000-0000-0000-0000-000000000002'::UUID, 'Jane Smith', 'jane@example.com', '555-0200')
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.properties (id, client_id, name, address, city, state, zip_code)
VALUES 
    (
        '20000000-0000-0000-0000-000000000001'::UUID,
        '10000000-0000-0000-0000-000000000001'::UUID,
        'Sunset Apartments',
        '123 Main Street',
        'Springfield',
        'IL',
        '62701'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.buildings (id, property_id, name, floors, year_built)
VALUES 
    (
        '30000000-0000-0000-0000-000000000001'::UUID,
        '20000000-0000-0000-0000-000000000001'::UUID,
        'Building A',
        5,
        2015
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.units (id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms)
VALUES 
    (
        '40000000-0000-0000-0000-000000000001'::UUID,
        '30000000-0000-0000-0000-000000000001'::UUID,
        '101',
        '1BR',
        750,
        1,
        1.0
    ),
    (
        '40000000-0000-0000-0000-000000000002'::UUID,
        '30000000-0000-0000-0000-000000000001'::UUID,
        '102',
        '2BR',
        1100,
        2,
        2.0
    )
ON CONFLICT (id) DO NOTHING;
EOF
```

---

## Step 5: Run Migrations

```bash
cd backend

# Run all migrations
sqlx migrate run

# Verify migrations
psql -U spectrum_user -d spectrum_poc -h localhost << 'EOF'
-- Check tenants
SELECT * FROM public.tenants;

-- Check if demo tenant schema exists
SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'tenant_demo';

-- Check demo data
SELECT * FROM tenant_demo.clients;
SELECT * FROM tenant_demo.properties;
SELECT * FROM tenant_demo.buildings;
SELECT * FROM tenant_demo.units;

\q
EOF
```

---

## Step 6: Create SQLite Schema for Desktop

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/desktop

# Create SQL file for SQLite schema
cat > schema.sql << 'EOF'
-- SQLite schema for local desktop database
-- This mirrors the PostgreSQL tenant schema structure

-- Clients table
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    _sync_status TEXT DEFAULT 'synced' -- 'synced', 'pending', 'conflict'
);

-- Properties table
CREATE TABLE IF NOT EXISTS properties (
    id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    city TEXT,
    state TEXT,
    zip_code TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    _sync_status TEXT DEFAULT 'synced',
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
);

-- Buildings table
CREATE TABLE IF NOT EXISTS buildings (
    id TEXT PRIMARY KEY,
    property_id TEXT NOT NULL,
    name TEXT NOT NULL,
    floors INTEGER NOT NULL DEFAULT 1,
    year_built INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    _sync_status TEXT DEFAULT 'synced',
    FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
);

-- Units table
CREATE TABLE IF NOT EXISTS units (
    id TEXT PRIMARY KEY,
    building_id TEXT NOT NULL,
    unit_number TEXT NOT NULL,
    unit_type TEXT NOT NULL,
    square_feet INTEGER,
    bedrooms INTEGER,
    bathrooms REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    _sync_status TEXT DEFAULT 'synced',
    FOREIGN KEY (building_id) REFERENCES buildings(id) ON DELETE CASCADE
);

-- Reports table (read-only from server)
CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    property_id TEXT NOT NULL,
    report_type TEXT NOT NULL,
    title TEXT NOT NULL,
    pdf_url TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    generated_at TEXT,
    created_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
);

-- Sync metadata
CREATE TABLE IF NOT EXISTS _sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Pending operations queue
CREATE TABLE IF NOT EXISTS _pending_operations (
    id TEXT PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL, -- 'INSERT', 'UPDATE', 'DELETE'
    record_id TEXT NOT NULL,
    data TEXT NOT NULL, -- JSON
    timestamp TEXT NOT NULL,
    retry_count INTEGER DEFAULT 0
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_properties_client_id ON properties(client_id);
CREATE INDEX IF NOT EXISTS idx_buildings_property_id ON buildings(property_id);
CREATE INDEX IF NOT EXISTS idx_units_building_id ON units(building_id);
CREATE INDEX IF NOT EXISTS idx_reports_property_id ON reports(property_id);
CREATE INDEX IF NOT EXISTS idx_pending_ops_timestamp ON _pending_operations(timestamp);

-- Insert initial sync metadata
INSERT OR IGNORE INTO _sync_metadata (key, value) VALUES ('last_sync_version', '0');
INSERT OR IGNORE INTO _sync_metadata (key, value) VALUES ('tenant_id', '');
INSERT OR IGNORE INTO _sync_metadata (key, value) VALUES ('client_id', '');
EOF
```

---

## Step 7: Update Desktop DB Module

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/desktop/src

cat > db.rs << 'EOF'
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        Ok(Database { conn })
    }
    
    pub fn init_schema(&self) -> Result<()> {
        let schema = include_str!("../schema.sql");
        self.conn.execute_batch(schema)?;
        Ok(())
    }
    
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

pub async fn init_db() -> Result<Database> {
    // Get app data directory
    let app_data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
        .join("spectrum-tenant-poc");
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)?;
    
    let db_path = app_data_dir.join("spectrum.db");
    println!("Initializing database at: {:?}", db_path);
    
    let db = Database::new(db_path)?;
    db.init_schema()?;
    
    println!("Database initialized successfully");
    Ok(db)
}
EOF

# Add dirs dependency for getting app data directory
cd ..
cargo add dirs
```

---

## Step 8: Update Backend DB Module

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/backend/src

mkdir -p db

cat > db/mod.rs << 'EOF'
use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

pub mod tenants;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
EOF

cat > db/tenants.rs << 'EOF'
use sqlx::PgPool;
use anyhow::Result;
use uuid::Uuid;

pub struct Tenant {
    pub id: Uuid,
    pub schema_name: String,
    pub name: String,
}

pub async fn get_tenant_by_id(pool: &PgPool, tenant_id: Uuid) -> Result<Option<Tenant>> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, schema_name, name
        FROM public.tenants
        WHERE id = $1
        "#,
        tenant_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(tenant)
}

pub async fn get_tenant_by_schema(pool: &PgPool, schema_name: &str) -> Result<Option<Tenant>> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, schema_name, name
        FROM public.tenants
        WHERE schema_name = $1
        "#,
        schema_name
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(tenant)
}

pub async fn create_tenant(pool: &PgPool, name: &str, schema_name: &str) -> Result<Tenant> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        INSERT INTO public.tenants (schema_name, name)
        VALUES ($1, $2)
        RETURNING id, schema_name, name
        "#,
        schema_name,
        name
    )
    .fetch_one(pool)
    .await?;
    
    // Create the tenant schema
    sqlx::query("SELECT create_tenant_schema($1, $2)")
        .bind(tenant.id)
        .bind(&tenant.schema_name)
        .execute(pool)
        .await?;
    
    Ok(tenant)
}
EOF
```

---

## Step 9: Update Backend Main to Use Database

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/backend/src

cat > main.rs << 'EOF'
use axum::{
    Router,
    routing::get,
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

    // Build application routes
    let app = Router::new()
        .route("/", get(|| async { "Spectrum Tenant POC API" }))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
EOF
```

---

## Step 10: Test Everything

```bash
# Test backend compilation
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/backend
cargo check

# Test backend runs
cargo run
# Should see: "Database connected successfully"
# Press Ctrl+C to stop

# Test desktop compilation
cd ../desktop
cargo check

# Commit progress
cd ..
git add -A
git commit -m "Phase 2: Database setup complete with PostgreSQL and SQLite schemas"
git tag -a v0.2.0-database -m "Phase 2: Database setup complete"
```

---

## Verification Checklist

- [ ] PostgreSQL database created and accessible
- [ ] Migrations run successfully
- [ ] Demo tenant created with sample data
- [ ] Backend connects to database
- [ ] SQLite schema created for desktop
- [ ] Foreign keys and indexes in place
- [ ] Both backend and desktop compile without errors

---

## Troubleshooting

### PostgreSQL Connection Issues

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check connection manually
psql -U spectrum_user -d spectrum_poc -h localhost

# Reset password if needed
sudo -u postgres psql
ALTER USER spectrum_user PASSWORD 'spectrum_pass_123';
```

### Migration Errors

```bash
# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Drop database and recreate
sudo -u postgres psql
DROP DATABASE spectrum_poc;
CREATE DATABASE spectrum_poc;
GRANT ALL PRIVILEGES ON DATABASE spectrum_poc TO spectrum_user;
\q

# Rerun migrations
cd backend
sqlx migrate run
```

---

## Next Steps

After Phase 2:
1. **Phase 3**: Implement backend API handlers for CRUD operations
2. **Phase 4**: Build desktop UI with SvelteKit
3. **Phase 5**: Implement sync logic

Your database foundation is now solid! 🎉