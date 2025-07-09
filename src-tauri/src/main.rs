#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, Manager}; // ← ini penting: import Manager
use std::fs;

#[command]
fn get_products() -> Result<String, String> {
    match fs::read_to_string("product.json") {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Failed to read product: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_products])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                // ✅ Cara resmi buka DevTools di Tauri v2
                if let Some(main_window) = app.get_webview_window("main") {
                    main_window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
