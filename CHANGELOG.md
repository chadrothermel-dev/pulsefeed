# Changelog

All notable changes to **PulseFeed** will be documented in this file.

## [1.3.0] - 2026-11-26

### Added
- **100% Standalone Local Android App (No Server Required!):**
  - Embedded on-device SQLite database (`LocalDbHelper`) and native async RSS/Atom parser (`LocalPulseFeedEngine`) directly inside the Android APK.
  - Launches straight into **Personal On-Device Local Mode** with 6 starter feeds, automatic initial sync, on-device Leo AI priority scoring, 1-click extractive AI summaries, **Discover / Search by Topic** feed finder (curated directory + live Feedly search + Google News/Reddit topic feeds), and a **Feeds** management tab to unfollow/remove feeds.
  - Keeps optional **Connect to Remote Enterprise Server** mode in Settings for hosted deployments.

## [1.2.2] - 2026-11-26

### Fixed
- **Personal Edition Branding:** Updated window/tab title (`PulseFeed Personal — RSS & Intelligence Reader`) and removed all remaining "Enterprise" / "Team Board" / "Team Notes" labels when running in Personal Edition.
- **Clean Upgrade/Restart on Install:** `PulseFeed-Personal-Setup` now automatically stops any previously running `pulsefeed.exe` before installing so the updated Personal Edition binary and UI (including Topic Feed Search and Delete controls) load cleanly right after setup.

## [1.2.1] - 2026-11-26

### Added
- **Remove / Unfollow Feeds, Folders, and Boards:** Added hover trash icons in the sidebar for every Feed, Folder, and Board, plus **Unfollow Feed**, **Delete Board**, and **Delete Folder** buttons in the top header bar with confirmation prompts.

### Fixed
- Replaced inline-generated `launch-app.vbs` with a clean, self-locating portable VBS launcher so the Windows Installer shortcut launches cleanly from any directory.

## [1.2.0] - 2026-11-26

### Added
- **Windows Personal Edition Installer (`PulseFeed-Personal-Setup-v1.2.0.exe`):**
  - Built with Inno Setup 6 (`installer/pulsefeed-personal.iss`).
  - Interactive Windows Setup Wizard with optional **Create a desktop icon** and **Create a Start Menu shortcut** checkboxes, custom emerald icon, automatic silent launcher setup, and proper Windows Add/Remove Programs uninstaller.
- **Native Android App (`android/`):**
  - Built in Kotlin + Jetpack Compose (Material 3) with adaptive emerald RSS launcher icon.
  - Connects to either your local **Personal Edition** PC (zero login) or hosted **Enterprise Edition** server (JWT login).
  - Features Today, Leo AI Priority, Saved/Read Later, **Discover / Search by Topic** feed finder, full-text search, and native Reader with 1-click Leo AI summaries.
- **Automated Android APK Builds (`.github/workflows/build-android.yml`):**
  - GitHub Actions automatically compiles the Android APK and attaches `PulseFeed-Android-v1.2.0.apk` to GitHub Releases.

## [1.1.0] - 2026-11-26

### Added
- **Personal Edition (Local) vs. Enterprise Edition (Hosted):**
  - Controlled via `PULSEFEED_EDITION=personal` (default for local `start.bat`, `start.ps1`, and Desktop shortcut) or `PULSEFEED_EDITION=enterprise` (default for `Dockerfile`, Render, Railway, Fly.io, Docker Compose, and `start-enterprise.bat`).
  - **Personal Edition** launches directly into your feeds with **zero login screen**, a clean local interface (`Saved Boards`, `Settings & Health`), unlimited feeds, Leo AI rules, deduplication, highlights, and optional LLM summaries.
  - **Enterprise Edition** preserves full multi-user authentication, RBAC (`admin`/`editor`/`member`/`viewer`), Team Boards with Public RSS, SSO/OIDC, Webhooks, API Keys, and Audit Logs.
