import { invoke } from "@tauri-apps/api/core";
import type { AppState, DriverProfile, LeagueInvite, LeagueSummary } from "./types";

export const api = {
  getAppState: () => invoke<AppState>("get_app_state"),
  setAppMode: (mode: "host" | "driver") => invoke<AppState>("set_app_mode", { mode }),
  steamLogout: () => invoke<AppState>("steam_logout"),
  steamLogin: () => invoke<DriverProfile>("steam_login"),
  steamLoginDev: (steamId64: string) =>
    invoke<DriverProfile>("steam_login_dev", { steamId64 }),
  listLeagues: () => invoke<LeagueSummary[]>("list_leagues"),
  createLeague: (name: string) => invoke<LeagueSummary>("create_league", { name }),
  listMyInvites: () => invoke<LeagueInvite[]>("list_my_invites"),
  getDbPath: () => invoke<string>("get_db_path"),
};
