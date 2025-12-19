// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod sync;
mod commands;

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Initialize local database
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    if let Err(e) = db::init_db().await {
                        eprintln!("Failed to initialize database: {}", e);
                    }
                });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
