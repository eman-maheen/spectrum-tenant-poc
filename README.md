# Spectrum Tenant POC - Offline-First Multi-Tenant System

A proof-of-concept demonstrating an offline-first, multi-tenant property management system with bidirectional sync between desktop clients and a central server.

## 🏗️ Architecture Overview

### Stack
- **Backend**: Rust (Axum) + PostgreSQL
- **Desktop**: Rust (Tauri) + SvelteKit + SQLite
- **Sync**: Custom implementation with conflict resolution

### How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     DESKTOP APPLICATION                      │
│  ┌────────────┐         ┌──────────┐        ┌─────────────┐│
│  │  SvelteKit │ ◄─────► │  Tauri   │ ◄────► │   SQLite    ││
│  │    (UI)    │         │ Commands │        │  (Local DB) ││
│  └────────────┘         └──────────┘        └─────────────┘│
│                               │                              │
│                               │ HTTP/REST                    │
└───────────────────────────────┼──────────────────────────────┘
                                │
                         ┌──────▼──────┐
                         │   Internet  │
                         └──────┬──────┘
                                │
┌───────────────────────────────┼──────────────────────────────┐
│                    BACKEND SERVER (Axum)                      │
│  ┌────────────┐         ┌──────────┐        ┌─────────────┐│
│  │   REST     │ ◄─────► │  Tenant  │ ◄────► │ PostgreSQL  ││
│  │    API     │         │Middleware│        │(Multi-tenant│││
│  └────────────┘         └──────────┘        └─────────────┘│
└───────────────────────────────────────────────────────────────┘
```

### Data Model

**Entities** (with hierarchical relationships):
```
Clients (Users)
  └─► Properties
        └─► Buildings
              └─► Units
```

**Source of Truth**:
- **Client-owned data** (Clients, Properties, Buildings, Units): Desktop app is authoritative
- **Server-owned data** (Reports): Backend is authoritative

### Sync Mechanism

**How Sync Works:**

1. **Offline Creation** (Backend OFF):
   - Data saved to local SQLite with `_sync_status = 'pending'`
   - User can continue working without interruption

2. **Online Creation** (Backend ON):
   - Desktop tries to push to backend first
   - If successful: saves to SQLite with `_sync_status = 'synced'`
   - If failed: falls back to offline mode

3. **Pull Sync** (Manual):
   - User clicks "Sync Now"
   - Desktop fetches all data from backend
   - Upserts into local SQLite (INSERT OR REPLACE)
   - Marks all pulled data as `_sync_status = 'synced'`

4. **Push Sync** (Automatic on create):
   - On create/update/delete, desktop immediately tries to push to server
   - Uses HTTP REST API with tenant header: `x-tenant-id`

**Conflict Resolution Strategy:**
- Currently: **Server wins** (simple for POC)
- Future: Can implement Last-Write-Wins (LWW) using version numbers

---

## 📋 Prerequisites

- **Rust**: 1.75+ ([Install Rust](https://rustup.rs/))
- **Node.js**: 18+ ([Install Node](https://nodejs.org/))
- **PostgreSQL**: 16+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- **System Dependencies**:
  - **Linux**: `libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft C++ Build Tools

### Installing System Dependencies

**Arch Linux / CachyOS:**
```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget openssl librsvg
```

**Ubuntu / Debian:**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

---

## 🚀 Setup Instructions

### 1. Clone Repository

```bash
git clone <repository-url>
cd spectrum-tenant-poc
```

### 2. Setup PostgreSQL Database

```bash
# Create database
sudo -u postgres psql << 'EOF'
CREATE DATABASE spectrum_tenant_poc;
CREATE USER spectum WITH PASSWORD 'spectum_pass' SUPERUSER;
GRANT ALL PRIVILEGES ON DATABASE spectrum_tenant_poc TO spectum;
\q
EOF

# Connect and create UUID extension
sudo -u postgres psql -d spectrum_tenant_poc
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
\q
```

### 3. Configure Backend

```bash
cd backend

# Create environment file
cat > .env << 'EOF'
DATABASE_URL=postgresql://spectum:spectum_pass@localhost:5432/spectrum_tenant_poc
RUST_LOG=backend=debug,tower_http=debug,sqlx=info
SERVER_PORT=3000
EOF

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

### 4. Install Dependencies

```bash
# Build workspace (from project root)
cd ..
cargo build --all

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### 5. Install Tauri CLI

```bash
cargo install tauri-cli --version "^2.0.0"
```

---

## 🏃 Running the Application

### Development Mode

**Terminal 1 - Backend Server:**
```bash
cd backend
cargo run
```

**Terminal 2 - Frontend Dev Server:**
```bash
cd frontend
npm run dev
```

**Terminal 3 - Desktop App:**
```bash
cd desktop
cargo tauri dev
```

### Production Build

```bash
# Build backend
cd backend
cargo build --release

# Build desktop app
cd ../desktop
cargo tauri build
```

---

## 🧪 Testing the Sync Mechanism

### Test 1: Online Sync (Backend Running)

1. **Start backend**:
   ```bash
   cd backend && cargo run
   ```

2. **Start desktop app**:
   ```bash
   cd desktop && cargo tauri dev
   ```

3. **Verify connection**:
   - Look for 🟢 **"Online"** status in sidebar
   - Click **"Sync Now"** button
   - Should see demo data appear (2 clients, 1 property)

4. **Create a new client**:
   - Click "Clients" → "+ New Client"
   - Fill form and submit
   - **Expected**: Data appears immediately, marked as "Synced"

5. **Verify in backend**:
   ```bash
   curl -H "x-tenant-id: 00000000-0000-0000-0000-000000000001" \
        http://localhost:3000/api/clients | jq
   ```
   Should show your new client!

### Test 2: Offline Mode (Backend NOT Running)

1. **Stop backend** (Ctrl+C in backend terminal)

2. **Check desktop app**:
   - Status should show 🔴 **"Offline"**
   - "Sync Now" button should be disabled

3. **Create offline data**:
   - Click "Clients" → "+ New Client"
   - Name: "Offline User", Email: "offline@test.com"
   - Submit
   - **Expected**: Client appears with "Pending Sync" badge

4. **Verify SQLite saved it**:
   ```bash
   # Find your SQLite database
   sqlite3 ~/.local/share/spectrum-tenant-poc/spectrum.db
   
   # Query clients
   SELECT name, email, _sync_status FROM clients;
   
   # Should show:
   # Offline User|offline@test.com|pending
   
   # Exit
   .quit
   ```

5. **Start backend again**:
   ```bash
   cd backend && cargo run
   ```

6. **Manual sync**:
   - Desktop should show 🟢 "Online" again
   - Click **"Sync Now"**
   - Data from server overwrites local (pull)
   - ⚠️ **Current limitation**: Pending local changes are NOT pushed automatically
   
### Test 3: Data Persistence

1. **Close desktop app** completely

2. **Restart desktop app**:
   ```bash
   cd desktop && cargo tauri dev
   ```

3. **Verify data persists**:
   - All clients and properties should still be there
   - Data is stored in SQLite at: `~/.local/share/spectrum-tenant-poc/spectrum.db`

---

## 🗄️ Querying the SQLite Database

### Location
```bash
# Linux/macOS
~/.local/share/spectrum-tenant-poc/spectrum.db

# Windows
%APPDATA%\spectrum-tenant-poc\spectrum.db
```

### Using SQLite CLI

```bash
# Open database
sqlite3 ~/.local/share/spectrum-tenant-poc/spectrum.db

# List all tables
.tables

# View clients with sync status
SELECT id, name, email, _sync_status, is_deleted FROM clients;

# View properties
SELECT id, name, address, _sync_status FROM properties;

# Check pending sync items
SELECT * FROM clients WHERE _sync_status = 'pending';

# View schema
.schema clients

# Exit
.quit
```

### Using GUI Tool

Install [DB Browser for SQLite](https://sqlitebrowser.org/):
```bash
# Arch Linux
sudo pacman -S sqlitebrowser

# Ubuntu
sudo apt install sqlitebrowser
```

Then open: `~/.local/share/spectrum-tenant-poc/spectrum.db`

---

## 🔄 Understanding Push vs Pull

### Pull Sync (Manual)
**Requires**: Backend must be running ✅

**What it does**:
1. Desktop sends GET request to `/api/clients` and `/api/properties`
2. Backend returns all data for the tenant
3. Desktop does `INSERT OR REPLACE` into SQLite
4. Marks all data as `_sync_status = 'synced'`

**When to use**: 
- Get latest data from server
- After being offline for a while
- To see data created on other devices

### Push Sync (Automatic on Create)
**Requires**: Backend must be running (but fails gracefully) ⚠️

**What it does**:
1. When creating client/property, desktop sends POST to `/api/clients`
2. If successful: saves server response (with server-generated ID) to SQLite
3. If failed: saves locally with UUID and marks as `pending`

**Offline behavior**:
- Data saves to SQLite immediately
- Marked as `_sync_status = 'pending'`
- ⚠️ **Current limitation**: Not auto-pushed when back online (would need background worker)

### Delete (Soft Delete)
**Requires**: Backend running (optional)

**What it does**:
1. Tries to DELETE on server: `/api/clients/:id`
2. Always soft-deletes locally: `UPDATE clients SET is_deleted = 1`
3. Data hidden from UI but preserved in database

---

## 🎯 Current Sync Capabilities

### ✅ What Works

| Operation | Online (Backend ON) | Offline (Backend OFF) |
|-----------|--------------------|-----------------------|
| **View Data** | ✅ Yes | ✅ Yes (from SQLite) |
| **Create Client/Property** | ✅ Syncs immediately | ✅ Saves as `pending` |
| **Delete** | ✅ Deletes on server | ✅ Soft delete locally |
| **Pull Sync** | ✅ Gets server data | ❌ Disabled |
| **Data Persistence** | ✅ Saved to SQLite | ✅ Saved to SQLite |

### ⚠️ Current Limitations (POC Scope)

1. **No automatic push of pending changes**
   - When you go back online, pending items stay pending
   - Would need: Background sync worker to push pending changes

2. **No update (edit) functionality**
   - Can create and delete, but not edit
   - Would need: Update endpoints and UI forms

3. **No real conflict resolution**
   - Pull sync overwrites local changes
   - Would need: Version-based conflict detection and merge UI

4. **No background auto-sync**
   - Sync is manual (click button)
   - Would need: Timer-based background sync every 30s

5. **Buildings & Units not implemented**
   - Placeholder pages only
   - Would need: Same pattern as Clients/Properties

---

## 🏗️ Project Structure

```
spectrum-tenant-poc/
├── backend/                 # Axum REST API server
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── db/             # Database operations
│   │   │   ├── clients.rs
│   │   │   ├── properties.rs
│   │   │   └── tenants.rs
│   │   ├── handlers/       # HTTP handlers (CRUD)
│   │   ├── middleware/     # Tenant resolution
│   │   ├── models/         # Request/Response types
│   │   └── routes/         # Route definitions
│   ├── migrations/         # SQL migrations
│   └── .env               # Database config
│
├── desktop/                # Tauri desktop app
│   ├── src/
│   │   ├── main.rs        # Tauri entry point
│   │   ├── db.rs          # SQLite operations
│   │   ├── sync.rs        # Sync service (HTTP client)
│   │   └── commands.rs    # Tauri commands (API bridge)
│   ├── schema.sql         # SQLite schema
│   └── tauri.conf.json    # Tauri configuration
│
├── frontend/               # SvelteKit UI
│   ├── src/
│   │   ├── routes/        # Pages (Dashboard, Clients, etc.)
│   │   ├── lib/
│   │   │   ├── api/       # Tauri API wrapper
│   │   │   ├── stores/    # Svelte stores
│   │   │   ├── types/     # TypeScript types
│   │   │   └── components/ # Reusable components
│   │   └── app.html
│   └── package.json
│
├── shared/                 # Shared Rust types
│   └── src/
│       ├── lib.rs         # Common types (Client, Property, etc.)
│       └── sync/          # Sync protocol types
│
└── README.md              # This file
```

---

## 🔑 Key Technologies Explained

### Why Tauri?
- **Small bundle size**: ~3MB vs Electron's ~100MB
- **Native performance**: Rust backend, native webview
- **Security**: Sandboxed by default
- **Direct database access**: Can use rusqlite for SQLite

### Why SvelteKit?
- **Lightweight**: Less framework overhead than React
- **Reactive**: Automatic UI updates
- **SSR-friendly**: Can export as static site
- **Great DX**: Intuitive syntax, easy routing

### Why Schema-Based Tenancy?
- **Strong isolation**: Each tenant has separate PostgreSQL schema
- **Easier backups**: Can dump per-tenant
- **Migration control**: Can update one tenant at a time
- **Trade-off**: Cross-tenant queries are harder

### Sync Architecture
- **Not using CRDT libraries**: Custom implementation for POC
- **Not using PowerSync**: Would require additional setup
- **Simple HTTP REST**: Easy to understand and debug
- **Version numbers**: Future-proof for conflict resolution

---

## 🐛 Troubleshooting

### Backend won't start
```bash
# Check if port 3000 is in use
lsof -i :3000

# Check PostgreSQL is running
sudo systemctl status postgresql

# Check database connection
psql -U spectum -d spectrum_tenant_poc -h localhost
```

### Desktop app shows "Offline" when backend is running
```bash
# Check backend is accessible
curl http://localhost:3000/health

# Check Tauri console for errors (in desktop app, press F12)

# Restart both backend and desktop
```

### SQLite database location?
```bash
# Linux/macOS
ls -la ~/.local/share/spectrum-tenant-poc/

# Check if it exists
sqlite3 ~/.local/share/spectrum-tenant-poc/spectrum.db ".tables"
```

### Frontend build errors
```bash
cd frontend
rm -rf node_modules .svelte-kit build
npm install
npm run build
```

---

## 📚 API Documentation

### Tenant Header
All API requests require: `x-tenant-id: <uuid>`

Demo tenant ID: `00000000-0000-0000-0000-000000000001`

### Endpoints

**Clients:**
```bash
GET    /api/clients          # List all
GET    /api/clients/:id      # Get one
POST   /api/clients          # Create
PUT    /api/clients/:id      # Update
DELETE /api/clients/:id      # Delete
```

**Properties:**
```bash
GET    /api/properties       # List all
GET    /api/properties/:id   # Get one
POST   /api/properties       # Create
PUT    /api/properties/:id   # Update
DELETE /api/properties/:id   # Delete
```

**Example Request:**
```bash
curl -X POST http://localhost:3000/api/clients \
  -H "x-tenant-id: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Client",
    "email": "test@example.com",
    "phone": "555-1234"
  }'
```

---

## 🎓 Learning Resources

- [Tauri Documentation](https://tauri.app/)
- [SvelteKit Documentation](https://kit.svelte.dev/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Offline-First Patterns](https://offlinefirst.org/)

---

## 📝 License

MIT

---