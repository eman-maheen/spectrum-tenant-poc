# Phase 5: Sync Implementation - Step by Step

## Overview

In this phase, we'll implement:
1. Sync service in desktop app
2. Pull mechanism (server → client)
3. Push mechanism (client → server)
4. Conflict resolution
5. Auto-sync on connection
6. Sync status UI

---

## Step 1: Update Desktop Sync Module

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/desktop/src

cat > sync.rs << 'EOF'
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const API_BASE_URL: &str = "http://localhost:3000/api";
const TENANT_ID: &str = "00000000-0000-0000-0000-000000000001"; // Demo tenant

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    pub api_url: String,
    pub tenant_id: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            api_url: API_BASE_URL.to_string(),
            tenant_id: TENANT_ID.to_string(),
        }
    }
}

pub struct SyncService {
    config: SyncConfig,
    client: reqwest::Client,
}

impl SyncService {
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
            client: reqwest::Client::new(),
        }
    }

    // Pull clients from server
    pub async fn pull_clients(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/clients", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch clients: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        // Extract data from ApiResponse wrapper
        if let Some(data) = body.get("data") {
            if let Some(clients) = data.as_array() {
                return Ok(clients.clone());
            }
        }
        
        Ok(vec![])
    }

    // Pull properties from server
    pub async fn pull_properties(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/properties", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch properties: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            if let Some(properties) = data.as_array() {
                return Ok(properties.clone());
            }
        }
        
        Ok(vec![])
    }

    // Push client to server
    pub async fn push_client(
        &self,
        name: &str,
        email: &str,
        phone: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/clients", self.config.api_url);
        
        let payload = serde_json::json!({
            "name": name,
            "email": email,
            "phone": phone,
        });

        let response = self.client
            .post(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create client: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            return Ok(data.clone());
        }
        
        anyhow::bail!("No data in response")
    }

    // Push property to server
    pub async fn push_property(
        &self,
        client_id: &str,
        name: &str,
        address: &str,
        city: Option<&str>,
        state: Option<&str>,
        zip_code: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/properties", self.config.api_url);
        
        let payload = serde_json::json!({
            "client_id": client_id,
            "name": name,
            "address": address,
            "city": city,
            "state": state,
            "zip_code": zip_code,
        });

        let response = self.client
            .post(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create property: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            return Ok(data.clone());
        }
        
        anyhow::bail!("No data in response")
    }

    // Delete client on server
    pub async fn delete_client(&self, id: &str) -> Result<()> {
        let url = format!("{}/clients/{}", self.config.api_url, id);
        
        let response = self.client
            .delete(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete client: {}", response.status());
        }

        Ok(())
    }

    // Check if server is reachable
    pub async fn check_connection(&self) -> bool {
        let url = "http://localhost:3000/health";
        
        match self.client
            .get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}
EOF
```

---

## Step 2: Add Sync Commands to Tauri

```bash
cat > commands.rs << 'EOF'
use tauri::State;
use std::sync::Mutex;
use crate::db::Database;
use crate::sync::SyncService;

