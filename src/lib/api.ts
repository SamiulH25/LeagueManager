import { invoke } from "@tauri-apps/api/core";
import type {
  AppState,
  DriverProfile,
  HostSettings,
  LeagueInvite,
  LeagueSummary,
  PathSuggestions,
  RaceLaunchConfig,
  ServerStatus,
} from "./types";

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
  detectPaths: () => invoke<PathSuggestions>("detect_paths"),
  getHostSettings: () => invoke<HostSettings>("get_host_settings"),
  saveHostSettings: (settings: HostSettings) =>
    invoke<void>("save_host_settings", { settings }),
  startRaceServer: (config: RaceLaunchConfig) =>
    invoke<void>("start_race_server", { config }),
  stopRaceServer: () => invoke<void>("stop_race_server"),
  getServerStatus: () => invoke<ServerStatus>("get_server_status"),
  openCmJoinLink: () => invoke<string>("open_cm_join_link"),
};
