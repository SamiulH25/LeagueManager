use crate::db::Database;
use crate::league_api::{fetch_current_event, fetch_health, fetch_standings, start_for_host, LeagueApiManager};
use crate::models::{
    AppState, CurrentEvent, DriverProfile, HostSettings, ImportResult, LeagueApiStatus, LeagueInvite,
    LeagueSummary, PathSuggestions, PitLinkTestResult, RaceLaunchConfig, ResultsFeed, ServerStatus,
    StandingsResponse,
};
use crate::results_watcher::{results_dir_for_server, ResultsWatcher};
use crate::server::{self, ServerManager};
use crate::steam::{
    build_openid_login_url, fetch_player_profile, parse_query_string, verify_openid_response,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct AppDb(pub Arc<Mutex<Database>>);
pub struct AppServer(pub Arc<ServerManager>);
pub struct AppApi(pub LeagueApiManager);
pub struct AppResultsWatcher(pub ResultsWatcher);

fn steam_api_key() -> Option<String> {
    std::env::var("LEAGUE_MANAGER_STEAM_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

fn sync_league_api(
    api: &LeagueApiManager,
    db: &Arc<Mutex<Database>>,
    server: &Arc<ServerManager>,
    mode: Option<&str>,
) -> Result<(), String> {
    if mode == Some("host") {
        start_for_host(api, db, server)
    } else {
        api.stop();
        Ok(())
    }
}

#[tauri::command]
pub fn get_app_state(db: State<'_, AppDb>) -> Result<AppState, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_app_state()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_mode(
    db: State<'_, AppDb>,
    api: State<'_, AppApi>,
    server: State<'_, AppServer>,
    mode: String,
) -> Result<AppState, String> {
    let state = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database.set_app_mode(&mode).map_err(|e| e.to_string())?;
        database.get_app_state().map_err(|e| e.to_string())?
    };
    sync_league_api(&api.0, &db.0, &server.0, state.app_mode.as_deref())?;
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
pub fn save_host_settings(
    db: State<'_, AppDb>,
    api: State<'_, AppApi>,
    server: State<'_, AppServer>,
    settings: HostSettings,
) -> Result<(), String> {
    let mode = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database
            .save_host_settings(&settings)
            .map_err(|e| e.to_string())?;
        database
            .get_app_state()
            .map_err(|e| e.to_string())?
            .app_mode
    };
    if mode.as_deref() == Some("host") {
        start_for_host(&api.0, &db.0, &server.0)?;
    }
    Ok(())
}

#[tauri::command]
pub fn start_race_server(
    db: State<'_, AppDb>,
    server: State<'_, AppServer>,
    watcher: State<'_, AppResultsWatcher>,
    config: RaceLaunchConfig,
) -> Result<(), String> {
    let (settings, event_name, track) = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        let settings = database.get_host_settings().map_err(|e| e.to_string())?;
        let event_name = config.server_name.clone();
        let track = config.track.clone();
        (settings, event_name, track)
    };

    server
        .0
        .start(&settings, &config)
        .map_err(|e| e.to_string())?;

    {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database
            .begin_active_event(&event_name, &track)
            .map_err(|e| e.to_string())?;
    }

    let results_dir = results_dir_for_server(
        &server
            .0
            .server_root()
            .map_err(|e| e.to_string())?,
    );
    watcher.0.start(results_dir, Arc::clone(&db.0));

    Ok(())
}

#[tauri::command]
pub fn stop_race_server(
    db: State<'_, AppDb>,
    server: State<'_, AppServer>,
    watcher: State<'_, AppResultsWatcher>,
) -> Result<(), String> {
    watcher.0.stop();
    server.0.stop().map_err(|e| e.to_string())?;
    db.0.lock()
        .map_err(|e| e.to_string())?
        .complete_active_event()
        .map_err(|e| e.to_string())
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

#[tauri::command]
pub fn get_league_api_status(db: State<'_, AppDb>, api: State<'_, AppApi>) -> Result<LeagueApiStatus, String> {
    let port = db
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())?
        .sync_port;
    Ok(LeagueApiStatus {
        running: api.0.is_running(),
        port,
    })
}

#[tauri::command]
pub async fn test_pit_link(host: String, port: u16) -> Result<PitLinkTestResult, String> {
    match fetch_health(&host, port).await {
        Ok((health, latency_ms)) => Ok(PitLinkTestResult {
            connected: true,
            latency_ms,
            version: Some(health.version),
            message: "Connected to host pit link".to_string(),
        }),
        Err(e) => Ok(PitLinkTestResult {
            connected: false,
            latency_ms: 0,
            version: None,
            message: e,
        }),
    }
}

#[tauri::command]
pub async fn fetch_remote_current_event(
    host: String,
    port: u16,
) -> Result<CurrentEvent, String> {
    fetch_current_event(&host, port).await
}

#[tauri::command]
pub async fn fetch_remote_standings(
    host: String,
    port: u16,
    championship_id: i64,
) -> Result<StandingsResponse, String> {
    fetch_standings(&host, port, championship_id).await
}

#[tauri::command]
pub async fn open_remote_cm_join_link(
    app: AppHandle,
    host: String,
    port: u16,
) -> Result<String, String> {
    let event = fetch_current_event(&host, port).await?;
    let link = event
        .cm_join_link
        .ok_or_else(|| "No live race or join link unavailable".to_string())?;
    app.opener()
        .open_url(&link, None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(link)
}

#[tauri::command]
pub fn get_results_feed(
    db: State<'_, AppDb>,
    watcher: State<'_, AppResultsWatcher>,
) -> Result<ResultsFeed, String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .get_results_feed(watcher.0.is_running())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_results_json(
    db: State<'_, AppDb>,
    json: String,
    file_name: Option<String>,
) -> Result<ImportResult, String> {
    let name = file_name.unwrap_or_else(|| "manual_import.json".to_string());
    db.0.lock()
        .map_err(|e| e.to_string())?
        .import_results_file(&name, &json, "manual")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dismiss_results_warning(db: State<'_, AppDb>, warning_id: i64) -> Result<(), String> {
    db.0.lock()
        .map_err(|e| e.to_string())?
        .dismiss_results_warning(warning_id)
        .map_err(|e| e.to_string())
}
