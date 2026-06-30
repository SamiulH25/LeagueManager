use crate::models::{AppState, DriverProfile, HostSettings, LeagueInvite, LeagueSummary, StandingsResponse, StandingRow};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Message(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self, DbError> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DbError::Message(e.to_string()))?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS app_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS drivers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                steam_id64 TEXT NOT NULL UNIQUE,
                personaname TEXT NOT NULL,
                avatar_url TEXT NOT NULL,
                profile_updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS leagues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                archived_at TEXT
            );

            CREATE TABLE IF NOT EXISTS league_invites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                league_id INTEGER NOT NULL REFERENCES leagues(id),
                steam_id64 TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                invited_at TEXT NOT NULL,
                responded_at TEXT,
                invited_by_steam_id64 TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS league_members (
                league_id INTEGER NOT NULL REFERENCES leagues(id),
                driver_id INTEGER NOT NULL REFERENCES drivers(id),
                team TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                joined_at TEXT NOT NULL,
                PRIMARY KEY (league_id, driver_id)
            );

            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                steam_id64 TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS championships (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                league_id INTEGER NOT NULL REFERENCES leagues(id),
                name TEXT NOT NULL,
                season TEXT NOT NULL DEFAULT '2026'
            );

            CREATE TABLE IF NOT EXISTS championship_standings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                championship_id INTEGER NOT NULL REFERENCES championships(id),
                position INTEGER NOT NULL,
                driver_name TEXT NOT NULL,
                team TEXT,
                points INTEGER NOT NULL DEFAULT 0,
                avatar_url TEXT
            );
            "#,
        )?;
        self.seed_demo_championship()?;
        Ok(())
    }

    fn seed_demo_championship(&self) -> Result<(), DbError> {
        let league_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM leagues ORDER BY id LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);

        if league_id == 0 {
            return Ok(());
        }
        self.ensure_demo_championship(league_id)
    }

    fn ensure_demo_championship(&self, league_id: i64) -> Result<(), DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM championships WHERE league_id = ?1",
            [league_id],
            |row| row.get(0),
        )?;
        if count > 0 {
            return Ok(());
        }

        self.conn.execute(
            "INSERT INTO championships (league_id, name, season) VALUES (?1, ?2, ?3)",
            params![league_id, "Demo Championship", "2026"],
        )?;

        let championship_id = self.conn.last_insert_rowid();
        let rows = [
            (1, "A. Rossi", "Scuderia Rosso", 45),
            (2, "B. Müller", "Alpine Racing", 38),
            (3, "C. Tanaka", "Sunrise Motorsport", 32),
            (4, "D. Brooks", "Brooks GP", 28),
            (5, "E. Silva", "Porto Racing", 21),
        ];
        for (pos, name, team, pts) in rows {
            self.conn.execute(
                "INSERT INTO championship_standings
                 (championship_id, position, driver_name, team, points)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![championship_id, pos, name, team, pts],
            )?;
        }
        Ok(())
    }

    fn meta_get(&self, key: &str) -> Result<Option<String>, DbError> {
        self.conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()
            .map_err(DbError::from)
    }

    fn meta_set(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_app_state(&self) -> Result<AppState, DbError> {
        let onboarding_complete = self
            .meta_get("onboarding_complete")?
            .map(|v| v == "true")
            .unwrap_or(false);
        let app_mode = self.meta_get("app_mode")?;

        let session_token = self.meta_get("session_token")?;
        let session = if let Some(token) = session_token {
            self.get_session_profile(&token)?
        } else {
            None
        };

        Ok(AppState {
            onboarding_complete,
            app_mode,
            session,
        })
    }

    pub fn set_app_mode(&self, mode: &str) -> Result<(), DbError> {
        if mode != "host" && mode != "driver" {
            return Err(DbError::Message("mode must be host or driver".into()));
        }
        self.meta_set("app_mode", mode)?;
        self.meta_set("onboarding_complete", "true")
    }

    pub fn clear_session(&self) -> Result<(), DbError> {
        if let Some(token) = self.meta_get("session_token")? {
            self.conn
                .execute("DELETE FROM sessions WHERE token = ?1", [token])?;
        }
        self.meta_set("session_token", "")?;
        Ok(())
    }

    pub fn upsert_driver(&self, profile: &DriverProfile) -> Result<i64, DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO drivers (steam_id64, personaname, avatar_url, profile_updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(steam_id64) DO UPDATE SET
               personaname = excluded.personaname,
               avatar_url = excluded.avatar_url,
               profile_updated_at = excluded.profile_updated_at",
            params![
                profile.steam_id64,
                profile.personaname,
                profile.avatar_url,
                now
            ],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM drivers WHERE steam_id64 = ?1",
            [&profile.steam_id64],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn create_session(&self, steam_id64: &str) -> Result<String, DbError> {
        let token = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires = now + chrono::Duration::days(30);
        self.conn.execute(
            "INSERT INTO sessions (token, steam_id64, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                token,
                steam_id64,
                now.to_rfc3339(),
                expires.to_rfc3339()
            ],
        )?;
        self.meta_set("session_token", &token)?;
        Ok(token)
    }

    fn get_session_profile(&self, token: &str) -> Result<Option<DriverProfile>, DbError> {
        if token.is_empty() {
            return Ok(None);
        }
        let row = self
            .conn
            .query_row(
                "SELECT d.steam_id64, d.personaname, d.avatar_url
                 FROM sessions s
                 JOIN drivers d ON d.steam_id64 = s.steam_id64
                 WHERE s.token = ?1 AND s.expires_at > ?2",
                params![token, Utc::now().to_rfc3339()],
                |row| {
                    Ok(DriverProfile {
                        steam_id64: row.get(0)?,
                        personaname: row.get(1)?,
                        avatar_url: row.get(2)?,
                    })
                },
            )
            .optional()?;
        Ok(row)
    }

    pub fn login_driver(&self, profile: DriverProfile) -> Result<DriverProfile, DbError> {
        self.upsert_driver(&profile)?;
        self.create_session(&profile.steam_id64)?;
        Ok(profile)
    }

    pub fn create_league(&self, name: &str, host_steam_id64: &str) -> Result<LeagueSummary, DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO leagues (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;
        let id = self.conn.last_insert_rowid();
        let driver_id: i64 = self.conn.query_row(
            "SELECT id FROM drivers WHERE steam_id64 = ?1",
            [host_steam_id64],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO league_members (league_id, driver_id, joined_at)
             VALUES (?1, ?2, ?3)",
            params![id, driver_id, now],
        )?;
        self.ensure_demo_championship(id)?;
        Ok(LeagueSummary {
            id,
            name: name.to_string(),
            member_count: 1,
            created_at: now,
        })
    }

    pub fn list_leagues(&self) -> Result<Vec<LeagueSummary>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, l.name, l.created_at,
                    (SELECT COUNT(*) FROM league_members m WHERE m.league_id = l.id) as members
             FROM leagues l
             WHERE l.archived_at IS NULL
             ORDER BY l.created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(LeagueSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    member_count: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_invites_for_steam(&self, steam_id64: &str) -> Result<Vec<LeagueInvite>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT i.id, i.league_id, l.name, i.status, i.invited_at,
                    hd.personaname, hd.avatar_url
             FROM league_invites i
             JOIN leagues l ON l.id = i.league_id
             LEFT JOIN drivers hd ON hd.steam_id64 = i.invited_by_steam_id64
             WHERE i.steam_id64 = ?1 AND i.status = 'pending'
             ORDER BY i.invited_at DESC",
        )?;
        let rows = stmt
            .query_map([steam_id64], |row| {
                Ok(LeagueInvite {
                    id: row.get(0)?,
                    league_id: row.get(1)?,
                    league_name: row.get(2)?,
                    status: row.get(3)?,
                    invited_at: row.get(4)?,
                    host_personaname: row.get(5)?,
                    host_avatar_url: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_host_settings(&self) -> Result<HostSettings, DbError> {
        let mut s = HostSettings::default();
        if let Some(v) = self.meta_get("assetto_server_path")? {
            s.assetto_server_path = v;
        }
        if let Some(v) = self.meta_get("ac_install_path")? {
            s.ac_install_path = v;
        }
        if let Some(v) = self.meta_get("http_port")? {
            if let Ok(p) = v.parse() {
                s.http_port = p;
            }
        }
        if let Some(v) = self.meta_get("game_port")? {
            if let Ok(p) = v.parse() {
                s.game_port = p;
            }
        }
        if let Some(v) = self.meta_get("admin_password")? {
            if !v.is_empty() {
                s.admin_password = v;
            }
        }
        if let Some(v) = self.meta_get("public_ip_override")? {
            s.public_ip_override = v;
        }
        if let Some(v) = self.meta_get("sync_port")? {
            if let Ok(p) = v.parse() {
                s.sync_port = p;
            }
        }
        Ok(s)
    }

    pub fn save_host_settings(&self, settings: &HostSettings) -> Result<(), DbError> {
        self.meta_set("assetto_server_path", &settings.assetto_server_path)?;
        self.meta_set("ac_install_path", &settings.ac_install_path)?;
        self.meta_set("http_port", &settings.http_port.to_string())?;
        self.meta_set("game_port", &settings.game_port.to_string())?;
        self.meta_set("admin_password", &settings.admin_password)?;
        self.meta_set("public_ip_override", &settings.public_ip_override)?;
        self.meta_set("sync_port", &settings.sync_port.to_string())?;
        Ok(())
    }

    pub fn get_championship_standings(
        &self,
        championship_id: i64,
    ) -> Result<StandingsResponse, DbError> {
        let name: String = self
            .conn
            .query_row(
                "SELECT name FROM championships WHERE id = ?1",
                [championship_id],
                |row| row.get(0),
            )
            .map_err(|_| DbError::Message("championship not found".into()))?;

        let mut stmt = self.conn.prepare(
            "SELECT position, driver_name, team, points, avatar_url
             FROM championship_standings
             WHERE championship_id = ?1
             ORDER BY position ASC",
        )?;
        let rows = stmt
            .query_map([championship_id], |row| {
                Ok(StandingRow {
                    position: row.get::<_, i64>(0)? as u32,
                    driver_name: row.get(1)?,
                    team: row.get(2)?,
                    points: row.get::<_, i64>(3)? as u32,
                    avatar_url: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(StandingsResponse {
            championship_id,
            championship_name: name,
            rows,
        })
    }

    pub fn first_championship_id(&self) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT id FROM championships ORDER BY id LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }
}

pub fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("LeagueManager").join("league-manager.db")
}
