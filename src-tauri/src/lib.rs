mod commands;
mod db;
mod models;
mod steam;

use commands::AppDb;
use db::Database;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Database::open().expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppDb(Mutex::new(database)))
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::set_app_mode,
            commands::steam_logout,
            commands::steam_login,
            commands::steam_login_dev,
            commands::list_leagues,
            commands::create_league,
            commands::list_my_invites,
            commands::get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
