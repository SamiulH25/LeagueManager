use crate::db::Database;
use crate::models::{CurrentEvent, HealthResponse, StandingsResponse};
use crate::server::ServerManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Mutex<Database>>,
    pub server: Arc<ServerManager>,
}

pub struct LeagueApiManager {
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    port: Mutex<u16>,
}

impl LeagueApiManager {
    pub fn new() -> Self {
        Self {
            shutdown: Mutex::new(None),
            handle: Mutex::new(None),
            port: Mutex::new(0),
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle
            .lock()
            .map(|h| h.is_some())
            .unwrap_or(false)
    }

    pub fn current_port(&self) -> u16 {
        *self.port.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn start(&self, port: u16, state: ApiState) -> Result<(), String> {
        self.stop();

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = std::net::TcpListener::bind(addr).map_err(|e| {
            format!("failed to bind league API on port {port}: {e}")
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|e| e.to_string())?;

        let (tx, rx) = oneshot::channel();
        let router = build_router(state);

        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("tcp listener");
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .ok();
        });

        *self.shutdown.lock().map_err(|e| e.to_string())? = Some(tx);
        *self.handle.lock().map_err(|e| e.to_string())? = Some(handle);
        *self.port.lock().map_err(|e| e.to_string())? = port;
        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.shutdown.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }
        if let Ok(mut guard) = self.handle.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
    }
}

fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/events/current", get(events_current))
        .route(
            "/api/championships/{id}/standings",
            get(championship_standings),
        )
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn events_current(State(state): State<ApiState>) -> Result<Json<CurrentEvent>, StatusCode> {
    let settings = {
        let db = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        db.get_host_settings()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let status = state
        .server
        .status(&settings)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = if status.running {
        CurrentEvent {
            status: "live".to_string(),
            server_name: status.server_name,
            track: status.info.as_ref().map(|i| i.track.clone()),
            public_ip: status.public_ip,
            http_port: Some(status.http_port),
            game_port: Some(status.game_port),
            password: None,
            cm_join_link: status.cm_join_link,
            clients: status.info.as_ref().map(|i| i.clients),
            max_clients: status.info.as_ref().map(|i| i.max_clients),
            time_left: status.info.as_ref().map(|i| i.time_left),
        }
    } else {
        CurrentEvent {
            status: "idle".to_string(),
            server_name: None,
            track: None,
            public_ip: status.public_ip,
            http_port: Some(status.http_port),
            game_port: Some(status.game_port),
            password: None,
            cm_join_link: None,
            clients: None,
            max_clients: None,
            time_left: None,
        }
    };

    Ok(Json(event))
}

async fn championship_standings(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<StandingsResponse>, StatusCode> {
    let standings = {
        let db = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        db.get_championship_standings(id)
            .map_err(|_| StatusCode::NOT_FOUND)?
    };
    Ok(Json(standings))
}

pub async fn fetch_health(host: &str, port: u16) -> Result<(HealthResponse, u64), String> {
    let url = format!("http://{host}:{port}/api/health");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let start = std::time::Instant::now();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("connection failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("host returned HTTP {}", resp.status()));
    }
    let health: HealthResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok((health, start.elapsed().as_millis() as u64))
}

pub async fn fetch_current_event(host: &str, port: u16) -> Result<CurrentEvent, String> {
    let url = format!("http://{host}:{port}/api/events/current");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("connection failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("host returned HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn fetch_standings(
    host: &str,
    port: u16,
    championship_id: i64,
) -> Result<StandingsResponse, String> {
    let url = format!("http://{host}:{port}/api/championships/{championship_id}/standings");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("connection failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("host returned HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub fn start_for_host(
    api: &LeagueApiManager,
    db: &Arc<Mutex<Database>>,
    server: &Arc<ServerManager>,
) -> Result<(), String> {
    let settings = db
        .lock()
        .map_err(|e| e.to_string())?
        .get_host_settings()
        .map_err(|e| e.to_string())?;

    let port = settings.sync_port;
    if api.is_running() && api.current_port() == port {
        return Ok(());
    }

    let state = ApiState {
        db: Arc::clone(db),
        server: Arc::clone(server),
    };
    api.start(port, state)
}
