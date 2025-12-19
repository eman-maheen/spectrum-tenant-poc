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
