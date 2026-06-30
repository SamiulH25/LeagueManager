use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverProfile {
    pub steam_id64: String,
    pub personaname: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub onboarding_complete: bool,
    pub app_mode: Option<String>,
    pub session: Option<DriverProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueSummary {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueInvite {
    pub id: i64,
    pub league_id: i64,
    pub league_name: String,
    pub status: String,
    pub invited_at: String,
    pub host_personaname: Option<String>,
    pub host_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSettings {
    pub assetto_server_path: String,
    pub ac_install_path: String,
    pub http_port: u16,
    pub game_port: u16,
    pub admin_password: String,
    pub public_ip_override: String,
    pub sync_port: u16,
}

impl Default for HostSettings {
    fn default() -> Self {
        Self {
            assetto_server_path: String::new(),
            ac_install_path: String::new(),
            http_port: 8081,
            game_port: 9600,
            admin_password: "leaguemgr".to_string(),
            public_ip_override: String::new(),
            sync_port: 9847,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSuggestions {
    pub assetto_server_path: Option<String>,
    pub ac_install_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceLaunchConfig {
    pub server_name: String,
    pub track: String,
    #[serde(default)]
    pub track_config: String,
    pub cars: Vec<String>,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub practice_minutes: u32,
    #[serde(default = "default_qualify")]
    pub qualify_minutes: u32,
    #[serde(default = "default_race")]
    pub race_minutes: u32,
    #[serde(default = "default_max_clients")]
    pub max_clients: u32,
    #[serde(default)]
    pub ai_slots: u32,
    #[serde(default)]
    pub mod_urls: Vec<String>,
}

fn default_qualify() -> u32 {
    15
}
fn default_race() -> u32 {
    20
}
fn default_max_clients() -> u32 {
    16
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub track: String,
    pub clients: u32,
    pub max_clients: u32,
    pub session: u32,
    pub time_left: u32,
    pub port: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub running: bool,
    pub public_ip: Option<String>,
    pub http_port: u16,
    pub game_port: u16,
    pub cm_join_link: Option<String>,
    pub server_name: Option<String>,
    pub info: Option<ServerInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentEvent {
    pub status: String,
    pub server_name: Option<String>,
    pub track: Option<String>,
    pub public_ip: Option<String>,
    pub http_port: Option<u16>,
    pub game_port: Option<u16>,
    pub password: Option<String>,
    pub cm_join_link: Option<String>,
    pub clients: Option<u32>,
    pub max_clients: Option<u32>,
    pub time_left: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandingRow {
    pub position: u32,
    pub driver_name: String,
    pub team: Option<String>,
    pub points: u32,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandingsResponse {
    pub championship_id: i64,
    pub championship_name: String,
    pub rows: Vec<StandingRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitLinkTestResult {
    pub connected: bool,
    pub latency_ms: u64,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueApiStatus {
    pub running: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedResultEntry {
    pub driver_name: String,
    pub driver_guid: Option<String>,
    pub car_model: Option<String>,
    pub position: Option<u32>,
    pub best_lap_ms: Option<i64>,
    pub laps: u32,
    pub total_time_ms: Option<i64>,
    pub dnf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub success: bool,
    pub session_type: String,
    pub track: String,
    pub entries_imported: u32,
    pub points_awarded: u32,
    pub message: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResultSummary {
    pub id: i64,
    pub session_type: String,
    pub track: String,
    pub source: String,
    pub file_name: Option<String>,
    pub imported_at: String,
    pub entry_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsWarning {
    pub id: i64,
    pub message: String,
    pub file_name: Option<String>,
    pub created_at: String,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsFeed {
    pub warnings: Vec<ResultsWarning>,
    pub recent: Vec<SessionResultSummary>,
    pub watcher_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueMember {
    pub driver_id: i64,
    pub steam_id64: String,
    pub personaname: String,
    pub avatar_url: String,
    pub team: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingLeagueInvite {
    pub id: i64,
    pub steam_id64: String,
    pub personaname: String,
    pub avatar_url: String,
    pub invited_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueRoster {
    pub members: Vec<LeagueMember>,
    pub pending_invites: Vec<PendingLeagueInvite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveLeague {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverLeague {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionshipRound {
    pub id: i64,
    pub championship_id: i64,
    pub round_number: u32,
    pub name: String,
    pub track: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}
