# PulseFeed — Enterprise RSS & Intelligence Reader

**PulseFeed** is a blazing-fast, self-hosted **Feedly Enterprise** alternative built with **Rust** (`axum`, `tokio`, embedded **SQLite FTS5**) and **React + TypeScript + Tailwind CSS**. It supports **unlimited RSS, Atom, and JSON feeds** with zero external database setup, enterprise team collaboration, and a hybrid "Leo"-style AI intelligence engine.

---

## 1-Click Cloud Deploy

Deploy PulseFeed to the cloud in one click with persistent SQLite storage and automatic HTTPS:

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/chadrothermel-dev/pulsefeed)
[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/new/github/chadrothermel-dev/pulsefeed)

> **Railway Tip:** After deploying on Railway, attach a persistent **Volume** mounted at `/data` and set `PULSEFEED_JWT_SECRET` to a random string so your feeds and users persist across redeploys. (Render mounts `/data` and generates `PULSEFEED_JWT_SECRET` automatically via `render.yaml`.)

### Deploy with Fly.io or Docker Compose (VPS / Home Lab)
```bash
# Option A: Fly.io (uses fly.toml with persistent /data volume)
fly launch --copy-config --no-deploy
fly secrets set PULSEFEED_JWT_SECRET="$(openssl rand -hex 32)"
fly deploy

# Option B: Docker Compose (any Linux VPS or Home Lab — includes optional Caddy HTTPS)
git clone https://github.com/chadrothermel-dev/pulsefeed.git
cd pulsefeed
docker compose up -d
```

---

## Personal vs. Enterprise Editions

PulseFeed ships as a single ultra-fast binary with **two built-in editions**:

| Feature | Personal Edition (Local) | Enterprise Edition (Hosted) |
|---|---|---|
| **Default For** | `start.bat`, `start.ps1`, Desktop Icon | Docker, Render, Railway, Fly.io, `start-enterprise.bat` |
| **Login Screen** | **Zero-login instant access** | Multi-user Argon2 + JWT login & SSO/OIDC |
| **Unlimited Feeds & Topic Search** | Included | Included |
| **Leo AI Rules, Deduplication & Summaries** | Included | Included |
| **Boards, Highlights & Notes** | Personal Saved Boards | Shared Team Boards + Public RSS Output |
| **RBAC, User Management & Audit Logs** | Hidden for simplicity | Full Enterprise Admin Console |

Switch editions anytime via `PULSEFEED_EDITION=personal` or `PULSEFEED_EDITION=enterprise`.

---

## Quick Download (Windows Installer, Portable ZIP & Android APK)

Want to run PulseFeed locally without installing Rust, Node.js, or Android Studio?

