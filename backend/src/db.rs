use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
            PRAGMA busy_timeout = 5000;
            ",
        )
    });

    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .expect("Failed to create SQLite connection pool");

    {
        let conn = pool.get().expect("Failed to acquire DB connection");
        run_migrations(&conn).expect("Failed to run database migrations");
        seed_initial_data(&conn).expect("Failed to seed initial workspace");
    }

    pool
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            key_prefix TEXT NOT NULL,
            key_hash TEXT NOT NULL UNIQUE,
            scopes TEXT NOT NULL DEFAULT 'read,write',
            last_used_at TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#10b981',
            icon TEXT NOT NULL DEFAULT 'folder',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS feeds (
            id TEXT PRIMARY KEY,
            folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
            title TEXT NOT NULL,
            feed_url TEXT NOT NULL UNIQUE,
            site_url TEXT,
            description TEXT,
            icon_url TEXT,
            etag TEXT,
            last_modified TEXT,
            refresh_interval_mins INTEGER NOT NULL DEFAULT 15,
            last_fetched_at TEXT,
            next_fetch_at TEXT,
            last_error TEXT,
            error_count INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS articles (
            id TEXT PRIMARY KEY,
            feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
            guid TEXT NOT NULL,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            author TEXT,
            summary TEXT NOT NULL DEFAULT '',
            content_html TEXT NOT NULL DEFAULT '',
            content_text TEXT NOT NULL DEFAULT '',
            image_url TEXT,
            published_at TEXT NOT NULL,
            fetched_at TEXT NOT NULL,
            reading_time_mins INTEGER NOT NULL DEFAULT 1,
            simhash INTEGER NOT NULL DEFAULT 0,
            duplicate_cluster_id TEXT,
            priority_score INTEGER NOT NULL DEFAULT 50,
            is_muted INTEGER NOT NULL DEFAULT 0,
            tags_json TEXT NOT NULL DEFAULT '[]',
            ai_summary TEXT,
            ai_sentiment TEXT,
            UNIQUE(feed_id, guid)
        );

        CREATE INDEX IF NOT EXISTS idx_articles_feed_pub ON articles(feed_id, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_articles_pub ON articles(published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_articles_priority ON articles(priority_score DESC, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_articles_cluster ON articles(duplicate_cluster_id);
        CREATE INDEX IF NOT EXISTS idx_articles_simhash ON articles(simhash);

        CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
            title,
            content_text,
            author,
            tags_json,
            content='articles',
            content_rowid='rowid',
            tokenize='porter unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
            INSERT INTO articles_fts(rowid, title, content_text, author, tags_json)
            VALUES (new.rowid, new.title, new.content_text, COALESCE(new.author, ''), new.tags_json);
        END;

        CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, content_text, author, tags_json)
            VALUES ('delete', old.rowid, old.title, old.content_text, COALESCE(old.author, ''), old.tags_json);
        END;

        CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, content_text, author, tags_json)
            VALUES ('delete', old.rowid, old.title, old.content_text, COALESCE(old.author, ''), old.tags_json);
            INSERT INTO articles_fts(rowid, title, content_text, author, tags_json)
            VALUES (new.rowid, new.title, new.content_text, COALESCE(new.author, ''), new.tags_json);
        END;

        CREATE TABLE IF NOT EXISTS user_article_state (
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
            is_read INTEGER NOT NULL DEFAULT 0,
            is_read_later INTEGER NOT NULL DEFAULT 0,
            read_at TEXT,
            PRIMARY KEY (user_id, article_id)
        );
        CREATE INDEX IF NOT EXISTS idx_uas_user_read ON user_article_state(user_id, is_read);
        CREATE INDEX IF NOT EXISTS idx_uas_user_later ON user_article_state(user_id, is_read_later);

        CREATE TABLE IF NOT EXISTS boards (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            visibility TEXT NOT NULL DEFAULT 'team',
            color TEXT NOT NULL DEFAULT '#3b82f6',
            created_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            public_token TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS board_articles (
            board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
            article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
            added_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            added_at TEXT NOT NULL,
            PRIMARY KEY (board_id, article_id)
        );

        CREATE TABLE IF NOT EXISTS highlights (
            id TEXT PRIMARY KEY,
            article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            selected_text TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'yellow',
            note TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS article_notes (
            id TEXT PRIMARY KEY,
            article_id TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS leo_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            rule_type TEXT NOT NULL,
            keywords TEXT NOT NULL,
            match_field TEXT NOT NULL DEFAULT 'any',
            feed_id TEXT REFERENCES feeds(id) ON DELETE SET NULL,
            folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
            score_delta INTEGER NOT NULL DEFAULT 25,
            target_tag TEXT,
            target_board_id TEXT REFERENCES boards(id) ON DELETE SET NULL,
            target_webhook_id TEXT REFERENCES webhooks(id) ON DELETE SET NULL,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            match_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS webhooks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'generic',
            url TEXT NOT NULL,
            secret TEXT,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            last_status TEXT,
            last_triggered_at TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            user_email TEXT,
            action TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT,
            details TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at DESC);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

fn seed_initial_data(conn: &Connection) -> rusqlite::Result<()> {
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
    if user_count > 0 {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    let admin_id = Uuid::new_v4().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(b"admin123", &salt)
        .expect("Failed to hash default admin password")
        .to_string();

    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, role, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, 'admin', 1, ?5)",
        params![admin_id, "admin@pulsefeed.local", "Workspace Admin", password_hash, now],
    )?;

    // Default settings
    let defaults = [
        ("llm_provider", "none"),
        ("llm_model", "gemini-2.5-flash"),
        ("llm_api_key", ""),
        ("llm_base_url", ""),
        ("sso_enabled", "false"),
        ("sso_provider_name", "Microsoft Entra ID / OIDC"),
        ("sso_issuer_url", ""),
        ("sso_client_id", ""),
        ("dedup_threshold", "6"),
        ("default_poll_mins", "15"),
    ];
    for (k, v) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![k, v],
        )?;
    }

    // Default folders
    let f_tech = Uuid::new_v4().to_string();
    let f_sec = Uuid::new_v4().to_string();
    let f_ai = Uuid::new_v4().to_string();
    let f_cloud = Uuid::new_v4().to_string();

    let folders = [
        (&f_tech, "Tech & Engineering", "#10b981", "cpu", 1),
        (&f_sec, "Cybersecurity", "#ef4444", "shield", 2),
        (&f_ai, "AI & Research", "#8b5cf6", "sparkles", 3),
        (&f_cloud, "Cloud & Infra", "#3b82f6", "cloud", 4),
    ];
    for (id, name, color, icon, order) in folders {
        conn.execute(
            "INSERT INTO folders (id, name, color, icon, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, color, icon, order, now],
        )?;
    }

    // Starter high-signal RSS feeds
    let starter_feeds = [
        (&f_tech, "Hacker News (Front Page)", "https://hnrss.org/frontpage", "https://news.ycombinator.com"),
        (&f_tech, "The Rust Programming Language Blog", "https://blog.rust-lang.org/feed.xml", "https://blog.rust-lang.org"),
        (&f_tech, "Ars Technica", "https://feeds.arstechnica.com/arstechnica/index", "https://arstechnica.com"),
        (&f_sec, "Krebs on Security", "https://krebsonsecurity.com/feed/", "https://krebsonsecurity.com"),
        (&f_sec, "BleepingComputer", "https://www.bleepingcomputer.com/feed/", "https://www.bleepingcomputer.com"),
        (&f_ai, "MIT Technology Review", "https://www.technologyreview.com/feed/", "https://www.technologyreview.com"),
        (&f_cloud, "Cloudflare Blog", "https://blog.cloudflare.com/rss/", "https://blog.cloudflare.com"),
        (&f_cloud, "The Verge", "https://www.theverge.com/rss/index.xml", "https://www.theverge.com"),
    ];
    for (folder_id, title, feed_url, site_url) in starter_feeds {
        conn.execute(
            "INSERT INTO feeds (id, folder_id, title, feed_url, site_url, refresh_interval_mins, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 15, 1, ?6)",
            params![Uuid::new_v4().to_string(), folder_id, title, feed_url, site_url, now],
        )?;
    }

    // Default Team Boards
    let b_exec = Uuid::new_v4().to_string();
    let b_sec = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO boards (id, name, description, visibility, color, created_by, public_token, created_at)
         VALUES (?1, 'Executive Briefing', 'Curated strategic articles for leadership weekly review.', 'team', '#10b981', ?2, ?3, ?4)",
        params![b_exec, admin_id, Uuid::new_v4().to_string(), now],
    )?;
    conn.execute(
        "INSERT INTO boards (id, name, description, visibility, color, created_by, public_token, created_at)
         VALUES (?1, 'Security Advisories', 'Critical vulnerabilities, zero-days, and threat intel.', 'public', '#ef4444', ?2, ?3, ?4)",
        params![b_sec, admin_id, Uuid::new_v4().to_string(), now],
    )?;

    // Default Leo Intelligence Rules
    conn.execute(
        "INSERT INTO leo_rules (id, name, rule_type, keywords, match_field, score_delta, target_tag, target_board_id, is_enabled, created_at)
         VALUES (?1, 'Critical Security & CVEs', 'priority', 'CVE-, zero-day, ransomware, vulnerability, exploit, breach', 'any', 35, 'security-alert', ?2, 1, ?3)",
        params![Uuid::new_v4().to_string(), b_sec, now],
    )?;
    conn.execute(
        "INSERT INTO leo_rules (id, name, rule_type, keywords, match_field, score_delta, target_tag, is_enabled, created_at)
         VALUES (?1, 'AI & LLM Releases', 'priority', 'LLM, Gemini, OpenAI, Anthropic, Claude, benchmark, neural', 'any', 25, 'ai', 1, ?2)",
        params![Uuid::new_v4().to_string(), now],
    )?;
    conn.execute(
        "INSERT INTO leo_rules (id, name, rule_type, keywords, match_field, score_delta, target_tag, is_enabled, created_at)
         VALUES (?1, 'Mute Sponsored & Deals', 'mute', 'sponsored, promo code, black friday deal, affiliate', 'title', -50, NULL, 1, ?2)",
        params![Uuid::new_v4().to_string(), now],
    )?;

    crate::audit::log_audit(
        conn,
        Some(&admin_id),
        Some("admin@pulsefeed.local"),
        "workspace.initialized",
        "system",
        None,
        "Initialized PulseFeed enterprise workspace with starter feeds, boards, and Leo rules",
    );

    Ok(())
}
