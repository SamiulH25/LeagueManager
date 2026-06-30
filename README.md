# LeagueManager

A **Windows desktop app** for **Assetto Corsa** league hosting and participation via **Content Manager**.

## Status

**Phase 0** — Tauri + Svelte shell, Steam login, SQLite, host/driver modes.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/)
- **Windows** — primary target for release builds
- **Linux (Nobara / Fedora)** — fine for UI dev; install build deps below

#### Linux dev setup (Nobara / Fedora 43)

Tauri needs **development headers**, not just the runtime libraries. If you see errors like `javascriptcoregtk-4.1 was not found` or `webkit2gtk-4.1 was not found`, install:

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  javascriptcoregtk4.1-devel \
  libsoup3-devel \
  gtk3-devel \
  openssl-devel \
  curl wget file \
  librsvg2-devel \
  libappindicator-gtk3-devel
```

Then retry:

```bash
npm run tauri dev
```

See also: [Tauri prerequisites](https://tauri.app/start/prerequisites/)

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