### Option 1: Windows Personal Edition Installer (Recommended)
1. **[Download PulseFeed Personal Setup (Windows .exe)](https://github.com/chadrothermel-dev/pulsefeed/releases/latest/download/PulseFeed-Personal-Setup-v1.2.0.exe)**
2. Run the installer — choose your install folder and check **Create a desktop icon** (optional) and **Start Menu shortcut**.
3. Click **Finish** to launch **PulseFeed Personal Edition** immediately with **zero login screen**!

### Option 2: Portable Windows ZIP
- **[Download PulseFeed Portable ZIP (Windows x64)](https://github.com/chadrothermel-dev/pulsefeed/releases/latest/download/pulsefeed-windows-x64.zip)**
- Extract anywhere and double-click `start.bat` (Personal Edition) or `start-enterprise.bat` (Enterprise Edition).

### Option 3: Native Android App (Kotlin + Jetpack Compose)
- **[Download PulseFeed Android APK](https://github.com/chadrothermel-dev/pulsefeed/releases/latest/download/PulseFeed-Android-v1.2.0.apk)** *(or check the [Releases page](https://github.com/chadrothermel-dev/pulsefeed/releases) / [Actions](https://github.com/chadrothermel-dev/pulsefeed/actions))*
- Install the APK on your Android phone (Android 8.0+), enter your hosted PulseFeed URL (e.g., Render/Railway) or your PC's local LAN IP (`http://192.168.x.x:8080`), and start reading with **Leo AI**, **FTS5 Search**, and **Topic Feed Discovery**!
- *(Enterprise Edition only)* Default admin login: `admin@pulsefeed.local` / `admin123`.

> **Note:** On first launch, PulseFeed automatically creates `pulsefeed.db` in the same folder, seeds starter feeds/folders/boards/Leo rules, and begins syncing live articles within 2 seconds.

---

## Key Features

### Unlimited Feeds & High-Speed Sync
- **Unlimited RSS, Atom, and JSON Feeds** with zero subscription caps.
- **Concurrent Async Fetcher** powered by Rust & Tokio with conditional HTTP `304 Not Modified` (`ETag` / `Last-Modified`) caching so unchanged feeds cost almost zero bandwidth or CPU.
- **Feed Auto-Discovery** — paste any website URL and PulseFeed automatically discovers its RSS/Atom/JSON feeds.
- **OPML 2.0 Import & Export** — migrate your entire Feedly or Inoreader library in one click.

### Hybrid "Leo" AI Intelligence Engine
- **Smart Deduplication & Clustering:** 64-bit SimHash locality-sensitive hashing automatically groups syndicated or duplicate stories across feeds.
- **Automation Rules:** Create rules using keywords or `/regex/` to:
  - **Prioritize** high-signal articles (0–100 Leo Priority Score)
  - **Mute** noise, promos, or unwanted topics
  - **Auto-Tag** articles
  - **Auto-Curate** matching articles into Team Boards
  - **Trigger Real-Time Alerts** to Slack, Microsoft Teams, or custom Webhooks
- **AI Summaries & Sentiment:** One-click 3-bullet executive takeaways and tone classification. Works **100% locally** out of the box, or connect **Google Gemini**, **OpenAI**, or a local **Ollama** instance in **Enterprise Console → LLM & SSO**.

### Enterprise Collaboration & Governance
- **Team Boards:** Curate shared collections with **Private**, **Team**, or **Public** visibility. Public boards automatically generate a live **Public RSS Feed URL** for newsletters, intranets, or downstream tools.
- **Shared Highlights & Team Notes:** Highlight quotes and discuss articles directly inside the Reader.
- **Multi-User Auth & RBAC:** Argon2 password hashing, JWT sessions, and 4 roles (`admin`, `editor`, `member`, `viewer`).
- **SSO / OIDC Ready:** Built-in configuration panel for Microsoft Entra ID, Okta, or Google Workspace.
- **Webhooks & API Keys:** Native Slack & Microsoft Teams alert formatting, signed generic JSON webhooks, and `pf_...` Bearer API keys for SIEM/script automation.
- **Immutable Audit Logs & Telemetry:** Track logins, feed changes, rule updates, API keys, feed health, and SQLite storage metrics.

### User-Friendly Reading Experience
- **Instant Full-Text Search** powered by SQLite **FTS5** (Porter stemmer + Unicode61).
- **3 Feedly-Inspired Layouts:** **Title-Only**, **Magazine**, and **Cards**.
- **Reader Comfort:** **Dark**, **Light**, and **Sepia** reading modes with Serif/Sans font toggle.
- **Keyboard Shortcuts:** `j`/`k` (next/prev), `m` (read/unread), `l` (read later), `v` (open original), `/` (search), `?` (shortcuts help), `Esc` (close).

---

## Install from Source (Windows / macOS / Linux)

### Prerequisites
- **Rust** 1.85+ (`rustc` and `cargo`)
- **Node.js** 20+ (`node` and `npm`)

### Build & Run
```bash
# 1. Clone the repository
git clone https://github.com/chadrothermel-dev/pulsefeed.git
cd pulsefeed

# 2. Build the React frontend
cd frontend
npm install
npm run build

# 3. Build the optimized Rust backend
cd ../backend
cargo build --release

# 4. Run PulseFeed (serves both API and UI on http://127.0.0.1:8080)
cd ..
# Windows PowerShell:
.\start.ps1

# macOS / Linux:
PULSEFEED_STATIC_DIR=./frontend/dist ./backend/target/release/pulsefeed
```

---

## Configuration (Environment Variables)

| Variable | Default | Description |
|---|---|---|
| `PULSEFEED_HOST` | `127.0.0.1` | Bind address (`0.0.0.0` to expose on your LAN) |
| `PULSEFEED_PORT` | `8080` | HTTP server port |
| `PULSEFEED_DB` | `pulsefeed.db` | Path to the SQLite database file |
| `PULSEFEED_STATIC_DIR` | `../frontend/dist` | Path to compiled frontend static assets |
| `PULSEFEED_JWT_SECRET` | *(dev default)* | Secret key used to sign JWTs (**set this in production**) |
| `PULSEFEED_CONCURRENCY` | `24` | Maximum concurrent feed fetches |
| `PULSEFEED_POLL_SECS` | `300` | Background feed refresh interval in seconds |

---

## Changelog

### v1.1.0 — 2026-11-26
- Added **Personal Edition** (`PULSEFEED_EDITION=personal`, zero-login local default) alongside **Enterprise Edition** (`PULSEFEED_EDITION=enterprise`, hosted cloud default).
- Added **Search by Topic** feed finder with curated category browsing, live web feed search, and automatic Google News / Reddit topic RSS feeds.
- Added `start-enterprise.bat` alongside `start.bat` (Personal Edition default).

### v1.0.2 — 2026-11-24
- Added **1-Click Cloud Deploy** buttons for **Render** (`render.yaml`) and **Railway** (`railway.toml`), plus **Fly.io** (`fly.toml`) and **Docker Compose + Caddy HTTPS** (`Dockerfile`, `docker-compose.yml`, `Caddyfile`).
- Added automatic `PORT` environment variable fallback alongside `PULSEFEED_PORT` for seamless PaaS deployments.

### v1.0.1 — 2026-11-24
- `start.bat` and `start.ps1` now automatically create **Desktop** and **Start Menu** shortcuts with the custom `pulsefeed.ico` icon.
- Added silent background launcher (`launch-app.ps1`) that opens PulseFeed in Chrome App Mode when launched from Desktop/Taskbar.
- Updated standalone Windows ZIP asset to `pulsefeed-windows-x64.zip` with icon and shortcut scripts bundled.

### v1.0.0 — 2026-11-24
- Initial public release of **PulseFeed Enterprise RSS & Intelligence Reader**.
- High-efficiency Rust (`axum`/`tokio`) backend with embedded SQLite WAL + FTS5 full-text search.
- Unlimited RSS/Atom/JSON feeds, HTTP 304 conditional caching, website feed auto-discovery, and OPML 2.0 import/export.
- Hybrid **Leo AI Engine** with SimHash deduplication, priority/mute/tag/board/alert rules, and Gemini/OpenAI/Ollama + local extractive summaries.
- Enterprise **Team Boards** (with public RSS output), **Highlights**, **Team Notes**, **RBAC**, **SSO/OIDC settings**, **Slack/Teams/JSON Webhooks**, **API Keys**, **Audit Logs**, and **Telemetry**.
- Modern Feedly-style React UI with Title-Only, Magazine, and Cards views, Dark/Light/Sepia reading modes, and keyboard shortcuts.
- Packaged standalone Windows x64 `.zip` release with one-click `start.bat` / `start.ps1` launchers.

See [CHANGELOG.md](CHANGELOG.md) for full details.
