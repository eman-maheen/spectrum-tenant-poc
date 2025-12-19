# Phase 4: Desktop Application UI - Step by Step

## Overview

In this phase, we'll build:
1. SvelteKit UI components for CRUD operations
2. Tauri commands for SQLite access
3. Local data management
4. API communication layer
5. Basic offline capability

---

## Step 1: Update Desktop Database Module

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/desktop/src

cat > db.rs << 'EOF'
use anyhow::Result;
use rusqlite::{Connection, params};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(Database { conn })
    }
    
    pub fn init_schema(&self) -> Result<()> {
        let schema = include_str!("../schema.sql");
        self.conn.execute_batch(schema)?;
        Ok(())
    }
    
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
    
    // Client operations
    pub fn list_clients(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, email, phone, created_at, updated_at, version, _sync_status 
             FROM clients WHERE is_deleted = 0 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "email": row.get::<_, String>(2)?,
                "phone": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
                "version": row.get::<_, i64>(6)?,
                "sync_status": row.get::<_, String>(7)?,
            }))
        })?;
        
        let mut clients = Vec::new();
        for row in rows {
            clients.push(row?);
        }
        Ok(clients)
    }
    
    pub fn get_client(&self, id: &str) -> Result<Option<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, email, phone, created_at, updated_at, version, _sync_status 
             FROM clients WHERE id = ?1 AND is_deleted = 0"
        )?;
        
        let result = stmt.query_row([id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "email": row.get::<_, String>(2)?,
                "phone": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
                "version": row.get::<_, i64>(6)?,
                "sync_status": row.get::<_, String>(7)?,
            }))
        });
        
        match result {
            Ok(client) => Ok(Some(client)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn create_client(&self, name: &str, email: &str, phone: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO clients (id, name, email, phone, created_at, updated_at, _sync_status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
            params![&id, name, email, phone, &now, &now],
        )?;
        
        Ok(id)
    }
    
    pub fn update_client(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE clients 
             SET name = COALESCE(?1, name),
                 email = COALESCE(?2, email),
                 phone = COALESCE(?3, phone),
                 updated_at = ?4,
                 version = version + 1,
                 _sync_status = 'pending'
             WHERE id = ?5",
            params![name, email, phone, &now, id],
        )?;
        
        Ok(())
    }
    
    pub fn delete_client(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE clients 
             SET is_deleted = 1,
                 updated_at = ?1,
                 version = version + 1,
                 _sync_status = 'pending'
             WHERE id = ?2",
            params![&now, id],
        )?;
        
        Ok(())
    }
    
    // Property operations
    pub fn list_properties(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, client_id, name, address, city, state, zip_code, 
                    created_at, updated_at, version, _sync_status 
             FROM properties WHERE is_deleted = 0 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "client_id": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "address": row.get::<_, String>(3)?,
                "city": row.get::<_, Option<String>>(4)?,
                "state": row.get::<_, Option<String>>(5)?,
                "zip_code": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "updated_at": row.get::<_, String>(8)?,
                "version": row.get::<_, i64>(9)?,
                "sync_status": row.get::<_, String>(10)?,
            }))
        })?;
        
        let mut properties = Vec::new();
        for row in rows {
            properties.push(row?);
        }
        Ok(properties)
    }
    
    pub fn create_property(
        &self,
        client_id: &str,
        name: &str,
        address: &str,
        city: Option<&str>,
        state: Option<&str>,
        zip_code: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO properties (id, client_id, name, address, city, state, zip_code, created_at, updated_at, _sync_status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending')",
            params![&id, client_id, name, address, city, state, zip_code, &now, &now],
        )?;
        
        Ok(id)
    }
}

pub async fn init_db() -> Result<Database> {
    let app_data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
        .join("spectrum-tenant-poc");
    
    std::fs::create_dir_all(&app_data_dir)?;
    
    let db_path = app_data_dir.join("spectrum.db");
    println!("Initializing database at: {:?}", db_path);
    
    let db = Database::new(db_path)?;
    db.init_schema()?;
    
    println!("Database initialized successfully");
    Ok(db)
}
EOF
```

---

## Step 2: Create Tauri Commands

```bash
cat > commands.rs << 'EOF'
use tauri::State;
use std::sync::Mutex;
use crate::db::Database;

