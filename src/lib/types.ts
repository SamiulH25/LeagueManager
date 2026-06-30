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
