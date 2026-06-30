use crate::models::DriverProfile;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum SteamError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("steam openid: {0}")]
    OpenId(String),
    #[error("steam api: {0}")]
    Api(String),
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesResponse {
    response: PlayerSummariesInner,
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesInner {
    players: Vec<SteamPlayer>,
}

#[derive(Debug, Deserialize)]
struct SteamPlayer {
    steamid: String,
    personaname: String,
    avatarfull: String,
}

pub fn build_openid_login_url(return_to: &str) -> String {
    let mut url = Url::parse("https://steamcommunity.com/openid/login").expect("valid url");
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("openid.ns", "http://specs.openid.net/auth/2.0");
        pairs.append_pair("openid.mode", "checkid_setup");
        pairs.append_pair("openid.return_to", return_to);
        pairs.append_pair("openid.realm", return_to);
        pairs.append_pair("openid.identity", "http://specs.openid.net/auth/2.0/identifier_select");
        pairs.append_pair("openid.claimed_id", "http://specs.openid.net/auth/2.0/identifier_select");
    }
    url.to_string()
}

pub fn steam_id_from_claimed_id(claimed_id: &str) -> Result<String, SteamError> {
    const PREFIX: &str = "https://steamcommunity.com/openid/id/";
    if !claimed_id.starts_with(PREFIX) {
        return Err(SteamError::OpenId("invalid claimed_id".into()));
    }
    Ok(claimed_id[PREFIX.len()..].to_string())
}

pub async fn verify_openid_response(query: &HashMap<String, String>) -> Result<String, SteamError> {
    if query.get("openid.mode").map(String::as_str) != Some("id_res") {
        return Err(SteamError::OpenId("expected id_res mode".into()));
    }

    let claimed_id = query
        .get("openid.claimed_id")
        .ok_or_else(|| SteamError::OpenId("missing claimed_id".into()))?;

    let mut body = HashMap::new();
    for (k, v) in query {
        let key = if k == "openid.mode" {
            "openid.mode".to_string()
        } else {
            k.clone()
        };
        body.insert(key, v.clone());
    }
    body.insert("openid.mode".to_string(), "check_authentication".to_string());

    let client = Client::new();
    let text = client
        .post("https://steamcommunity.com/openid/login")
        .form(&body)
        .send()
        .await?
        .text()
        .await?;

    if !text.lines().any(|line| line.trim() == "is_valid:true") {
        return Err(SteamError::OpenId("steam rejected assertion".into()));
    }

    steam_id_from_claimed_id(claimed_id)
}

pub async fn fetch_player_profile(steam_id64: &str, api_key: Option<&str>) -> Result<DriverProfile, SteamError> {
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={steam_id64}"
        );
        let client = Client::new();
        let resp: PlayerSummariesResponse = client.get(&url).send().await?.json().await?;
        let player = resp
            .response
            .players
            .into_iter()
            .next()
            .ok_or_else(|| SteamError::Api("player not found".into()))?;
        return Ok(DriverProfile {
            steam_id64: player.steamid,
            personaname: player.personaname,
            avatar_url: player.avatarfull,
        });
    }

    Ok(dev_profile(steam_id64))
}

pub fn dev_profile(steam_id64: &str) -> DriverProfile {
    DriverProfile {
        steam_id64: steam_id64.to_string(),
        personaname: format!("Driver {}", &steam_id64[steam_id64.len().saturating_sub(4)..]),
        avatar_url: "https://avatars.steamstatic.com/fef49e7fa7e1997310d705b2a6158ff25dc2cd16_full.jpg"
            .to_string(),
    }
}

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
        .into_owned()
        .collect()
}
