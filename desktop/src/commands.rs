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
