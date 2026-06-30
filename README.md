# LeagueManager

A **Windows desktop app** for **Assetto Corsa** league hosting and participation via **Content Manager**.

## Status

**Phase 0** — Tauri + Svelte shell, Steam login, SQLite, host/driver modes.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/)
- Windows build targets (primary); Linux dev needs [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Run locally

```bash
npm install
npm run tauri dev
```

### Steam API key (avatars & display names)

Set for real Steam profile data:

```bash
export LEAGUE_MANAGER_STEAM_API_KEY=your_steam_web_api_key
```

Without it, dev login uses placeholder avatars.

### Dev login

On the login screen in dev mode, use **Quick dev sign-in** with any SteamID64 (no browser).

## Docs

| Document | Contents |
|----------|----------|
| [PLAN.md](docs/PLAN.md) | Full build plan |
| [RESEARCH.md](docs/RESEARCH.md) | CM / ecosystem research |

## Stack

- **Tauri 2** + **Svelte 5** + **Tailwind CSS 4**
- **SQLite** (host)
- **AssettoServer** (Phase 1+)

## License

TBD
