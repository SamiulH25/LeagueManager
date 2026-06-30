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
