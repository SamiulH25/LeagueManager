use crate::models::{
    ActiveLeague, AppState, ChampionshipRound, DriverLeague, DriverProfile, HostSettings, ImportResult,
    LeagueInvite, LeagueMember, LeagueRoster, LeagueSummary, PendingLeagueInvite, ResultsFeed,
    ResultsWarning, SessionResultSummary, StandingsResponse, StandingRow,
};
use crate::points::points_for_position;
use crate::results::parse_results_json;
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
                driver_id INTEGER REFERENCES drivers(id),
                driver_name TEXT NOT NULL,
                team TEXT,
                points INTEGER NOT NULL DEFAULT 0,
                avatar_url TEXT
            );

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                championship_id INTEGER NOT NULL REFERENCES championships(id),
                name TEXT NOT NULL,
                track TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'scheduled',
                started_at TEXT,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS session_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER REFERENCES events(id),
                championship_id INTEGER NOT NULL REFERENCES championships(id),
                session_type TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'auto',
                file_name TEXT,
                raw_json TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                track TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS result_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_result_id INTEGER NOT NULL REFERENCES session_results(id),
                driver_id INTEGER REFERENCES drivers(id),
                driver_name TEXT NOT NULL,
                driver_guid TEXT,
                position INTEGER,
                best_lap_ms INTEGER,
                laps INTEGER NOT NULL DEFAULT 0,
                total_time_ms INTEGER,
                dnf INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS points_ledger (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                championship_id INTEGER NOT NULL REFERENCES championships(id),
                event_id INTEGER REFERENCES events(id),
                session_result_id INTEGER REFERENCES session_results(id),
                driver_id INTEGER REFERENCES drivers(id),
                driver_name TEXT NOT NULL,
                points INTEGER NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS results_warnings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message TEXT NOT NULL,
                file_name TEXT,
                created_at TEXT NOT NULL,
                dismissed INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS championship_rounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                championship_id INTEGER NOT NULL REFERENCES championships(id),
                round_number INTEGER NOT NULL,
                name TEXT NOT NULL,
                track TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'scheduled',
                created_at TEXT NOT NULL,
                completed_at TEXT
            );
            "#,
        )?;
        let _ = self.conn.execute(
            "ALTER TABLE championship_standings ADD COLUMN driver_id INTEGER REFERENCES drivers(id)",
            [],
        );
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

        let mut members = self.conn.prepare(
            "SELECT d.id, d.personaname, d.avatar_url, m.team
             FROM league_members m
             JOIN drivers d ON d.id = m.driver_id
             WHERE m.league_id = ?1 AND m.status = 'active'",
        )?;
        let member_rows: Vec<(i64, String, String, Option<String>)> = members
            .query_map([league_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if member_rows.is_empty() {
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
        } else {
            for (i, (driver_id, name, avatar, team)) in member_rows.iter().enumerate() {
                self.conn.execute(
                    "INSERT INTO championship_standings
                     (championship_id, position, driver_id, driver_name, team, points, avatar_url)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
                    params![championship_id, i + 1, driver_id, name, team, avatar],
                )?;
            }
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
        self.set_active_league_id(id)?;
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

    pub fn begin_active_event(&self, event_name: &str, track: &str) -> Result<i64, DbError> {
        let league_id = self.get_active_league_id()?;
        let championship_id = self.championship_id_for_league(league_id)?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO events (championship_id, name, track, status, started_at)
             VALUES (?1, ?2, ?3, 'live', ?4)",
            params![championship_id, event_name, track, now],
        )?;
        let event_id = self.conn.last_insert_rowid();
        self.meta_set("active_event_id", &event_id.to_string())?;
        self.create_championship_round(championship_id, event_name, track)?;
        Ok(event_id)
    }

    pub fn complete_active_event(&self) -> Result<(), DbError> {
        if let Some(id) = self.meta_get("active_event_id")? {
            if !id.is_empty() {
                let now = Utc::now().to_rfc3339();
                self.conn.execute(
                    "UPDATE events SET status = 'completed', completed_at = ?1 WHERE id = ?2",
                    params![now, id],
                )?;
            }
        }
        self.meta_set("active_event_id", "")?;
        self.complete_active_round()?;
        Ok(())
    }

    fn active_championship_id(&self) -> Result<i64, DbError> {
        if let Some(id) = self.meta_get("active_event_id")? {
            if !id.is_empty() {
                if let Ok(champ_id) = self.conn.query_row(
                    "SELECT championship_id FROM events WHERE id = ?1",
                    [id],
                    |row| row.get::<_, i64>(0),
                ) {
                    return Ok(champ_id);
                }
            }
        }
        self.first_championship_id()
            .ok_or_else(|| DbError::Message("no active championship".into()))
    }

    fn find_driver_id(&self, guid: Option<&str>, name: &str) -> Option<i64> {
        if let Some(g) = guid.filter(|g| !g.is_empty()) {
            if let Ok(id) = self.conn.query_row(
                "SELECT id FROM drivers WHERE steam_id64 = ?1",
                [g],
                |row| row.get(0),
            ) {
                return Some(id);
            }
        }
        self.conn
            .query_row(
                "SELECT id FROM drivers WHERE personaname = ?1 COLLATE NOCASE",
                [name],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn import_results_file(
        &self,
        file_name: &str,
        raw: &str,
        source: &str,
    ) -> Result<ImportResult, DbError> {
        let parsed = match parse_results_json(raw, Some(file_name)) {
            Ok(p) => p,
            Err(e) => {
                let msg = e.to_string();
                self.add_warning(&msg, Some(file_name))?;
                return Ok(ImportResult {
                    success: false,
                    session_type: "?".into(),
                    track: String::new(),
                    entries_imported: 0,
                    points_awarded: 0,
                    message: msg.clone(),
                    warning: Some(msg),
                });
            }
        };

        if self.conn.query_row(
            "SELECT COUNT(*) FROM session_results WHERE file_name = ?1",
            [file_name],
            |row| row.get::<_, i64>(0),
        )? > 0
        {
            return Ok(ImportResult {
                success: true,
                session_type: parsed.session_type.clone(),
                track: parsed.track.clone(),
                entries_imported: 0,
                points_awarded: 0,
                message: "Results file already imported".into(),
                warning: None,
            });
        }

        let championship_id = self.active_championship_id()?;
        let event_id: Option<i64> = self
            .meta_get("active_event_id")?
            .and_then(|v| v.parse().ok())
            .filter(|id| *id > 0);
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO session_results
             (event_id, championship_id, session_type, source, file_name, raw_json, imported_at, track)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event_id,
                championship_id,
                parsed.session_type,
                source,
                file_name,
                raw,
                now,
                parsed.track
            ],
        )?;
        let session_result_id = self.conn.last_insert_rowid();

        let mut unknown_drivers = 0u32;
        let mut points_awarded = 0u32;

        for entry in &parsed.entries {
            let driver_id = self.find_driver_id(entry.driver_guid.as_deref(), &entry.driver_name);
            if driver_id.is_none() {
                unknown_drivers += 1;
            }

            self.conn.execute(
                "INSERT INTO result_entries
                 (session_result_id, driver_id, driver_name, driver_guid, position, best_lap_ms, laps, total_time_ms, dnf)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    session_result_id,
                    driver_id,
                    entry.driver_name,
                    entry.driver_guid,
                    entry.position,
                    entry.best_lap_ms,
                    entry.laps,
                    entry.total_time_ms,
                    entry.dnf as i32
                ],
            )?;

            if parsed.session_type == "R" {
                if let Some(pos) = entry.position {
                    let pts = if entry.dnf { 0 } else { points_for_position(pos) };
                    if pts > 0 {
                        self.conn.execute(
                            "INSERT INTO points_ledger
                             (championship_id, event_id, session_result_id, driver_id, driver_name, points, reason, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            params![
                                championship_id,
                                event_id,
                                session_result_id,
                                driver_id,
                                entry.driver_name,
                                pts,
                                format!("Race P{pos}"),
                                now
                            ],
                        )?;
                        points_awarded += pts;
                    }
                }
            }
        }

        self.recalculate_championship_standings(championship_id)?;

        if parsed.session_type == "R" {
            let _ = self.complete_active_round();
        }

        let mut warning = None;
        if unknown_drivers > 0 {
            let msg = format!(
                "{unknown_drivers} driver(s) not matched to league roster — invite them on Steam"
            );
            self.add_warning(&msg, Some(file_name))?;
            warning = Some(msg);
        }

        Ok(ImportResult {
            success: true,
            session_type: parsed.session_type,
            track: parsed.track,
            entries_imported: parsed.entries.len() as u32,
            points_awarded,
            message: format!(
                "Imported {} entries from {}",
                parsed.entries.len(),
                file_name
            ),
            warning,
        })
    }

    fn recalculate_championship_standings(&self, championship_id: i64) -> Result<(), DbError> {
        let ledger_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM points_ledger WHERE championship_id = ?1",
            [championship_id],
            |row| row.get(0),
        )?;
        if ledger_count == 0 {
            return Ok(());
        }

        self.conn.execute(
            "DELETE FROM championship_standings WHERE championship_id = ?1",
            [championship_id],
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(driver_id, 0), driver_name, SUM(points) as total
             FROM points_ledger
             WHERE championship_id = ?1
             GROUP BY COALESCE(driver_id, 0), driver_name
             ORDER BY total DESC, driver_name ASC",
        )?;
        let rows: Vec<(i64, String, i64)> = stmt
            .query_map([championship_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        if rows.is_empty() {
            return Ok(());
        }

        for (i, (driver_id, name, total)) in rows.iter().enumerate() {
            let avatar: Option<String> = if *driver_id > 0 {
                self.conn
                    .query_row(
                        "SELECT avatar_url FROM drivers WHERE id = ?1",
                        [driver_id],
                        |row| row.get(0),
                    )
                    .optional()?
            } else {
                None
            };
            let team: Option<String> = self
                .conn
                .query_row(
                    "SELECT team FROM result_entries re
                     JOIN session_results sr ON sr.id = re.session_result_id
                     WHERE sr.championship_id = ?1 AND re.driver_name = ?2
                     ORDER BY sr.imported_at DESC LIMIT 1",
                    params![championship_id, name],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();

            self.conn.execute(
                "INSERT INTO championship_standings
                 (championship_id, position, driver_id, driver_name, team, points, avatar_url)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    championship_id,
                    i + 1,
                    if *driver_id > 0 { Some(*driver_id) } else { None },
                    name,
                    team,
                    total,
                    avatar
                ],
            )?;
        }
        Ok(())
    }

    fn add_warning(&self, message: &str, file_name: Option<&str>) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO results_warnings (message, file_name, created_at) VALUES (?1, ?2, ?3)",
            params![message, file_name, now],
        )?;
        Ok(())
    }

    pub fn get_results_feed(&self, watcher_active: bool) -> Result<ResultsFeed, DbError> {
        let mut warnings_stmt = self.conn.prepare(
            "SELECT id, message, file_name, created_at, dismissed
             FROM results_warnings
             WHERE dismissed = 0
             ORDER BY created_at DESC
             LIMIT 10",
        )?;
        let warnings = warnings_stmt
            .query_map([], |row| {
                Ok(ResultsWarning {
                    id: row.get(0)?,
                    message: row.get(1)?,
                    file_name: row.get(2)?,
                    created_at: row.get(3)?,
                    dismissed: row.get::<_, i64>(4)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut recent_stmt = self.conn.prepare(
            "SELECT sr.id, sr.session_type, sr.track, sr.source, sr.file_name, sr.imported_at,
                    (SELECT COUNT(*) FROM result_entries re WHERE re.session_result_id = sr.id)
             FROM session_results sr
             ORDER BY sr.imported_at DESC
             LIMIT 10",
        )?;
        let recent = recent_stmt
            .query_map([], |row| {
                Ok(SessionResultSummary {
                    id: row.get(0)?,
                    session_type: row.get(1)?,
                    track: row.get(2)?,
                    source: row.get(3)?,
                    file_name: row.get(4)?,
                    imported_at: row.get(5)?,
                    entry_count: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ResultsFeed {
            warnings,
            recent,
            watcher_active,
        })
    }

    pub fn dismiss_results_warning(&self, warning_id: i64) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE results_warnings SET dismissed = 1 WHERE id = ?1",
            [warning_id],
        )?;
        Ok(())
    }

    pub fn get_active_league_id(&self) -> Result<i64, DbError> {
        if let Some(v) = self.meta_get("active_league_id")? {
            if let Ok(id) = v.parse::<i64>() {
                if id > 0 {
                    return Ok(id);
                }
            }
        }
        self.conn
            .query_row(
                "SELECT id FROM leagues WHERE archived_at IS NULL ORDER BY id LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(|_| DbError::Message("no leagues — create one first".into()))
    }

    pub fn set_active_league_id(&self, league_id: i64) -> Result<(), DbError> {
        let exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM leagues WHERE id = ?1 AND archived_at IS NULL",
            [league_id],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(DbError::Message("league not found".into()));
        }
        self.meta_set("active_league_id", &league_id.to_string())
    }

    pub fn get_active_league(&self) -> Result<ActiveLeague, DbError> {
        let id = self.get_active_league_id()?;
        let name: String = self.conn.query_row(
            "SELECT name FROM leagues WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        Ok(ActiveLeague { id, name })
    }

    pub fn send_league_invite(
        &self,
        league_id: i64,
        target_steam_id64: &str,
        host_steam_id64: &str,
        profile: &DriverProfile,
    ) -> Result<PendingLeagueInvite, DbError> {
        self.upsert_driver(profile)?;

        let member: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM league_members m
             JOIN drivers d ON d.id = m.driver_id
             WHERE m.league_id = ?1 AND d.steam_id64 = ?2",
            params![league_id, target_steam_id64],
            |row| row.get(0),
        )?;
        if member > 0 {
            return Err(DbError::Message("driver is already in this league".into()));
        }

        let pending: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM league_invites
             WHERE league_id = ?1 AND steam_id64 = ?2 AND status = 'pending'",
            params![league_id, target_steam_id64],
            |row| row.get(0),
        )?;
        if pending > 0 {
            return Err(DbError::Message("invite already pending for this driver".into()));
        }

        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO league_invites (league_id, steam_id64, status, invited_at, invited_by_steam_id64)
             VALUES (?1, ?2, 'pending', ?3, ?4)",
            params![league_id, target_steam_id64, now, host_steam_id64],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(PendingLeagueInvite {
            id,
            steam_id64: target_steam_id64.to_string(),
            personaname: profile.personaname.clone(),
            avatar_url: profile.avatar_url.clone(),
            invited_at: now,
        })
    }

    pub fn accept_invite(&self, invite_id: i64, steam_id64: &str) -> Result<(), DbError> {
        let (league_id, invite_steam, status): (i64, String, String) = self.conn.query_row(
            "SELECT league_id, steam_id64, status FROM league_invites WHERE id = ?1",
            [invite_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        if status != "pending" {
            return Err(DbError::Message("invite is no longer pending".into()));
        }
        if invite_steam != steam_id64 {
            return Err(DbError::Message(
                "this invite is for a different Steam account".into(),
            ));
        }

        let driver_id: i64 = self.conn.query_row(
            "SELECT id FROM drivers WHERE steam_id64 = ?1",
            [steam_id64],
            |row| row.get(0),
        )?;

        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE league_invites SET status = 'accepted', responded_at = ?1 WHERE id = ?2",
            params![now, invite_id],
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO league_members (league_id, driver_id, joined_at)
             VALUES (?1, ?2, ?3)",
            params![league_id, driver_id, now],
        )?;
        self.ensure_demo_championship(league_id)?;
        Ok(())
    }

    pub fn decline_invite(&self, invite_id: i64, steam_id64: &str) -> Result<(), DbError> {
        let (invite_steam, status): (String, String) = self.conn.query_row(
            "SELECT steam_id64, status FROM league_invites WHERE id = ?1",
            [invite_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if status != "pending" {
            return Err(DbError::Message("invite is no longer pending".into()));
        }
        if invite_steam != steam_id64 {
            return Err(DbError::Message(
                "this invite is for a different Steam account".into(),
            ));
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE league_invites SET status = 'declined', responded_at = ?1 WHERE id = ?2",
            params![now, invite_id],
        )?;
        Ok(())
    }

    pub fn revoke_invite(&self, invite_id: i64, host_steam_id64: &str) -> Result<(), DbError> {
        let invited_by: String = self.conn.query_row(
            "SELECT invited_by_steam_id64 FROM league_invites WHERE id = ?1",
            [invite_id],
            |row| row.get(0),
        )?;
        if invited_by != host_steam_id64 {
            return Err(DbError::Message("only the inviting host can revoke".into()));
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE league_invites SET status = 'revoked', responded_at = ?1 WHERE id = ?2",
            params![now, invite_id],
        )?;
        Ok(())
    }

    pub fn list_league_roster(&self, league_id: i64) -> Result<LeagueRoster, DbError> {
        let mut members_stmt = self.conn.prepare(
            "SELECT d.id, d.steam_id64, d.personaname, d.avatar_url, m.team, m.joined_at
             FROM league_members m
             JOIN drivers d ON d.id = m.driver_id
             WHERE m.league_id = ?1 AND m.status = 'active'
             ORDER BY m.joined_at ASC",
        )?;
        let members = members_stmt
            .query_map([league_id], |row| {
                Ok(LeagueMember {
                    driver_id: row.get(0)?,
                    steam_id64: row.get(1)?,
                    personaname: row.get(2)?,
                    avatar_url: row.get(3)?,
                    team: row.get(4)?,
                    joined_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut pending_stmt = self.conn.prepare(
            "SELECT i.id, i.steam_id64, COALESCE(d.personaname, i.steam_id64), COALESCE(d.avatar_url, ''), i.invited_at
             FROM league_invites i
             LEFT JOIN drivers d ON d.steam_id64 = i.steam_id64
             WHERE i.league_id = ?1 AND i.status = 'pending'
             ORDER BY i.invited_at DESC",
        )?;
        let pending_invites = pending_stmt
            .query_map([league_id], |row| {
                Ok(PendingLeagueInvite {
                    id: row.get(0)?,
                    steam_id64: row.get(1)?,
                    personaname: row.get(2)?,
                    avatar_url: row.get(3)?,
                    invited_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LeagueRoster {
            members,
            pending_invites,
        })
    }

    pub fn list_driver_leagues(&self, steam_id64: &str) -> Result<Vec<DriverLeague>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT l.id, l.name,
                    (SELECT COUNT(*) FROM league_members m WHERE m.league_id = l.id) as members
             FROM leagues l
             JOIN league_members m ON m.league_id = l.id
             JOIN drivers d ON d.id = m.driver_id
             WHERE d.steam_id64 = ?1 AND l.archived_at IS NULL
             ORDER BY l.name ASC",
        )?;
        let rows = stmt
            .query_map([steam_id64], |row| {
                Ok(DriverLeague {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    member_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn refresh_league_roster_profiles(&self, league_id: i64, profiles: &[DriverProfile]) -> Result<u32, DbError> {
        let mut updated = 0u32;
        for profile in profiles {
            self.upsert_driver(profile)?;
            updated += 1;
        }
        let _ = league_id;
        Ok(updated)
    }

    pub fn championship_id_for_league(&self, league_id: i64) -> Result<i64, DbError> {
        self.conn
            .query_row(
                "SELECT id FROM championships WHERE league_id = ?1 ORDER BY id LIMIT 1",
                [league_id],
                |row| row.get(0),
            )
            .map_err(|_| DbError::Message("no championship for league".into()))
    }

    pub fn create_championship_round(
        &self,
        championship_id: i64,
        name: &str,
        track: &str,
    ) -> Result<ChampionshipRound, DbError> {
        let round_number: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(round_number), 0) + 1 FROM championship_rounds WHERE championship_id = ?1",
            [championship_id],
            |row| row.get(0),
        )?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO championship_rounds (championship_id, round_number, name, track, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'live', ?5)",
            params![championship_id, round_number, name, track, now],
        )?;
        let id = self.conn.last_insert_rowid();
        self.meta_set("active_round_id", &id.to_string())?;
        Ok(ChampionshipRound {
            id,
            championship_id,
            round_number: round_number as u32,
            name: name.to_string(),
            track: track.to_string(),
            status: "live".to_string(),
            created_at: now,
            completed_at: None,
        })
    }

    pub fn list_championship_rounds(&self, championship_id: i64) -> Result<Vec<ChampionshipRound>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, championship_id, round_number, name, track, status, created_at, completed_at
             FROM championship_rounds
             WHERE championship_id = ?1
             ORDER BY round_number ASC",
        )?;
        let rows = stmt
            .query_map([championship_id], |row| {
                Ok(ChampionshipRound {
                    id: row.get(0)?,
                    championship_id: row.get(1)?,
                    round_number: row.get::<_, i64>(2)? as u32,
                    name: row.get(3)?,
                    track: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    completed_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn complete_active_round(&self) -> Result<(), DbError> {
        if let Some(id) = self.meta_get("active_round_id")? {
            if !id.is_empty() {
                let now = Utc::now().to_rfc3339();
                self.conn.execute(
                    "UPDATE championship_rounds SET status = 'completed', completed_at = ?1 WHERE id = ?2",
                    params![now, id],
                )?;
            }
        }
        self.meta_set("active_round_id", "")?;
        Ok(())
    }

    pub fn export_standings_csv(&self, championship_id: i64) -> Result<String, DbError> {
        let standings = self.get_championship_standings(championship_id)?;
        let mut csv = String::from("position,driver,team,points\n");
        for row in standings.rows {
            let team = row.team.unwrap_or_default().replace(',', " ");
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.position,
                row.driver_name.replace(',', " "),
                team,
                row.points
            ));
        }
        Ok(csv)
    }
}

pub fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("LeagueManager").join("league-manager.db")
}
