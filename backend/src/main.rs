mod audit;
mod auth;
mod config;
mod db;
mod feed_search;
mod leo;
mod models;
mod routes;
mod rss_engine;
mod webhooks;

use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;

use crate::config::AppConfig;
use crate::db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub http: Client,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pulsefeed=info,tower_http=warn".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let db = db::init_pool(&config.db_path);
    let http = Client::builder()
        .pool_max_idle_per_host(16)
        .user_agent("PulseFeed-Enterprise/1.0")
        .build()
        .expect("Failed to build HTTP client");

    let state = AppState {
        db: db.clone(),
        http: http.clone(),
        config: config.clone(),
    };

    // Background async RSS polling scheduler
    let bg_http = http.clone();
    let bg_db = db.clone();
    let bg_conc = config.max_concurrent_fetches;
    let bg_poll_secs = config.poll_interval_secs;
    tokio::spawn(async move {
        // Initial feed sync 2 seconds after startup
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let _ = rss_engine::refresh_all_feeds(bg_http.clone(), bg_db.clone(), bg_conc).await;

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(bg_poll_secs));
        loop {
            interval.tick().await;
            let _ = rss_engine::refresh_all_feeds(bg_http.clone(), bg_db.clone(), bg_conc).await;
        }
    });

    let api_router = Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok", "engine": "pulsefeed-rust" })) }))
        .route("/edition", get(|axum::extract::State(st): axum::extract::State<AppState>| async move {
            Json(json!({
                "edition": st.config.edition,
                "is_personal": st.config.is_personal(),
            }))
        }))
        // Auth & OAuth2 Identity Providers (Google, Microsoft Entra ID, Facebook)
        .route("/auth/login", post(routes::auth_routes::login))
        .route("/auth/register", post(routes::auth_routes::register))
        .route("/auth/me", get(routes::auth_routes::me))
        .route("/auth/sso/config", get(routes::auth_routes::sso_public_config))
        .route("/auth/providers", get(routes::oauth_routes::list_public_providers))
        .route("/auth/oauth/{provider}/start", get(routes::oauth_routes::oauth_start))
        .route("/auth/oauth/{provider}/callback", get(routes::oauth_routes::oauth_callback))
        // Sidebar, Folders, Feeds, Topic Search, OPML
        .route("/sidebar", get(routes::feeds_routes::get_sidebar))
        .route("/folders", post(routes::feeds_routes::create_folder))
        .route("/folders/{id}", delete(routes::feeds_routes::delete_folder))
        .route("/feeds/search", get(routes::feeds_routes::search_feeds))
        .route("/feeds/discover", post(routes::feeds_routes::discover_feeds))
        .route("/feeds", post(routes::feeds_routes::create_feed))
        .route("/feeds/refresh-all", post(routes::feeds_routes::refresh_all))
        .route("/feeds/{id}", patch(routes::feeds_routes::update_feed).delete(routes::feeds_routes::delete_feed))
        .route("/feeds/{id}/refresh", post(routes::feeds_routes::refresh_feed))
        .route("/opml/import", post(routes::feeds_routes::import_opml))
        .route("/opml/export", get(routes::feeds_routes::export_opml))
        // Articles, FTS5 Search, Reader State, AI Summary, Highlights, Notes
        .route("/articles", get(routes::articles_routes::list_articles))
        .route("/articles/mark-all-read", post(routes::articles_routes::mark_all_read))
        .route("/articles/{id}", get(routes::articles_routes::get_article))
        .route("/articles/{id}/state", patch(routes::articles_routes::update_article_state))
        .route("/articles/{id}/ai-summary", post(routes::articles_routes::generate_ai_summary))
        .route("/articles/{id}/highlights", post(routes::articles_routes::add_highlight))
        .route("/articles/{id}/highlights/{hid}", delete(routes::articles_routes::delete_highlight))
        .route("/articles/{id}/notes", post(routes::articles_routes::add_note))
        // Team Boards + Public RSS
        .route("/boards", get(routes::enterprise_routes::list_boards).post(routes::enterprise_routes::create_board))
        .route("/boards/{id}", delete(routes::enterprise_routes::delete_board))
        .route("/boards/{id}/articles", post(routes::enterprise_routes::toggle_board_article))
        .route("/public/boards/{token}/rss", get(routes::enterprise_routes::board_public_rss))
        // Leo Intelligence Rules
        .route("/leo/rules", get(routes::enterprise_routes::list_leo_rules).post(routes::enterprise_routes::create_leo_rule))
        .route("/leo/rules/rerun", post(routes::enterprise_routes::rerun_leo_rules))
        .route("/leo/rules/{id}/toggle", post(routes::enterprise_routes::toggle_leo_rule))
        .route("/leo/rules/{id}", delete(routes::enterprise_routes::delete_leo_rule))
        // Webhooks
        .route("/webhooks", get(routes::enterprise_routes::list_webhooks).post(routes::enterprise_routes::create_webhook))
        .route("/webhooks/{id}", delete(routes::enterprise_routes::delete_webhook))
        .route("/webhooks/{id}/test", post(routes::enterprise_routes::test_webhook))
        // Enterprise Admin (Users/RBAC, API Keys, Audit Logs, Settings, Telemetry)
        .route("/admin/users", get(routes::enterprise_routes::list_users).post(routes::enterprise_routes::admin_create_user))
        .route("/admin/users/{id}", patch(routes::enterprise_routes::admin_update_user))
        .route("/admin/api-keys", get(routes::enterprise_routes::list_api_keys).post(routes::enterprise_routes::create_api_key))
        .route("/admin/api-keys/{id}", delete(routes::enterprise_routes::delete_api_key))
        .route("/admin/audit-logs", get(routes::enterprise_routes::list_audit_logs))
        .route("/admin/settings", get(routes::enterprise_routes::get_settings).patch(routes::enterprise_routes::update_settings))
        .route("/admin/telemetry", get(routes::enterprise_routes::get_telemetry));

    let static_path = PathBuf::from(&config.static_dir);
    let index_file = static_path.join("index.html");
    let spa_service = ServeDir::new(&static_path).not_found_service(ServeFile::new(index_file));

    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(spa_service)
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    info!(
        "PulseFeed ({} Edition) listening on http://{}",
        if config.is_personal() { "Personal" } else { "Enterprise" },
        addr
    );
    axum::serve(listener, app).await.expect("Server error");
}