// Wrap database in Mutex for thread-safe access
pub struct AppState {
    pub db: Mutex<Database>,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Spectrum Tenant POC.", name)
}

// Client commands
#[tauri::command]
pub fn list_clients(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let clients = db.list_clients().map_err(|e| e.to_string())?;
    serde_json::to_string(&clients).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_client(state: State<AppState>, id: String) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let client = db.get_client(&id).map_err(|e| e.to_string())?;
    serde_json::to_string(&client).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_client(
    state: State<AppState>,
    name: String,
    email: String,
    phone: Option<String>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = db.create_client(&name, &email, phone.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn update_client(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_client(
        &id,
        name.as_deref(),
        email.as_deref(),
        phone.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_client(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_client(&id).map_err(|e| e.to_string())
}

// Property commands
#[tauri::command]
pub fn list_properties(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let properties = db.list_properties().map_err(|e| e.to_string())?;
    serde_json::to_string(&properties).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_property(
    state: State<AppState>,
    client_id: String,
    name: String,
    address: String,
    city: Option<String>,
    state_val: Option<String>,
    zip_code: Option<String>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = db.create_property(
        &client_id,
        &name,
        &address,
        city.as_deref(),
        state_val.as_deref(),
        zip_code.as_deref(),
    ).map_err(|e| e.to_string())?;
    Ok(id)
}
EOF
```

---

## Step 3: Update main.rs

```bash
cat > main.rs << 'EOF'
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

mod db;
mod sync;
mod commands;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db = tauri::async_runtime::block_on(async {
                db::init_db().await
            }).expect("Failed to initialize database");
            
            // Store database in app state
            app.manage(AppState {
                db: Mutex::new(db),
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::list_clients,
            commands::get_client,
            commands::create_client,
            commands::update_client,
            commands::delete_client,
            commands::list_properties,
            commands::create_property,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

---

## Step 4: Create SvelteKit Layout and Stores

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/frontend/src

# Create lib directory structure
mkdir -p lib/stores lib/api lib/types

# Create types
cat > lib/types/index.ts << 'EOF'
export interface Client {
  id: string;
  name: string;
  email: string;
  phone?: string;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Property {
  id: string;
  client_id: string;
  name: string;
  address: string;
  city?: string;
  state?: string;
  zip_code?: string;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Building {
  id: string;
  property_id: string;
  name: string;
  floors: number;
  year_built?: number;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Unit {
  id: string;
  building_id: string;
  unit_number: string;
  unit_type: string;
  square_feet?: number;
  bedrooms?: number;
  bathrooms?: number;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}
EOF

# Create API wrapper
cat > lib/api/tauri.ts << 'EOF'
import { invoke } from '@tauri-apps/api/core';
import type { Client, Property } from '$lib/types';

export const tauriApi = {
  // Client operations
  async listClients(): Promise<Client[]> {
    const result = await invoke<string>('list_clients');
    return JSON.parse(result);
  },

  async getClient(id: string): Promise<Client | null> {
    const result = await invoke<string>('get_client', { id });
    return JSON.parse(result);
  },

  async createClient(name: string, email: string, phone?: string): Promise<string> {
    return await invoke<string>('create_client', { name, email, phone });
  },

  async updateClient(
    id: string,
    name?: string,
    email?: string,
    phone?: string
  ): Promise<void> {
    await invoke('update_client', { id, name, email, phone });
  },

  async deleteClient(id: string): Promise<void> {
    await invoke('delete_client', { id });
  },

  // Property operations
  async listProperties(): Promise<Property[]> {
    const result = await invoke<string>('list_properties');
    return JSON.parse(result);
  },

  async createProperty(
    clientId: string,
    name: string,
    address: string,
    city?: string,
    state?: string,
    zipCode?: string
  ): Promise<string> {
    return await invoke<string>('create_property', {
      clientId,
      name,
      address,
      city,
      stateVal: state,
      zipCode
    });
  }
};
EOF

# Create stores
cat > lib/stores/clients.ts << 'EOF'
import { writable } from 'svelte/store';
import type { Client } from '$lib/types';

export const clients = writable<Client[]>([]);
export const selectedClient = writable<Client | null>(null);
export const isLoading = writable(false);
EOF

cat > lib/stores/properties.ts << 'EOF'
import { writable } from 'svelte/store';
import type { Property } from '$lib/types';

export const properties = writable<Property[]>([]);
export const selectedProperty = writable<Property | null>(null);
EOF
```

---

## Step 5: Create Main Layout

```bash
cat > routes/+layout.svelte << 'EOF'
<script lang="ts">
  import { page } from '$app/stores';
  
  const navItems = [
    { href: '/', label: 'Dashboard' },
    { href: '/clients', label: 'Clients' },
    { href: '/properties', label: 'Properties' },
    { href: '/buildings', label: 'Buildings' },
    { href: '/units', label: 'Units' },
  ];
</script>

<div class="app">
  <nav class="sidebar">
    <div class="logo">
      <h1>Spectrum</h1>
      <p>Tenant POC</p>
    </div>
    
    <ul class="nav-items">
      {#each navItems as item}
        <li>
          <a 
            href={item.href}
            class:active={$page.url.pathname === item.href}
          >
            {item.label}
          </a>
        </li>
      {/each}
    </ul>
    
    <div class="sync-status">
      <div class="status-indicator synced"></div>
      <span>All synced</span>
    </div>
  </nav>
  
  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: #f5f5f5;
  }

  .app {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 250px;
    background: #1a1a1a;
    color: white;
    display: flex;
    flex-direction: column;
  }

  .logo {
    padding: 2rem 1.5rem;
    border-bottom: 1px solid #333;
  }

  .logo h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .logo p {
    margin: 0.25rem 0 0 0;
    font-size: 0.875rem;
    color: #888;
  }

  .nav-items {
    list-style: none;
    padding: 1rem 0;
    margin: 0;
    flex: 1;
  }

  .nav-items li a {
    display: block;
    padding: 0.75rem 1.5rem;
    color: #ccc;
    text-decoration: none;
    transition: all 0.2s;
  }

  .nav-items li a:hover {
    background: #2a2a2a;
    color: white;
  }

  .nav-items li a.active {
    background: #3b82f6;
    color: white;
  }

  .sync-status {
    padding: 1rem 1.5rem;
    border-top: 1px solid #333;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-indicator.synced {
    background: #10b981;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 2rem;
  }
</style>
EOF
```

---

## Step 6: Create Clients Page

```bash
cat > routes/clients/+page.svelte << 'EOF'
<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';
  import { clients, isLoading } from '$lib/stores/clients';
  import type { Client } from '$lib/types';

  let showForm = false;
  let formData = {
    name: '',
    email: '',
    phone: ''
  };

  onMount(async () => {
    await loadClients();
  });

  async function loadClients() {
    isLoading.set(true);
    try {
      const data = await tauriApi.listClients();
      clients.set(data);
    } catch (error) {
      console.error('Failed to load clients:', error);
      alert('Failed to load clients');
    } finally {
      isLoading.set(false);
    }
  }

  async function handleSubmit() {
    try {
      await tauriApi.createClient(
        formData.name,
        formData.email,
        formData.phone || undefined
      );
      
      // Reset form
      formData = { name: '', email: '', phone: '' };
      showForm = false;
      
      // Reload clients
      await loadClients();
    } catch (error) {
      console.error('Failed to create client:', error);
      alert('Failed to create client');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this client?')) return;
    
    try {
      await tauriApi.deleteClient(id);
      await loadClients();
    } catch (error) {
      console.error('Failed to delete client:', error);
      alert('Failed to delete client');
    }
  }
</script>

<div class="page">
  <div class="header">
    <h1>Clients</h1>
    <button class="btn-primary" on:click={() => showForm = !showForm}>
      {showForm ? 'Cancel' : '+ New Client'}
    </button>
  </div>

  {#if showForm}
    <div class="card form-card">
      <h2>New Client</h2>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="name">Name *</label>
          <input
            id="name"
            type="text"
            bind:value={formData.name}
            required
            placeholder="John Doe"
          />
        </div>

        <div class="form-group">
          <label for="email">Email *</label>
          <input
            id="email"
            type="email"
            bind:value={formData.email}
            required
            placeholder="john@example.com"
          />
        </div>

        <div class="form-group">
          <label for="phone">Phone</label>
          <input
            id="phone"
            type="tel"
            bind:value={formData.phone}
            placeholder="555-0100"
          />
        </div>

        <div class="form-actions">
          <button type="submit" class="btn-primary">Create Client</button>
        </div>
      </form>
    </div>
  {/if}

  {#if $isLoading}
    <div class="loading">Loading clients...</div>
  {:else if $clients.length === 0}
    <div class="empty">
      <p>No clients yet. Create your first client to get started!</p>
    </div>
  {:else}
    <div class="cards-grid">
      {#each $clients as client}
        <div class="card client-card">
          <div class="card-header">
            <h3>{client.name}</h3>
            {#if client.sync_status === 'pending'}
              <span class="badge badge-warning">Pending Sync</span>
            {:else}
              <span class="badge badge-success">Synced</span>
            {/if}
          </div>
          
          <div class="card-body">
            <p><strong>Email:</strong> {client.email}</p>
            {#if client.phone}
              <p><strong>Phone:</strong> {client.phone}</p>
            {/if}
            <p class="text-muted">
              Created: {new Date(client.created_at).toLocaleDateString()}
            </p>
          </div>

          <div class="card-actions">
            <button class="btn-secondary" on:click={() => alert('Edit coming in Phase 5')}>
              Edit
            </button>
            <button class="btn-danger" on:click={() => handleDelete(client.id)}>
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1400px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .header h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-primary:hover {
    background: #2563eb;
  }

  .btn-secondary {
    background: #6b7280;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-secondary:hover {
    background: #4b5563;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-danger:hover {
    background: #dc2626;
  }

  .card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
  }

  .form-card {
    margin-bottom: 2rem;
  }

  .form-card h2 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #374151;
  }

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
    box-sizing: border-box;
  }

  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .form-actions {
    margin-top: 1.5rem;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .client-card .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .client-card h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .badge-success {
    background: #d1fae5;
    color: #065f46;
  }

  .badge-warning {
    background: #fef3c7;
    color: #92400e;
  }

  .card-body p {
    margin: 0.5rem 0;
    color: #374151;
  }

  .text-muted {
    color: #6b7280 !important;
    font-size: 0.875rem;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #e5e7eb;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 3rem;
    color: #6b7280;
  }

  .empty p {
    font-size: 1.125rem;
  }
</style>
EOF
```

---

## Step 7: Create Properties Page

```bash
cat > routes/properties/+page.svelte << 'EOF'
<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';
  import { properties } from '$lib/stores/properties';
  import { clients } from '$lib/stores/clients';
  import type { Property } from '$lib/types';

  let showForm = false;
  let isLoading = false;
  let formData = {
    clientId: '',
    name: '',
    address: '',
    city: '',
    state: '',
    zipCode: ''
  };

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    isLoading = true;
    try {
      const [propertiesData, clientsData] = await Promise.all([
        tauriApi.listProperties(),
        tauriApi.listClients()
      ]);
      properties.set(propertiesData);
      clients.set(clientsData);
    } catch (error) {
      console.error('Failed to load data:', error);
      alert('Failed to load data');
    } finally {
      isLoading = false;
    }
  }

  async function handleSubmit() {
    if (!formData.clientId) {
      alert('Please select a client');
      return;
    }

    try {
      await tauriApi.createProperty(
        formData.clientId,
        formData.name,
        formData.address,
        formData.city || undefined,
        formData.state || undefined,
        formData.zipCode || undefined
      );
      
      formData = {
        clientId: '',
        name: '',
        address: '',
        city: '',
        state: '',
        zipCode: ''
      };
      showForm = false;
      await loadData();
    } catch (error) {
      console.error('Failed to create property:', error);
      alert('Failed to create property');
    }
  }

  function getClientName(clientId: string): string {
    const client = $clients.find(c => c.id === clientId);
    return client ? client.name : 'Unknown Client';
  }
</script>

<div class="page">
  <div class="header">
    <h1>Properties</h1>
    <button class="btn-primary" on:click={() => showForm = !showForm}>
      {showForm ? 'Cancel' : '+ New Property'}
    </button>
  </div>

  {#if showForm}
    <div class="card form-card">
      <h2>New Property</h2>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="client">Client *</label>
          <select id="client" bind:value={formData.clientId} required>
            <option value="">Select a client</option>
            {#each $clients as client}
              <option value={client.id}>{client.name}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="name">Property Name *</label>
          <input
            id="name"
            type="text"
            bind:value={formData.name}
            required
            placeholder="Sunset Apartments"
          />
        </div>

        <div class="form-group">
          <label for="address">Address *</label>
          <input
            id="address"
            type="text"
            bind:value={formData.address}
            required
            placeholder="123 Main Street"
          />
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 1rem;">
          <div class="form-group">
            <label for="city">City</label>
            <input
              id="city"
              type="text"
              bind:value={formData.city}
              placeholder="Springfield"
            />
          </div>

          <div class="form-group">
            <label for="state">State</label>
            <input
              id="state"
              type="text"
              bind:value={formData.state}
              placeholder="IL"
            />
          </div>

          <div class="form-group">
            <label for="zipCode">Zip Code</label>
            <input
              id="zipCode"
              type="text"
              bind:value={formData.zipCode}
              placeholder="62701"
            />
          </div>
        </div>

        <div class="form-actions">
          <button type="submit" class="btn-primary">Create Property</button>
        </div>
      </form>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">Loading properties...</div>
  {:else if $properties.length === 0}
    <div class="empty">
      <p>No properties yet. Create your first property!</p>
    </div>
  {:else}
    <div class="cards-grid">
      {#each $properties as property}
        <div class="card">
          <div class="card-header">
            <h3>{property.name}</h3>
            {#if property.sync_status === 'pending'}
              <span class="badge badge-warning">Pending</span>
            {:else}
              <span class="badge badge-success">Synced</span>
            {/if}
          </div>
          
          <div class="card-body">
            <p><strong>Client:</strong> {getClientName(property.client_id)}</p>
            <p><strong>Address:</strong> {property.address}</p>
            {#if property.city || property.state || property.zip_code}
              <p>
                {property.city || ''}{property.city && property.state ? ', ' : ''}{property.state || ''} {property.zip_code || ''}
              </p>
            {/if}
            <p class="text-muted">
              Created: {new Date(property.created_at).toLocaleDateString()}
            </p>
          </div>

          <div class="card-actions">
            <button class="btn-secondary">View Details</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1400px;
  }

  select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
    box-sizing: border-box;
    background: white;
  }

  select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .header h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    cursor: pointer;
  }

  .btn-primary:hover {
    background: #2563eb;
  }

  .btn-secondary {
    background: #6b7280;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-secondary:hover {
    background: #4b5563;
  }

  .card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
  }

  .form-card {
    margin-bottom: 2rem;
  }

  .form-card h2 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #374151;
  }

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
    box-sizing: border-box;
  }

  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .form-actions {
    margin-top: 1.5rem;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .card h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .badge-success {
    background: #d1fae5;
    color: #065f46;
  }

  .badge-warning {
    background: #fef3c7;
    color: #92400e;
  }

  .card-body p {
    margin: 0.5rem 0;
    color: #374151;
  }

  .text-muted {
    color: #6b7280 !important;
    font-size: 0.875rem;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #e5e7eb;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 3rem;
    color: #6b7280;
  }
</style>
EOF
```

---

## Step 8: Create Dashboard Page

```bash
cat > routes/+page.svelte << 'EOF'
<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';

  let stats = {
    clients: 0,
    properties: 0,
    pendingSync: 0
  };

  let isLoading = true;

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    try {
      const [clients, properties] = await Promise.all([
        tauriApi.listClients(),
        tauriApi.listProperties()
      ]);

      stats.clients = clients.length;
      stats.properties = properties.length;
      stats.pendingSync = clients.filter(c => c.sync_status === 'pending').length +
                         properties.filter(p => p.sync_status === 'pending').length;
    } catch (error) {
      console.error('Failed to load stats:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="dashboard">
  <h1>Dashboard</h1>
  
  {#if isLoading}
    <div class="loading">Loading...</div>
  {:else}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon clients">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Clients</h3>
          <p class="stat-number">{stats.clients}</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon properties">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Properties</h3>
          <p class="stat-number">{stats.properties}</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon sync">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Pending Sync</h3>
          <p class="stat-number">{stats.pendingSync}</p>
        </div>
      </div>
    </div>

    <div class="welcome-card">
      <h2>Welcome to Spectrum Tenant POC</h2>
      <p>This is an offline-first property management system with bidirectional sync.</p>
      
      <div class="features">
        <div class="feature">
          <div class="feature-icon">📱</div>
          <h3>Offline First</h3>
          <p>Work without internet, sync when connected</p>
        </div>
        
        <div class="feature">
          <div class="feature-icon">🔄</div>
          <h3>Auto Sync</h3>
          <p>Changes automatically sync with the server</p>
        </div>
        
        <div class="feature">
          <div class="feature-icon">🏢</div>
          <h3>Multi-Tenant</h3>
          <p>Isolated data for each tenant</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    max-width: 1400px;
  }

  h1 {
    margin: 0 0 2rem 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .loading {
    text-align: center;
    padding: 3rem;
    color: #6b7280;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
    display: flex;
    gap: 1rem;
  }

  .stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .stat-icon.clients {
    background: #3b82f6;
  }

  .stat-icon.properties {
    background: #10b981;
  }

  .stat-icon.sync {
    background: #f59e0b;
  }

  .stat-content h3 {
    margin: 0;
    font-size: 0.875rem;
    color: #6b7280;
    font-weight: 500;
  }

  .stat-number {
    margin: 0.25rem 0 0 0;
    font-size: 2rem;
    font-weight: 700;
    color: #111827;
  }

  .welcome-card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 2rem;
  }

  .welcome-card h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .welcome-card > p {
    margin: 0 0 2rem 0;
    color: #6b7280;
  }

  .features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 2rem;
  }

  .feature {
    text-align: center;
  }

  .feature-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .feature h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .feature p {
    margin: 0;
    color: #6b7280;
    font-size: 0.875rem;
  }
</style>
EOF
```

---

## Step 9: Test the Desktop App

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/desktop

# Build and run
cargo tauri dev
```

This will:
1. Start the SvelteKit dev server on port 5173
2. Compile the Rust code
3. Open the desktop application window

You should see:
- ✅ Dashboard with stats
- ✅ Clients page with create/list/delete
- ✅ Properties page with create/list
- ✅ Working navigation
- ✅ Data stored in local SQLite

---

## Step 10: Commit Progress

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc

git add -A
git commit -m "Phase 4: Desktop UI with clients and properties management"
git tag -a v0.4.0-desktop-ui -m "Phase 4: Desktop application UI complete"
```

---

## Phase 4 Complete! 🎉

You now have:
- ✅ Working desktop application with Tauri
- ✅ SvelteKit UI with navigation
- ✅ SQLite local database
- ✅ CRUD operations for clients and properties
- ✅ Sync status indicators
- ✅ Responsive dashboard

**Next Steps**: Phase 5 will implement the actual sync mechanism between desktop and backend!

Would you like to proceed to **Phase 5: Sync Implementation**?