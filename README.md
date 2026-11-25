# PulseFeed — Enterprise RSS & Intelligence Reader

**PulseFeed** is a blazing-fast, self-hosted **Feedly Enterprise** alternative built with **Rust** (`axum`, `tokio`, embedded **SQLite FTS5**) and **React + TypeScript + Tailwind CSS**. It supports **unlimited RSS, Atom, and JSON feeds** with zero external database setup, enterprise team collaboration, and a hybrid "Leo"-style AI intelligence engine.

---

## Quick Download (Easy Windows Install)

Want to run PulseFeed immediately without installing Rust or Node.js?

1. **Download the latest pre-built ZIP:**
   **[Download PulseFeed v1.0.0 for Windows (x64 ZIP)](https://github.com/chadrothermel-dev/pulsefeed/releases/latest/download/pulsefeed-v1.0.0-windows-x64.zip)**
   *(Or browse all releases on the [Releases page](https://github.com/chadrothermel-dev/pulsefeed/releases).)*
2. **Extract the ZIP** to any folder (for example, `C:\PulseFeed`).
3. **Double-click `start.bat`** (or run `.\start.ps1` in PowerShell).
4. Your browser will open automatically to **http://127.0.0.1:8080**.
5. Sign in with the default administrator account (change or add users inside **Enterprise Console → Users & RBAC**):
   - **Email:** `admin@pulsefeed.local`
   - **Password:** `admin123`

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

### v1.0.0 — 2026-11-24
- Initial public release of **PulseFeed Enterprise RSS & Intelligence Reader**.
- High-efficiency Rust (`axum`/`tokio`) backend with embedded SQLite WAL + FTS5 full-text search.
- Unlimited RSS/Atom/JSON feeds, HTTP 304 conditional caching, website feed auto-discovery, and OPML 2.0 import/export.
- Hybrid **Leo AI Engine** with SimHash deduplication, priority/mute/tag/board/alert rules, and Gemini/OpenAI/Ollama + local extractive summaries.
- Enterprise **Team Boards** (with public RSS output), **Highlights**, **Team Notes**, **RBAC**, **SSO/OIDC settings**, **Slack/Teams/JSON Webhooks**, **API Keys**, **Audit Logs**, and **Telemetry**.
- Modern Feedly-style React UI with Title-Only, Magazine, and Cards views, Dark/Light/Sepia reading modes, and keyboard shortcuts.
- Packaged standalone Windows x64 `.zip` release with one-click `start.bat` / `start.ps1` launchers.

See [CHANGELOG.md](CHANGELOG.md) for full details.
