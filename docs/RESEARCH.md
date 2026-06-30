# Research: AC League Management via Content Manager

*Last updated: June 2026*

## Executive summary

**You cannot build a "Content Manager app" in the traditional sense.** CM (by x4fab) is closed-source .NET software with no public SDK for league-management extensions. What you *can* build is a **league management service that speaks Content Manager's join and server-detail protocols** — giving drivers a one-click path from your web UI into CM, with mods pre-resolved.

The highest-leverage integration points are:

1. **CM join URLs** (`acstuff.ru/s/q:race/online/join`)
2. **Content Manager Wrapper (CMW)** or **AssettoServer Server Details**
3. **Server config metadata** (`[__CM_SERVER]`, mod download URLs)
4. **Optional**: Assetto Server Manager / ACSR for championship back-end

---

## Content Manager overview

[Content Manager](https://acstuff.club/app/) replaces AC's stock launcher. Drivers use it to:

- Browse and join online servers
- Install missing cars/tracks from server-provided URLs
- Manage mods, setups, CSP, and server presets

### What CM is NOT

| Misconception | Reality |
|---------------|---------|
| "CM has a league plugin API" | No public API for third-party league UIs inside CM |
| "Python apps = league tools" | Python apps are **in-game overlays** (`/apps/python/`), not launcher extensions |
| "CM plugins" exist for everything | CM plugins (Discord presence, stats) are limited; no documented dev kit for league features |

### What CM IS good at (for our use case)

- **Join flow**: URL → CM opens → server selected → password filled
- **Content sync**: Server advertises required mods; CM offers "Install missing content"
- **Server browser UX**: Rich descriptions, images, links when CMW/Server Details are enabled

---

## Join mechanisms

### 1. CM join links (primary)

Standard format used by Server Manager, AssettoServer, and community tools:

```
https://acstuff.ru/s/q:race/online/join?ip=<PUBLIC_IP>&httpPort=<HTTP_PORT>
```

Optional query params (community-documented):

- `password` — pre-fill server password
- Additional params may be passed depending on server stack

**Example** (from AssettoServer logs):

```
https://acstuff.ru/s/q:race/online/join?ip=100.101.102.103&httpPort=8081
```

**Driver flow**: Click link in browser → CM launches → connects to server.

### 2. `acmanager://` protocol

Custom URL scheme registered by Content Manager on Windows. Used by platforms like [Rorzone](https://ror.zone/faq).

**Linux caveat**: Protocol handler often broken. Workarounds:

- Drag join button from browser into open CM window
- Custom launcher script: `steam -applaunch 244210 "$LINK"`

### 3. Manual server browser

Drivers search by server name/domain in CM → Online. Works but is the worst UX for leagues (password typos, wrong server among duplicates).

**LeagueManager should always prefer join links** on event pages, Discord bots, and email reminders.

---

## Content Manager Wrapper (CMW)

[Documented by Assetto Server Manager](https://github.com/JustaPenguin/assetto-server-manager/wiki/Sharing-Content-With-Content-Manager-Wrapper).

### How it works

1. A small HTTP server runs alongside the AC server (separate port).
2. The AC server name is suffixed with a special separator + wrapper port: `ℹ<port>` (Unicode info character).
3. CM parses the server name, contacts the wrapper port, and fetches:
   - Markdown/rich description (rules, standings snippet, schedule)
   - Car/track download URLs
   - Championship context

### Join link generation (Server Manager reference)

```go
ContentManagerJoinLinkBase = "https://acstuff.ru/s/q:race/online/join"
// Query: ip, httpPort (from server HTTP API port)
```

Source: [content_manager_wrapper.go](https://github.com/JustaPenguin/assetto-server-manager/blob/master/content_manager_wrapper.go)

### Admin setup (Server Manager)

1. Enable **Content Manager Integration** in Server Options (pick unused port).
2. Set **Download URL** on each car/track in content metadata.
3. Drivers see **Install missing content** in CM when mods are missing.

### LeagueManager opportunity

Implement a **CMW-compatible HTTP endpoint** in our backend so self-hosted leagues get the same UX without requiring full Server Manager — or proxy to Server Manager when present.

---

## Native CM Server Details (`[__CM_SERVER]`)

When hosting via CM's Server tab (or editing `server_cfg.ini`):

```ini
[SERVER]
NAME=My League Round 3 x:poQ6P

[__CM_SERVER]
NAME=My League Round 3
DETAILS_ID=poQ6P
```

- `DETAILS_ID` must match CM's Details view — changes when mod URLs are updated.
- CM can enforce **mod version** requirements and per-skin download URLs.
- Share mode "Download URL" per car/track enables one-click installs.

Guide: [OverTake — Download missing content](https://www.overtake.gg/threads/download-missing-car-track-server-content.165183/)

**LeagueManager** can generate/patch these INI sections when scheduling an event.

---

## AssettoServer Server Details (modern path)

[AssettoServer docs](https://assettoserver.org/docs/misc/server-details/):

- Set `EnableServerDetails: true` in `extra_cfg.yml`
- Put description in `ServerDescription`
- **Do not** also enable CM's "Provide extra details" — conflicts
- Server name suffix (`x:abcd` or `ℹport`) appended automatically

AssettoServer also logs invite links on startup and supports **C# plugins** with HTTP routes (AGPL — plugins must be open source).

---

## Existing league / server platforms

### Assetto Server Manager (ACSM)

- **GitHub**: [JustaPenguin/assetto-server-manager](https://github.com/JustaPenguin/assetto-server-manager) (Go, open source base)
- **Features**: Championships, race weekends, sign-up forms, live timings, results, CMW, content upload
- **Premium**: Custom AC server, multi-server, ACSR, Lua hooks, race weekends advanced features
- **Best for**: Self-hosted league with full server control

**LeagueManager could**: Complement ACSM (nicer driver-facing portal) or fork/integrate via its APIs.

### ACSR (Assetto Corsa Skill Ratings)

- **Site**: [acsr.assettocorsaservers.com](https://acsr.assettocorsaservers.com/)
- Championship listing, driver skill/safety ratings, sign-up forms
- Requires Server Manager Premium + API key
- API example: `GET /api/championship/{id}/standings.json`

**LeagueManager could**: Sync standings to/from ACSR or replace discovery layer for indie leagues.

### Rorzone

- **Site**: [ror.zone](https://ror.zone/)
- Hosted servers, Steam login, join codes, CM deep links
- **Reference UX** for "click Join → CM → on track" simplicity
- Not self-hosted; limited league admin vs ACSM

### Community patterns (ROS, Lega Corsa, etc.)

- Website sign-up + Discord
- Event page lists CM join link + password
- Pre-qualifying on dedicated servers
- Results on web / live timing pages

**Pain point our app targets**: Everything except the last-mile CM join is manual.

---

## In-game Python apps (out of scope for league admin)

AC Python apps (`acMain`, `acUpdate`, OpenGL overlays) run **inside the sim**, not in CM's launcher. Useful for:

- Delta timers, radar, penalties (Real Penalty, Sidekick, Helicorsa)

Not suitable for championship sign-up or server joining.

Template: [huntervaners/Template_Assetto_Corsa_App](https://github.com/huntervaners/Template_Assetto_Corsa_App)

---

## Gap analysis: what a new LeagueManager should solve

| Need | CM native | ACSM | Rorzone | **LeagueManager target** |
|------|-----------|------|---------|---------------------------|
| One-click join | ✅ | ✅ | ✅ | ✅ Generate links per event |
| Mod auto-install | ✅ (with CMW/details) | ✅ | Partial | ✅ Curate mod pack per season |
| Championship standings | ❌ | ✅ | ❌ | ✅ |
| Driver sign-up / waitlist | ❌ | ✅ (forms) | ❌ | ✅ Modern UX |
| Calendar / reminders | ❌ | Partial | ❌ | ✅ |
| Multi-division leagues | ❌ | ✅ | ❌ | ✅ |
| Self-hosted | N/A | ✅ | ❌ | ✅ |
| Works without SM Premium | N/A | Partial | N/A | ✅ |

---

## Technical risks

| Risk | Mitigation |
|------|------------|
| CM join links fail on Linux | Document drag-to-CM + launcher script; test Steam Proton setup |
| CMW port blocked by firewall | Document required ports; fallback to description-only |
| Mod licensing / distribution | League provides URLs only; no bundling without permission |
| Server Manager Premium lock-in for ACSR | Treat ACSR as optional integration |
| AGPL (AssettoServer plugins) | Keep server plugins in separate OSS repo if used |

---

## Key URLs

- Content Manager: https://acstuff.club/app/
- CM join base: https://acstuff.ru/s/q:race/online/join
- Assetto Server Manager: https://github.com/JustaPenguin/assetto-server-manager
- AssettoServer: https://github.com/compujuckel/AssettoServer
- AssettoServer docs: https://assettoserver.org/docs/
- ACSR: https://acsr.assettocorsaservers.com/
- CMW wiki: https://github.com/JustaPenguin/assetto-server-manager/wiki/Sharing-Content-With-Content-Manager-Wrapper
