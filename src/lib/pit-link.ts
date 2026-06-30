import type { PitLinkConfig } from "./types";

const HOST_KEY = "lm_host_address";
const PORT_KEY = "lm_sync_port";
const DEFAULT_PORT = 9847;

export function loadPitLink(): PitLinkConfig {
  return {
    host: localStorage.getItem(HOST_KEY) ?? "",
    port: Number(localStorage.getItem(PORT_KEY)) || DEFAULT_PORT,
  };
}

export function savePitLink(config: PitLinkConfig) {
  localStorage.setItem(HOST_KEY, config.host.trim());
  localStorage.setItem(PORT_KEY, String(config.port || DEFAULT_PORT));
}

export function pitLinkLabel(config: PitLinkConfig): string {
  if (!config.host) return "Not configured";
  return `${config.host}:${config.port}`;
}
