use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String, // "admin" | "editor" | "member" | "viewer"
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub sort_order: i64,
    pub unread_count: i64,
    pub feed_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feed {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub refresh_interval_mins: i64,
    pub last_fetched_at: Option<String>,
    pub last_error: Option<String>,
    pub error_count: i64,
    pub unread_count: i64,
    pub total_articles: i64,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleListItem {
    pub id: String,
    pub feed_id: String,
    pub feed_title: String,
    pub feed_icon: Option<String>,
    pub folder_id: Option<String>,
    pub title: String,
    pub url: String,
    pub author: Option<String>,
    pub summary: String,
    pub image_url: Option<String>,
    pub published_at: String,
    pub reading_time_mins: i64,
    pub priority_score: i64,
    pub is_muted: bool,
    pub is_read: bool,
    pub is_read_later: bool,
    pub tags: Vec<String>,
    pub duplicate_count: i64,
    pub duplicate_cluster_id: Option<String>,
    pub board_ids: Vec<String>,
    pub highlight_count: i64,
    pub note_count: i64,
    pub ai_summary: Option<String>,
    pub ai_sentiment: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleDetail {
    #[serde(flatten)]
    pub item: ArticleListItem,
    pub content_html: String,
    pub content_text: String,
    pub highlights: Vec<Highlight>,
    pub notes: Vec<ArticleNote>,
    pub duplicates: Vec<DuplicateBrief>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateBrief {
    pub id: String,
    pub title: String,
    pub url: String,
    pub feed_title: String,
    pub published_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub id: String,
    pub article_id: String,
    pub user_id: String,
    pub user_name: String,
    pub selected_text: String,
    pub color: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleNote {
    pub id: String,
    pub article_id: String,
    pub user_id: String,
    pub user_name: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamBoard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub visibility: String, // "private" | "team" | "public"
    pub color: String,
    pub created_by: String,
    pub creator_name: String,
    pub article_count: i64,
    pub public_token: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeoRule {
    pub id: String,
    pub name: String,
    pub rule_type: String, // "priority" | "mute" | "auto_tag" | "auto_board" | "alert"
    pub keywords: String,  // comma-separated keywords or /regex/
    pub match_field: String, // "any" | "title" | "content" | "author"
    pub feed_id: Option<String>,
    pub folder_id: Option<String>,
    pub score_delta: i64,
    pub target_tag: Option<String>,
    pub target_board_id: Option<String>,
    pub target_webhook_id: Option<String>,
    pub is_enabled: bool,
    pub match_count: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub name: String,
    pub kind: String, // "generic" | "slack" | "teams"
    pub url: String,
    pub secret: Option<String>,
    pub is_enabled: bool,
    pub last_status: Option<String>,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub user_id: String,
    pub user_email: String,
    pub name: String,
    pub key_prefix: String,
    pub scopes: String,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub details: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    pub llm_provider: String, // "none" | "gemini" | "openai" | "ollama"
    pub llm_model: String,
    pub llm_api_key_set: bool,
    pub llm_base_url: String,
    pub app_base_url: String,
    pub oauth_google_enabled: bool,
    pub oauth_google_client_id: String,
    pub oauth_google_secret_set: bool,
    pub oauth_entra_enabled: bool,
    pub oauth_entra_tenant_id: String,
    pub oauth_entra_client_id: String,
    pub oauth_entra_secret_set: bool,
    pub oauth_facebook_enabled: bool,
    pub oauth_facebook_client_id: String,
    pub oauth_facebook_secret_set: bool,
    pub sso_enabled: bool,
    pub sso_provider_name: String,
    pub sso_issuer_url: String,
    pub sso_client_id: String,
    pub dedup_threshold: i64,
    pub default_poll_mins: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryStats {
    pub total_feeds: i64,
    pub active_feeds: i64,
    pub failing_feeds: i64,
    pub total_articles: i64,
    pub unread_articles: i64,
    pub total_users: i64,
    pub total_boards: i64,
    pub total_rules: i64,
    pub total_webhooks: i64,
    pub db_size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarCounts {
    pub all_unread: i64,
    pub today_unread: i64,
    pub leo_priority: i64,
    pub read_later: i64,
    pub recently_read: i64,
}
