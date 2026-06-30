export type AppMode = "host" | "driver";

export interface DriverProfile {
  steamId64: string;
  personaname: string;
  avatarUrl: string;
}

export interface AppState {
  onboardingComplete: boolean;
  appMode: AppMode | null;
  session: DriverProfile | null;
}

export interface LeagueSummary {
  id: number;
  name: string;
  memberCount: number;
  createdAt: string;
}

export interface LeagueInvite {
  id: number;
  leagueId: number;
  leagueName: string;
  status: string;
  invitedAt: string;
  hostPersonaname?: string | null;
  hostAvatarUrl?: string | null;
}

export interface HostSettings {
  assettoServerPath: string;
  acInstallPath: string;
  httpPort: number;
  gamePort: number;
  adminPassword: string;
  publicIpOverride: string;
}

export interface PathSuggestions {
  assettoServerPath?: string | null;
  acInstallPath?: string | null;
}

export interface RaceLaunchConfig {
  serverName: string;
  track: string;
  trackConfig?: string;
  cars: string[];
  password?: string;
  practiceMinutes?: number;
  qualifyMinutes?: number;
  raceMinutes?: number;
  maxClients?: number;
  aiSlots?: number;
}

export interface ServerInfo {
  name: string;
  track: string;
  clients: number;
  maxClients: number;
  session: number;
  timeLeft: number;
  port: number;
}

export interface ServerStatus {
  running: boolean;
  publicIp?: string | null;
  httpPort: number;
  gamePort: number;
  cmJoinLink?: string | null;
  serverName?: string | null;
  info?: ServerInfo | null;
  error?: string | null;
}
