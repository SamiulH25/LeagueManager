mod commands;
mod db;
mod league_api;
mod models;
mod points;
mod results;
mod results_watcher;
mod server;
mod steam;

use commands::{AppApi, AppDb, AppResultsWatcher, AppServer};
use db::Database;
use league_api::{start_for_host, LeagueApiManager};
use results_watcher::ResultsWatcher;
use server::ServerManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Arc::new(Mutex::new(Database::open().expect("failed to open database")));
    let server = Arc::new(ServerManager::new());
    let api = LeagueApiManager::new();
    let watcher = ResultsWatcher::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppDb(Arc::clone(&database)))
        .manage(AppServer(Arc::clone(&server)))
        .manage(AppApi(api))
        .manage(AppResultsWatcher(watcher))
        .setup({
            let database = Arc::clone(&database);
            let server = Arc::clone(&server);
            move |app| {
                let db = app.state::<AppDb>();
                let api = app.state::<AppApi>();
                if let Ok(state) = db.0.lock() {
                    if state.get_app_state().ok().and_then(|s| s.app_mode).as_deref()
                        == Some("host")
                    {
                        let _ = start_for_host(&api.0, &database, &server);
                    }
                }
                Ok(())
            }
        })
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
            commands::get_league_api_status,
            commands::test_pit_link,
            commands::fetch_remote_current_event,
            commands::fetch_remote_standings,
            commands::open_remote_cm_join_link,
            commands::get_results_feed,
            commands::import_results_json,
            commands::dismiss_results_warning,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
