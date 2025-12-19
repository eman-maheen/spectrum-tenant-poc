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
