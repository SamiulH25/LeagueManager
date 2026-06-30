use crate::models::{HostSettings, PathSuggestions, RaceLaunchConfig, ServerInfo, ServerStatus};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("{0}")]
    Message(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

pub struct ServerManager {
    inner: Mutex<ServerRuntime>,
}

struct ServerRuntime {
    child: Option<Child>,
    http_port: u16,
    game_port: u16,
    password: String,
    server_name: String,
    config_dir: PathBuf,
}

impl Default for ServerRuntime {
    fn default() -> Self {
        Self {
            child: None,
            http_port: 8081,
            game_port: 9600,
            password: String::new(),
            server_name: String::new(),
            config_dir: PathBuf::new(),
        }
    }
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ServerRuntime::default()),
        }
    }

    pub fn is_running(&self) -> Result<bool, ServerError> {
        let mut rt = self.inner.lock().map_err(|e| ServerError::Message(e.to_string()))?;
        Ok(rt.check_running()?)
    }

    pub fn start(
        &self,
        settings: &HostSettings,
        race: &RaceLaunchConfig,
    ) -> Result<(), ServerError> {
        if settings.assetto_server_path.is_empty() {
            return Err(ServerError::Message(
                "Set AssettoServer path in Pit Config first".into(),
            ));
        }

        let server_root = PathBuf::from(&settings.assetto_server_path);
        let exe = assetto_server_exe(&server_root);
        if !exe.exists() {
            return Err(ServerError::Message(format!(
                "AssettoServer not found at {}",
                exe.display()
            )));
        }

        let config_dir = active_server_dir();
        fs::create_dir_all(config_dir.join("cfg"))?;

        if self.is_running()? {
            self.stop()?;
        }

        write_configs(&config_dir, settings, race)?;

        let cfg = config_dir.join("cfg/server_cfg.ini");
        let entry = config_dir.join("cfg/entry_list.ini");

        let mut cmd = Command::new(&exe);
        cmd.current_dir(&server_root)
            .arg("-c")
            .arg(&cfg)
            .arg("-e")
            .arg(&entry)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn()?;

        let mut rt = self.inner.lock().map_err(|e| ServerError::Message(e.to_string()))?;
        rt.child = Some(child);
        rt.http_port = settings.http_port;
        rt.game_port = settings.game_port;
        rt.password = race.password.clone();
        rt.server_name = race.server_name.clone();
        rt.config_dir = config_dir;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), ServerError> {
        let mut rt = self.inner.lock().map_err(|e| ServerError::Message(e.to_string()))?;
        if let Some(mut child) = rt.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub async fn status(
        &self,
        settings: &HostSettings,
    ) -> Result<ServerStatus, ServerError> {
        let (running, http_port, game_port, password, server_name) = {
            let mut rt = self.inner.lock().map_err(|e| ServerError::Message(e.to_string()))?;
            let running = rt.check_running()?;
            (
                running,
                rt.http_port,
                rt.game_port,
                rt.password.clone(),
                rt.server_name.clone(),
            )
        };

        let public_ip = resolve_public_ip(settings).await.ok();

        if !running {
            return Ok(ServerStatus {
                running: false,
                public_ip,
                http_port,
                game_port,
                cm_join_link: None,
                server_name: if server_name.is_empty() {
                    None
                } else {
                    Some(server_name)
                },
                info: None,
                error: None,
            });
        }

        match fetch_info(http_port).await {
            Ok(info) => {
                let cm_join_link = public_ip.as_ref().map(|ip| {
                    build_cm_join_link(ip, http_port, &password)
                });
                Ok(ServerStatus {
                    running: true,
                    public_ip,
                    http_port,
                    game_port,
                    cm_join_link,
                    server_name: Some(info.name.clone()),
                    info: Some(info),
                    error: None,
                })
            }
            Err(e) => Ok(ServerStatus {
                running: true,
                public_ip: public_ip.clone(),
                http_port,
                game_port,
                cm_join_link: public_ip.as_ref().map(|ip| {
                    build_cm_join_link(ip, http_port, &password)
                }),
                server_name: if server_name.is_empty() {
                    None
                } else {
                    Some(server_name)
                },
                info: None,
                error: Some(e.to_string()),
            }),
        }
    }
}

impl ServerRuntime {
    fn check_running(&mut self) -> Result<bool, ServerError> {
        if let Some(child) = &mut self.child {
            match child.try_wait()? {
                Some(_) => {
                    self.child = None;
                    return Ok(false);
                }
                None => return Ok(true),
            }
        }
        Ok(false)
    }
}

fn assetto_server_exe(server_root: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        server_root.join("AssettoServer.exe")
    }
    #[cfg(not(windows))]
    {
        server_root.join("AssettoServer")
    }
}

pub fn active_server_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("LeagueManager").join("servers").join("active")
}

pub fn detect_path_suggestions() -> PathSuggestions {
    let mut assetto_server_path = None;
    #[cfg(windows)]
    let mut ac_install_path = None;
    #[cfg(not(windows))]
    let ac_install_path = None;

    #[cfg(windows)]
    {
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let steam_ac = PathBuf::from(&program_files_x86)
                .join("Steam/steamapps/common/assettocorsa");
            if steam_ac.exists() {
                ac_install_path = Some(steam_ac.to_string_lossy().to_string());
            }
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let as_path = PathBuf::from(&userprofile).join("AssettoServer");
            if as_path.join("AssettoServer.exe").exists() {
                assetto_server_path = Some(as_path.to_string_lossy().to_string());
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let as_path = PathBuf::from(&home).join("AssettoServer");
            if as_path.join("AssettoServer").exists() {
                assetto_server_path = Some(as_path.to_string_lossy().to_string());
            }
        }
    }

    PathSuggestions {
        assetto_server_path,
        ac_install_path,
    }
}