pub struct AppState {
    pub db: Mutex<Database>,
    pub sync: Mutex<SyncService>,
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
pub async fn create_client(
    state: State<'_, AppState>,
    name: String,
    email: String,
    phone: Option<String>,
) -> Result<String, String> {
    // First, try to create on server
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    
    match sync.push_client(&name, &email, phone.as_deref()).await {
        Ok(server_data) => {
            // Server creation succeeded, save to local DB
            drop(sync); // Release sync lock
            
            let db = state.db.lock().map_err(|e| e.to_string())?;
            
            // Extract server-generated ID
            let server_id = server_data.get("id")
                .and_then(|v| v.as_str())
                .ok_or("No ID from server")?;
            
            // Save with server ID
            db.conn.execute(
                "INSERT OR REPLACE INTO clients (id, name, email, phone, created_at, updated_at, version, _sync_status) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'synced')",
                rusqlite::params![
                    server_id,
                    &name,
                    &email,
                    &phone,
                    server_data.get("created_at").and_then(|v| v.as_str()).unwrap_or(""),
                    server_data.get("updated_at").and_then(|v| v.as_str()).unwrap_or(""),
                    server_data.get("version").and_then(|v| v.as_i64()).unwrap_or(1)
                ],
            ).map_err(|e| e.to_string())?;
            
            Ok(server_id.to_string())
        }
        Err(_) => {
            // Server failed, save locally with pending status
            drop(sync);
            
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let id = db.create_client(&name, &email, phone.as_deref())
                .map_err(|e| e.to_string())?;
            Ok(id)
        }
    }
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
pub async fn delete_client(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Try to delete on server first
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    
    let _ = sync.delete_client(&id).await; // Ignore server errors
    drop(sync);
    
    // Always delete locally
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
pub async fn create_property(
    state: State<'_, AppState>,
    client_id: String,
    name: String,
    address: String,
    city: Option<String>,
    state_val: Option<String>,
    zip_code: Option<String>,
) -> Result<String, String> {
    // Try to create on server
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    
    match sync.push_property(
        &client_id,
        &name,
        &address,
        city.as_deref(),
        state_val.as_deref(),
        zip_code.as_deref(),
    ).await {
        Ok(server_data) => {
            drop(sync);
            
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let server_id = server_data.get("id")
                .and_then(|v| v.as_str())
                .ok_or("No ID from server")?;
            
            db.conn.execute(
                "INSERT OR REPLACE INTO properties (id, client_id, name, address, city, state, zip_code, created_at, updated_at, version, _sync_status) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'synced')",
                rusqlite::params![
                    server_id,
                    &client_id,
                    &name,
                    &address,
                    &city,
                    &state_val,
                    &zip_code,
                    server_data.get("created_at").and_then(|v| v.as_str()).unwrap_or(""),
                    server_data.get("updated_at").and_then(|v| v.as_str()).unwrap_or(""),
                    server_data.get("version").and_then(|v| v.as_i64()).unwrap_or(1)
                ],
            ).map_err(|e| e.to_string())?;
            
            Ok(server_id.to_string())
        }
        Err(_) => {
            drop(sync);
            
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
    }
}

// Sync commands
#[tauri::command]
pub async fn sync_pull(state: State<'_, AppState>) -> Result<String, String> {
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    
    // Fetch from server
    let clients = sync.pull_clients().await.map_err(|e| e.to_string())?;
    let properties = sync.pull_properties().await.map_err(|e| e.to_string())?;
    
    drop(sync);
    
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // Upsert clients
    for client in clients {
        let id = client.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = client.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let email = client.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let phone = client.get("phone").and_then(|v| v.as_str());
        let created_at = client.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        let updated_at = client.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let version = client.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
        
        db.conn.execute(
            "INSERT OR REPLACE INTO clients (id, name, email, phone, created_at, updated_at, version, is_deleted, _sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'synced')",
            rusqlite::params![id, name, email, phone, created_at, updated_at, version],
        ).map_err(|e| e.to_string())?;
    }
    
    // Upsert properties
    for property in properties {
        let id = property.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let client_id = property.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
        let name = property.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let address = property.get("address").and_then(|v| v.as_str()).unwrap_or("");
        let city = property.get("city").and_then(|v| v.as_str());
        let state_val = property.get("state").and_then(|v| v.as_str());
        let zip_code = property.get("zip_code").and_then(|v| v.as_str());
        let created_at = property.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        let updated_at = property.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
        let version = property.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
        
        db.conn.execute(
            "INSERT OR REPLACE INTO properties (id, client_id, name, address, city, state, zip_code, created_at, updated_at, version, is_deleted, _sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, 'synced')",
            rusqlite::params![id, client_id, name, address, city, state_val, zip_code, created_at, updated_at, version],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok("Sync completed successfully".to_string())
}

#[tauri::command]
pub async fn check_server_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    Ok(sync.check_connection().await)
}
EOF
```

---

## Step 3: Update main.rs with Sync Service

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
use sync::SyncService;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db = tauri::async_runtime::block_on(async {
                db::init_db().await
            }).expect("Failed to initialize database");
            
            // Initialize sync service
            let sync = SyncService::new();
            
            // Store in app state
            app.manage(AppState {
                db: Mutex::new(db),
                sync: Mutex::new(sync),
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
            commands::sync_pull,
            commands::check_server_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

---

## Step 4: Add Sync UI Component to Frontend

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/frontend/src

# Update API wrapper with sync
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
  },

  // Sync operations
  async syncPull(): Promise<string> {
    return await invoke<string>('sync_pull');
  },

  async checkConnection(): Promise<boolean> {
    return await invoke<boolean>('check_server_connection');
  }
};
EOF

# Create sync component
mkdir -p lib/components
cat > lib/components/SyncButton.svelte << 'EOF'
<script lang="ts">
  import { tauriApi } from '$lib/api/tauri';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let isSyncing = false;
  let isOnline = false;
  let lastSyncTime: Date | null = null;

  async function checkConnection() {
    isOnline = await tauriApi.checkConnection();
  }

  async function handleSync() {
    if (isSyncing) return;

    isSyncing = true;
    try {
      const result = await tauriApi.syncPull();
      lastSyncTime = new Date();
      dispatch('synced');
      alert(result);
    } catch (error) {
      console.error('Sync failed:', error);
      alert('Sync failed: ' + error);
    } finally {
      isSyncing = false;
    }
  }

  // Check connection on mount
  checkConnection();
  
  // Check connection every 30 seconds
  setInterval(checkConnection, 30000);
</script>

<div class="sync-controls">
  <div class="connection-status">
    <div class="status-dot" class:online={isOnline} class:offline={!isOnline}></div>
    <span>{isOnline ? 'Online' : 'Offline'}</span>
  </div>

  <button 
    class="sync-button"
    on:click={handleSync}
    disabled={isSyncing || !isOnline}
  >
    {#if isSyncing}
      <span class="spinner"></span>
      Syncing...
    {:else}
      🔄 Sync Now
    {/if}
  </button>

  {#if lastSyncTime}
    <div class="last-sync">
      Last synced: {lastSyncTime.toLocaleTimeString()}
    </div>
  {/if}
</div>

<style>
  .sync-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #6b7280;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-dot.online {
    background: #10b981;
    box-shadow: 0 0 8px #10b981;
  }

  .status-dot.offline {
    background: #ef4444;
  }

  .sync-button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: background 0.2s;
  }

  .sync-button:hover:not(:disabled) {
    background: #2563eb;
  }

  .sync-button:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .last-sync {
    font-size: 0.75rem;
    color: #9ca3af;
  }
</style>
EOF
```

---

## Step 5: Update Layout with Sync Button

```bash
cat > routes/+layout.svelte << 'EOF'
<script lang="ts">
  import { page } from '$app/stores';
  import SyncButton from '$lib/components/SyncButton.svelte';
  
  const navItems = [
    { href: '/', label: 'Dashboard' },
    { href: '/clients', label: 'Clients' },
    { href: '/properties', label: 'Properties' },
    { href: '/buildings', label: 'Buildings' },
    { href: '/units', label: 'Units' },
  ];

  function handleSynced() {
    // Reload current page data
    window.location.reload();
  }
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
    
    <div class="sidebar-footer">
      <SyncButton on:synced={handleSynced} />
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
    border-top: 1px solid #333;
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

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid #333;
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

## Step 6: Test the Sync

### 1. Make sure backend is running:
```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc/backend
cargo run
```

### 2. Restart desktop app:
```bash
# Terminal 1 (from frontend dir)
npm run dev

# Terminal 2 (from desktop dir)
cargo tauri dev
```

### 3. Click "Sync Now" button in the sidebar

You should see:
- ✅ Demo data from PostgreSQL appear in your desktop app
- ✅ 2 clients (John Doe, Jane Smith)
- ✅ 1 property (Sunset Apartments)
- ✅ Connection status shows "Online"

---

## Step 7: Test Create with Auto-Sync

1. Create a new client in the desktop app
2. It should immediately sync to the backend
3. Refresh the backend test script to see it there

```bash
# Check backend has the new client
curl -H "x-tenant-id: 00000000-0000-0000-0000-000000000001" \
  http://localhost:3000/api/clients | jq
```

---

## Step 8: Commit Progress

```bash
cd /home/eman/Downloads/Qt/spectrum-tenant-poc

git add -A
git commit -m "Phase 5: Sync implementation with push/pull and connection detection"
git tag -a v0.