- **Topic-Based RSS Feed Search (`GET /api/feeds/search`):**
  - Upgraded the **Follow** modal with a **Search by Topic** tab featuring 10+ category browse pills (Tech, Programming, Cybersecurity, AI & ML, Cloud & Infra, World News, Finance, Science, Sports, Gaming), a built-in curated directory, live web feed search, and automatic Google News & Reddit live topic RSS feeds so you can follow *any* subject in one click.

## [1.0.2] - 2026-11-24

### Added
- **1-Click Cloud Deployment:** Added `render.yaml` (auto-mounts persistent `/data` disk & generates JWT secret), `railway.toml`, `fly.toml`, multi-stage `Dockerfile`, `.dockerignore`, `docker-compose.yml`, and `Caddyfile` for automatic HTTPS.
- Added 1-Click **Deploy to Render** and **Deploy on Railway** buttons plus Fly.io and Docker Compose instructions to `README.md`.
- Added `PORT` environment variable fallback in `backend/src/config.rs` for zero-config PaaS port binding.

## [1.0.1] - 2026-11-24

### Added
- **Automatic Desktop & Start Menu Shortcuts:** Running `start.bat` or `start.ps1` now automatically creates **PulseFeed** shortcuts on your Desktop and Start Menu with the custom emerald `pulsefeed.ico` icon (`install-shortcuts.ps1`).
- **Silent App-Mode Launcher (`launch-app.ps1`):** Clicking the Desktop or Start Menu icon silently starts `pulsefeed.exe` in the background (if not already running) and opens PulseFeed in Chrome App Mode for a native desktop app feel.
- **Stable Latest ZIP Download Asset:** Release package now uses `pulsefeed-windows-x64.zip` so the `latest` download link stays permanent across future updates.

## [1.0.0] - 2026-11-24

### Added
- **High-Efficiency Rust Engine (`axum` + `tokio` + embedded SQLite FTS5):**
  - Single native binary with zero external database dependencies.
  - SQLite configured in WAL mode with 64 MB RAM cache, 256 MB memory-mapped I/O, and FTS5 Porter-stemmer full-text search index.
  - Concurrent async RSS/Atom/JSON Feed fetcher (`feed-rs`) with conditional `ETag` / `Last-Modified` HTTP 304 caching and website feed auto-discovery.
  - Streaming **OPML 2.0 Import & Export** for migrating unlimited feeds from Feedly, Inoreader, or other readers.
- **Hybrid "Leo" Intelligence Engine:**
  - 64-bit **SimHash** locality-sensitive hashing for near-duplicate detection and multi-source story clustering.
  - Custom automation rules supporting comma-separated keywords and `/regex/`: Priority Boost, Mute Filters, Auto-Tag, Auto-Save to Team Board, and Real-Time Webhook Alerts.
  - One-click **AI Summary & Sentiment** supporting Google Gemini, OpenAI, and local Ollama, with an automatic offline local extractive fallback.
- **Enterprise Collaboration & Governance:**
  - **Team Boards** (Private, Team-shared, and Public with auto-generated shareable **Public RSS Feed URLs**).
  - **Shared Highlights** and **Team Discussion Notes** on articles.
  - Multi-user authentication (**Argon2** + **JWT**) with **RBAC** (`admin`, `editor`, `member`, `viewer`).
  - **Enterprise Admin Console** featuring User/RBAC management, **Slack / Microsoft Teams / Generic JSON Webhooks**, **API Keys (`pf_...`)**, **SSO / OIDC configuration**, immutable **Audit Logs**, and **System Telemetry**.
- **Feedly-Style Modern UI (React + TypeScript + Tailwind CSS):**
  - **Title-Only**, **Magazine**, and **Cards** layouts.
  - **Dark**, **Light**, and **Sepia Reader Mode** with Serif/Sans typography toggle.
  - Full keyboard shortcuts (`j`/`k`, `m`, `l`, `v`, `/`, `?`, `Esc`).
  - Pre-seeded starter workspace with 8 curated tech/security/AI feeds, 2 Team Boards, and 3 starter Leo rules.
  - One-click Windows release ZIP packaging (`pulsefeed-v1.0.0-windows-x64.zip`) with `start.bat` and `start.ps1`.
