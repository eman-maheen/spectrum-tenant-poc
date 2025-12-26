// desktop/src/commands.rs - Using tokio::sync::Mutex (Send-safe)

use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex; // This is the key change!
use crate::db::Database;
use crate::sync::SyncService;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub sync: Arc<Mutex<SyncService>>,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Spectrum Tenant POC.", name)
}

// ========================================
// CLIENT COMMANDS
// ========================================

#[tauri::command]
pub async fn list_clients(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().await;
    let clients = db.list_clients().map_err(|e| e.to_string())?;
    serde_json::to_string(&clients).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_client(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let db = state.db.lock().await;
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
    // Check if online
    let is_online = {
        let sync = state.sync.lock().await;
        sync.check_connection().await
    };
    
    if is_online {
        // Try to create on server first
        let server_result = {
            let sync = state.sync.lock().await;
            sync.push_client(&name, &email, phone.as_deref()).await
        };
        
        match server_result {
            Ok(server_data) => {
                let db = state.db.lock().await;
                let server_id = server_data["id"].as_str().ok_or("No ID from server")?;
                
                db.conn.execute(
                    "INSERT INTO clients (id, name, email, phone, created_at, updated_at, version, _sync_status, is_deleted) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'synced', 0)",
                    rusqlite::params![
                        server_id,
                        &name,
                        &email,
                        &phone,
                        server_data["created_at"].as_str().unwrap_or(""),
                        server_data["updated_at"].as_str().unwrap_or(""),
                        server_data["version"].as_i64().unwrap_or(1)
                    ],
                ).map_err(|e| e.to_string())?;
                
                return Ok(server_id.to_string());
            }
            Err(_) => {
                // Fall through to offline mode
            }
        }
    }
    
    // Offline or server failed
    let db = state.db.lock().await;
    let id = db.create_client(&name, &email, phone.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_client(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.update_client(&id, name.as_deref(), email.as_deref(), phone.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_client(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Soft delete locally
    {
        let db = state.db.lock().await;
        db.delete_client(&id).map_err(|e| e.to_string())?;
    }
    
    // Check if online
    let is_online = {
        let sync = state.sync.lock().await;
        sync.check_connection().await
    };
    
    if is_online {
        let server_result = {
            let sync = state.sync.lock().await;
            sync.delete_client(&id).await
        };
        
        let sync_status = if server_result.is_ok() { "synced" } else { "pending" };
        
        let db = state.db.lock().await;
        db.conn.execute(
            "UPDATE clients SET _sync_status = ?1 WHERE id = ?2",
            [sync_status, &id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ========================================
// PROPERTY COMMANDS
// ========================================

#[tauri::command]
pub async fn list_properties(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().await;
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
    let is_online = {
        let sync = state.sync.lock().await;
        sync.check_connection().await
    };
    
    if is_online {
        let server_result = {
            let sync = state.sync.lock().await;
            sync.push_property(
                &client_id,
                &name,
                &address,
                city.as_deref(),
                state_val.as_deref(),
                zip_code.as_deref(),
            ).await
        };
        
        match server_result {
            Ok(server_data) => {
                let db = state.db.lock().await;
                let server_id = server_data["id"].as_str().ok_or("No ID from server")?;
                
                db.conn.execute(
                    "INSERT INTO properties (id, client_id, name, address, city, state, zip_code, created_at, updated_at, version, _sync_status, is_deleted) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'synced', 0)",
                    rusqlite::params![
                        server_id,
                        &client_id,
                        &name,
                        &address,
                        &city,
                        &state_val,
                        &zip_code,
                        server_data["created_at"].as_str().unwrap_or(""),
                        server_data["updated_at"].as_str().unwrap_or(""),
                        server_data["version"].as_i64().unwrap_or(1)
                    ],
                ).map_err(|e| e.to_string())?;
                
                return Ok(server_id.to_string());
            }
            Err(_) => {}
        }
    }
    
    let db = state.db.lock().await;
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

#[tauri::command]
pub async fn delete_property(state: State<'_, AppState>, id: String) -> Result<(), String> {
    {
        let db = state.db.lock().await;
        db.conn.execute(
            "UPDATE properties SET is_deleted = 1, _sync_status = 'pending' WHERE id = ?1",
            [&id],
        ).map_err(|e| e.to_string())?;
    }
    
    let is_online = {
        let sync = state.sync.lock().await;
        sync.check_connection().await
    };
    
    if is_online {
        let server_result = {
            let sync = state.sync.lock().await;
            sync.delete_property(&id).await
        };
        
        let sync_status = if server_result.is_ok() { "synced" } else { "pending" };
        
        let db = state.db.lock().await;
        db.conn.execute(
            "UPDATE properties SET _sync_status = ?1 WHERE id = ?2",
            [sync_status, &id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ========================================
// SYNC COMMANDS
// ========================================

#[tauri::command]
pub async fn sync_bidirectional(state: State<'_, AppState>) -> Result<String, String> {
    // Clone Arc pointers to move into blocking task
    let sync_arc = state.sync.clone();
    let db_arc = state.db.clone();
    
    // Run sync in a blocking thread since rusqlite::Connection is not Send
    let result = tokio::task::spawn_blocking(move || {
        // Use block_on to run async code in blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let sync = sync_arc.lock().await;
            let db = db_arc.lock().await;
            
            sync.sync_bidirectional(&db.conn)
                .await
                .map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
pub async fn check_server_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let sync = state.sync.lock().await;
    Ok(sync.check_connection().await)
}

// ========================================
// UTILITY COMMANDS
// ========================================

#[tauri::command]
pub async fn get_pending_count(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state.db.lock().await;
    
    let count: usize = db.conn.query_row(
        "SELECT 
            (SELECT COUNT(*) FROM clients WHERE _sync_status = 'pending') +
            (SELECT COUNT(*) FROM properties WHERE _sync_status = 'pending')",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    Ok(count)
}