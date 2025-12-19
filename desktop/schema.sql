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
