import { api } from "$lib/api";
import type { AppState } from "$lib/types";

let state = $state<AppState | null>(null);
let loading = $state(true);
let error = $state<string | null>(null);

export function getAppStore() {
  return {
    get state() {
      return state;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    async refresh() {
      loading = true;
      error = null;
      try {
        state = await api.getAppState();
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
      } finally {
        loading = false;
      }
    },
    async setMode(mode: "host" | "driver") {
      state = await api.setAppMode(mode);
    },
    async logout() {
      state = await api.steamLogout();
    },
    async loginWithSteam() {
      const profile = await api.steamLogin();
      await this.refresh();
      return profile;
    },
    async loginDev(steamId64: string) {
      await api.steamLoginDev(steamId64);
      await this.refresh();
    },
  };
}

export type AppStore = ReturnType<typeof getAppStore>;
