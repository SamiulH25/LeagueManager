mod commands;
mod db;
mod models;
mod server;
mod steam;

use commands::{AppDb, AppServer};
use db::Database;
use server::ServerManager;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Database::open().expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppDb(Mutex::new(database)))
        .manage(AppServer(ServerManager::new()))
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
            commands::detect_paths,
            commands::get_host_settings,
            commands::save_host_settings,
            commands::start_race_server,
            commands::stop_race_server,
            commands::get_server_status,
            commands::open_cm_join_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
