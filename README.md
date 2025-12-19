# Spectrum Tenant POC

Offline-first multi-tenant system with bidirectional sync between desktop clients and server.

## Architecture

- **Backend**: Axum + PostgreSQL (schema-based tenancy)
- **Desktop**: Tauri + SvelteKit + SQLite
- **Sync**: PowerSync / Custom CRDT
- **Shared**: Common Rust types across client and server

## Project Structure

```
spectrum-tenant-poc/
├── backend/          # Axum REST API server
│   ├── src/
│   │   ├── main.rs
│   │   ├── db/           # Database operations
│   │   ├── handlers/     # HTTP request handlers
│   │   ├── middleware/   # Tenant resolution, auth
│   │   ├── models/       # Backend-specific models
│   │   └── routes/       # Route definitions
│   ├── Cargo.toml
│   └── .env
├── desktop/          # Tauri desktop application
│   ├── src/
│   │   ├── main.rs
│   │   ├── db/           # SQLite operations
│   │   ├── sync/         # Sync logic
│   │   └── commands/     # Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── frontend/         # SvelteKit UI
│   ├── src/
│   │   ├── routes/       # SvelteKit pages
│   │   ├── lib/          # Shared components, stores
│   │   └── app.html
│   ├── package.json
│   └── svelte.config.js
├── shared/           # Shared Rust types
│   ├── src/
│   │   ├── lib.rs        # Core types (Client, Property, etc.)
│   │   ├── models/       # Data models
│   │   └── sync/         # Sync protocol types
│   └── Cargo.toml
├── migrations/       # PostgreSQL migrations
├── docs/             # Documentation
│   └── SETUP.md
├── Cargo.toml        # Workspace configuration
└── README.md
```

## Data Model

### Core Entities
- **Clients** (Users) → **Properties** → **Buildings** → **Units**
- **Reports** (Server-generated PDFs)

### Source of Truth
- **Client-owned**: clients, properties, buildings, units (offline-first)
- **Server-owned**: reports (server authoritative)

### Tenancy
- Schema-based isolation: each tenant gets `tenant_<id>` schema in PostgreSQL
- Strong isolation with separate schemas per tenant
- Metadata in shared `public` schema

## Getting Started

### Prerequisites

- **Rust**: 1.75 or higher
- **Node.js**: 18 or higher
- **PostgreSQL**: 16 or higher
- **System dependencies**: 
  - Linux: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - macOS: Xcode Command Line Tools
  - Windows: Microsoft C++ Build Tools

### Installation

```bash
# Clone repository
git clone <repository-url>
cd spectrum-tenant-poc

# Install Rust dependencies
cargo build --all

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### Database Setup

```bash
# Create PostgreSQL database
createdb spectrum_tenant_poc

# Run migrations (after Phase 2)
cd backend
sqlx migrate run
```

### Configuration

```bash
# Backend configuration
cp backend/.env.example backend/.env
# Edit backend/.env with your database credentials

# DATABASE_URL=postgresql://postgres:password@localhost:5432/spectrum_tenant_poc
```

## Development

### Run Backend Server

```bash
cd backend
cargo run

# Server will start on http://localhost:3000
```

### Run Desktop Application

```bash
cd desktop
cargo tauri dev

# This will:
# 1. Start frontend dev server (Vite on :5173)
# 2. Compile Rust backend
# 3. Open desktop application window
```

### Run Frontend Only (for development)

```bash
cd frontend
npm run dev

# Frontend will be available at http://localhost:5173
```

### Convenience Scripts

```bash
# From project root
make dev-backend    # Run backend
make dev-desktop    # Run desktop app
make clean          # Clean all build artifacts
make test           # Run all tests
```

## Testing

```bash
# Test all Rust code
cargo test --all

# Test frontend
cd frontend
npm run test
```

## Building for Production

### Backend

```bash
cd backend
cargo build --release

# Binary will be at target/release/backend
```

### Desktop Application

```bash
cd desktop
cargo tauri build

# Installers will be in target/release/bundle/
```

## Project Status

- [x] Phase 1: Project setup and structure
- [ ] Phase 2: Database setup and migrations
- [ ] Phase 3: Backend implementation
- [ ] Phase 4: Desktop application
- [ ] Phase 5: Sync implementation
- [ ] Phase 6: Testing
- [ ] Phase 7: Documentation

## Key Features (Planned)

- ✅ Offline-first architecture
- ✅ Multi-tenant with schema isolation
- ✅ Bidirectional sync with conflict resolution
- ✅ Client and server source-of-truth models
- ✅ PDF report generation on server
- ✅ Desktop application with local SQLite

## Technology Stack

### Backend
- **Axum** - Web framework
- **SQLx** - Type-safe SQL
- **PostgreSQL** - Production database
- **Tokio** - Async runtime

### Desktop
- **Tauri 2.x** - Desktop framework
- **SQLite** - Local database
- **Rusqlite** - SQLite bindings

### Frontend
- **SvelteKit** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool

### Sync
- **PowerSync** - Offline sync engine (planned)
- **CRDT** - Conflict-free replicated data types

## Architecture Decisions

### Why Schema-Based Tenancy?
- Strong isolation between tenants
- Easier per-tenant backups and migrations
- Can optimize per-tenant
- Trade-off: Cross-tenant queries require unions

### Why PowerSync?
- Built specifically for PostgreSQL
- Handles CRDT-like conflict resolution
- Production-ready sync protocol
- Good Rust/TypeScript support

### Why Tauri?
- Native performance with Rust
- Smaller bundle size than Electron
- Direct filesystem and SQLite access
- Secure by default

## Contributing

See [docs/SETUP.md](docs/SETUP.md) for detailed development setup.

## License

MIT

## Support

For issues and questions, please open an issue on GitHub.