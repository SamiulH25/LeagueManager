import { goto } from "$app/navigation";
import type { AppStore } from "$lib/stores/app.svelte";

export function guardHost(store: AppStore): boolean {
  const s = store.state;
  if (!s?.session) {
    goto("/login");
    return false;
  }
  if (s.appMode !== "host") {
    goto("/driver");
    return false;
  }
  return true;
}

export function guardDriver(store: AppStore): boolean {
  const s = store.state;
  if (!s?.session) {
    goto("/login");
    return false;
  }
  if (s.appMode !== "driver") {
    goto("/host");
    return false;
  }
  return true;
}