pub fn build_cm_join_link(public_ip: &str, http_port: u16, password: &str) -> String {
    let mut url = format!(
        "https://acstuff.ru/s/q:race/online/join?ip={public_ip}&httpPort={http_port}"
    );
    if !password.is_empty() {
        let encoded: String = url::form_urlencoded::byte_serialize(password.as_bytes()).collect();
        url.push_str(&format!("&password={encoded}"));
    }
    url
}

async fn resolve_public_ip(settings: &HostSettings) -> Result<String, ServerError> {
    if !settings.public_ip_override.is_empty() {
        return Ok(settings.public_ip_override.clone());
    }
    let client = reqwest::Client::new();
    let ip = client
        .get("https://api.ipify.org")
        .send()
        .await?
        .text()
        .await?
        .trim()
        .to_string();
    Ok(ip)
}

#[derive(Debug, Deserialize)]
struct InfoResponse {
    name: String,
    track: String,
    clients: u32,
    maxclients: u32,
    session: u32,
    timeleft: u32,
    port: u32,
}

async fn fetch_info(http_port: u16) -> Result<ServerInfo, ServerError> {
    let url = format!("http://127.0.0.1:{http_port}/INFO");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;
    let resp: InfoResponse = client.get(&url).send().await?.json().await?;
    Ok(ServerInfo {
        name: resp.name,
        track: resp.track,
        clients: resp.clients,
        max_clients: resp.maxclients,
        session: resp.session,
        time_left: resp.timeleft,
        port: resp.port,
    })
}

fn write_configs(
    config_dir: &Path,
    settings: &HostSettings,
    race: &RaceLaunchConfig,
) -> Result<(), ServerError> {
    let cars = if race.cars.is_empty() {
        vec!["abarth500".to_string()]
    } else {
        race.cars.clone()
    };
    let cars_line = cars.join(";");

    let mut ini = String::new();
    ini.push_str("[SERVER]\n");
    ini.push_str(&format!("NAME={}\n", race.server_name));
    ini.push_str(&format!("CARS={cars_line}\n"));
    ini.push_str(&format!("TRACK={}\n", race.track));
    if !race.track_config.is_empty() {
        ini.push_str(&format!("CONFIG_TRACK={}\n", race.track_config));
    }
    ini.push_str(&format!("UDP_PORT={}\n", settings.game_port));
    ini.push_str(&format!("TCP_PORT={}\n", settings.game_port));
    ini.push_str(&format!("HTTP_PORT={}\n", settings.http_port));
    ini.push_str(&format!("MAX_CLIENTS={}\n", race.max_clients));
    if !race.password.is_empty() {
        ini.push_str(&format!("PASSWORD={}\n", race.password));
    }
    ini.push_str(&format!("ADMIN_PASSWORD={}\n", settings.admin_password));
    ini.push_str("REGISTER_TO_LOBBY=1\n");
    ini.push_str("LOOP_MODE=0\n");
    ini.push_str("PICKUP_MODE_ENABLED=1\n");
    ini.push_str("LOCKED_ENTRY_LIST=0\n");

    if race.practice_minutes > 0 {
        ini.push_str("\n[PRACTICE]\n");
        ini.push_str("NAME=Practice\n");
        ini.push_str(&format!("TIME={}\n", race.practice_minutes));
        ini.push_str("IS_OPEN=1\n");
    }
    if race.qualify_minutes > 0 {
        ini.push_str("\n[QUALIFY]\n");
        ini.push_str("NAME=Qualifying\n");
        ini.push_str(&format!("TIME={}\n", race.qualify_minutes));
        ini.push_str("IS_OPEN=1\n");
    }
    if race.race_minutes > 0 {
        ini.push_str("\n[RACE]\n");
        ini.push_str("NAME=Race\n");
        ini.push_str(&format!("TIME={}\n", race.race_minutes));
        ini.push_str("IS_OPEN=2\n");
        ini.push_str("WAIT_TIME=60\n");
    }

    fs::write(config_dir.join("cfg/server_cfg.ini"), ini)?;

    let mut entry = String::new();
    let slots = race.max_clients.max(2);
    let ai_slots = race.ai_slots.min(slots.saturating_sub(1));
    for i in 0..slots {
        let model = &cars[i as usize % cars.len()];
        let is_ai = i >= slots - ai_slots;
        entry.push_str(&format!("[CAR_{i}]\n"));
        entry.push_str(&format!("MODEL={model}\n"));
        entry.push_str("SKIN=default\n");
        entry.push_str("SPECTATOR_MODE=0\n");
        if is_ai {
            entry.push_str(&format!("DRIVERNAME=AI Driver {}\n", i + 1));
            entry.push_str("AI=1\n");
        } else {
            entry.push_str("DRIVERNAME=\n");
            entry.push_str("AI=0\n");
        }
        entry.push_str("BALLAST=0\n");
        entry.push_str("RESTRICTOR=0\n");
        entry.push('\n');
    }
    fs::write(config_dir.join("cfg/entry_list.ini"), entry)?;

    let extra = format!(
        "EnableServerDetails: true\nServerDescription: \"{} — LeagueManager\"\n",
        race.server_name
    );
    fs::write(config_dir.join("cfg/extra_cfg.yml"), extra)?;

    Ok(())
}
