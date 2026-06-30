use crate::db::Database;
use crate::models::{
    AppState, DriverProfile, HostSettings, LeagueInvite, LeagueSummary, PathSuggestions,
    RaceLaunchConfig, ServerStatus,
};
use crate::server::{self, ServerManager};
use crate::steam::{
    build_openid_login_url, fetch_player_profile, parse_query_string, verify_openid_response,
};
use std::sync::Mutex;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct AppDb(pub Mutex<Database>);
pub struct AppServer(pub ServerManager);

fn steam_api_key() -> Option<String> {
    std::env::var("LEAGUE_MANAGER_STEAM_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

#[tauri::command]
pub fn get_app_state(db: State<'_, AppDb>) -> Result<AppState, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_app_state()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_mode(db: State<'_, AppDb>, mode: String) -> Result<AppState, String> {
    let state = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database.set_app_mode(&mode).map_err(|e| e.to_string())?;
        database.get_app_state().map_err(|e| e.to_string())?
    };
    Ok(state)
}

#[tauri::command]
pub fn steam_logout(db: State<'_, AppDb>) -> Result<AppState, String> {
    let state = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database.clear_session().map_err(|e| e.to_string())?;
        database.get_app_state().map_err(|e| e.to_string())?
    };
    Ok(state)
}

#[tauri::command]
pub async fn steam_login_dev(
    db: State<'_, AppDb>,
    steam_id64: String,
) -> Result<DriverProfile, String> {
    let profile = fetch_player_profile(&steam_id64, steam_api_key().as_deref())
        .await
        .map_err(|e| e.to_string())?;
    db.0.lock()
        .map_err(|e| e.to_string())?
        .login_driver(profile)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn steam_login(app: AppHandle, db: State<'_, AppDb>) -> Result<DriverProfile, String> {
    let port = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(45100..=45200)
    };
    let return_to = format!("http://127.0.0.1:{port}/auth/steam/callback");
    let login_url = build_openid_login_url(&return_to);

    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .map_err(|e| format!("failed to bind auth port: {e}"))?;

    app.opener()
        .open_url(&login_url, None::<&str>)
        .map_err(|e| format!("failed to open browser: {e}"))?;

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("auth callback failed: {e}"))?;

    let mut buffer = vec![0u8; 8192];
    let n = stream
        .read(&mut buffer)
        .await
        .map_err(|e| format!("read callback: {e}"))?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");
    let query = path.split('?').nth(1).unwrap_or("");
    let params = parse_query_string(query);

    let steam_id = verify_openid_response(&params)
        .await
        .map_err(|e| e.to_string())?;

    let body = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n\
        <html><body style='background:#0a0a0c;color:#fff;font-family:sans-serif;text-align:center;padding:4rem'>\
        <h1>Login successful</h1><p>You can close this tab and return to LeagueManager.</p></body></html>";
    stream
        .write_all(body.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    let _ = stream.shutdown().await;

    let profile = fetch_player_profile(&steam_id, steam_api_key().as_deref())
        .await
        .map_err(|e| e.to_string())?;

    db.0.lock()
        .map_err(|e| e.to_string())?
        .login_driver(profile)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_leagues(db: State<'_, AppDb>) -> Result<Vec<LeagueSummary>, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .list_leagues()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_league(db: State<'_, AppDb>, name: String) -> Result<LeagueSummary, String> {
    let host_steam_id = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        let state = database.get_app_state().map_err(|e| e.to_string())?;
        state
            .session
            .ok_or_else(|| "sign in with steam first".to_string())?
            .steam_id64
    };
    db.0.lock()
        .map_err(|e| e.to_string())?
        .create_league(&name, &host_steam_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_my_invites(db: State<'_, AppDb>) -> Result<Vec<LeagueInvite>, String> {
    let steam_id = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        let state = database.get_app_state().map_err(|e| e.to_string())?;
        state
            .session
            .ok_or_else(|| "sign in with steam first".to_string())?
            .steam_id64
    };
    db.0.lock()
        .map_err(|e| e.to_string())?
        .list_invites_for_steam(&steam_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_db_path() -> String {
    crate::db::db_path().to_string_lossy().to_string()
}

#[tauri::command]
pub fn detect_paths() -> PathSuggestions {
    server::detect_path_suggestions()
}

#[tauri::command]
pub fn get_host_settings(db: State<'_, AppDb>) -> Result<HostSettings, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_host_settings(db: State<'_, AppDb>, settings: HostSettings) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .save_host_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_race_server(
    db: State<'_, AppDb>,
    server: State<'_, AppServer>,
    config: RaceLaunchConfig,
) -> Result<(), String> {
    let settings = db
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())?;
    server
        .0
        .start(&settings, &config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_race_server(server: State<'_, AppServer>) -> Result<(), String> {
    server.0.stop().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_server_status(
    db: State<'_, AppDb>,
    server: State<'_, AppServer>,
) -> Result<ServerStatus, String> {
    let settings = db
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())?;
    server
        .0
        .status(&settings)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_cm_join_link(
    app: AppHandle,
    db: State<'_, AppDb>,
    server: State<'_, AppServer>,
) -> Result<String, String> {
    let settings = db
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())?;
    let status = server
        .0
        .status(&settings)
        .await
        .map_err(|e| e.to_string())?;
    let link = status
        .cm_join_link
        .ok_or_else(|| "Server not running or public IP unavailable".to_string())?;
    app.opener()
        .open_url(&link, None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(link)
}
