// desktop/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex; // Use tokio's Mutex
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
            
            // Store in app state with Arc for cloning
            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                sync: Arc::new(Mutex::new(sync)),
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            // Client operations
            commands::list_clients,
            commands::get_client,
            commands::create_client,
            commands::update_client,
            commands::delete_client,
            // Property operations
            commands::list_properties,
            commands::create_property,
            commands::delete_property,
            // Sync operations
            commands::sync_bidirectional,
            commands::check_server_connection,
            commands::get_pending_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}