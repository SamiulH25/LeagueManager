import { invoke } from "@tauri-apps/api/core";
import type {
  AppState,
  CurrentEvent,
  DriverProfile,
  HostSettings,
  LeagueApiStatus,
  LeagueInvite,
  LeagueSummary,
  PathSuggestions,
  PitLinkTestResult,
  RaceLaunchConfig,
  ResultsFeed,
  ImportResult,
  ActiveLeague,
  LeagueRoster,
  PendingLeagueInvite,
  ChampionshipRound,
  ServerStatus,
  StandingsResponse,
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
  getLeagueApiStatus: () => invoke<LeagueApiStatus>("get_league_api_status"),
  testPitLink: (host: string, port: number) =>
    invoke<PitLinkTestResult>("test_pit_link", { host, port }),
  fetchRemoteCurrentEvent: (host: string, port: number) =>
    invoke<CurrentEvent>("fetch_remote_current_event", { host, port }),
  fetchRemoteStandings: (host: string, port: number, championshipId: number) =>
    invoke<StandingsResponse>("fetch_remote_standings", { host, port, championshipId }),
  openRemoteCmJoinLink: (host: string, port: number) =>
    invoke<string>("open_remote_cm_join_link", { host, port }),
  getResultsFeed: () => invoke<ResultsFeed>("get_results_feed"),
  importResultsJson: (json: string, fileName?: string) =>
    invoke<ImportResult>("import_results_json", { json, fileName }),
  dismissResultsWarning: (warningId: number) =>
    invoke<void>("dismiss_results_warning", { warningId }),
  getActiveLeague: () => invoke<ActiveLeague>("get_active_league"),
  setActiveLeague: (leagueId: number) =>
    invoke<ActiveLeague>("set_active_league", { leagueId }),
  sendDriverInvite: (leagueId: number, steamInput: string) =>
    invoke<PendingLeagueInvite>("send_driver_invite", { leagueId, steamInput }),
  listLeagueRoster: (leagueId: number) =>
    invoke<LeagueRoster>("list_league_roster", { leagueId }),
  refreshRosterAvatars: (leagueId: number) =>
    invoke<number>("refresh_roster_avatars", { leagueId }),
  revokeDriverInvite: (inviteId: number) =>
    invoke<void>("revoke_driver_invite", { inviteId }),
  fetchRemoteInvites: (host: string, port: number) =>
    invoke<LeagueInvite[]>("fetch_remote_invites", { host, port }),
  acceptRemoteInvite: (host: string, port: number, inviteId: number) =>
    invoke<void>("accept_remote_invite", { host, port, inviteId }),
  declineRemoteInvite: (host: string, port: number, inviteId: number) =>
    invoke<void>("decline_remote_invite", { host, port, inviteId }),
  listChampionshipRounds: (leagueId?: number) =>
    invoke<ChampionshipRound[]>("list_championship_rounds", { leagueId: leagueId ?? null }),
  exportStandingsCsv: (championshipId?: number) =>
    invoke<string>("export_standings_csv", { championshipId: championshipId ?? null }),
};
