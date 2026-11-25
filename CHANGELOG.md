# Changelog

All notable changes to **PulseFeed** will be documented in this file.

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
