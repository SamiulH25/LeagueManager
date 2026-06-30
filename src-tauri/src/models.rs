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
