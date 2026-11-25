use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rand::Rng;
use rusqlite::params;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    audit::log_audit,
    auth::{hash_api_key, hash_password, AuthUser},
    leo,
    models::{
        ApiKeyRecord, AuditLogEntry, LeoRule, SystemSettings, TeamBoard, TelemetryStats, User,
        WebhookEndpoint,
    },
    webhooks, AppState,
};

// ===================== TEAM BOARDS =====================

#[derive(Deserialize)]
pub struct CreateBoardReq {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<String>, // private | team | public
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct BoardArticleReq {
    pub article_id: String,
}

pub async fn list_boards(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.name, b.description, b.visibility, b.color, b.created_by, u.name,
                    b.public_token, b.created_at,
                    (SELECT COUNT(*) FROM board_articles ba WHERE ba.board_id = b.id) as art_cnt
             FROM boards b
             JOIN users u ON u.id = b.created_by
             WHERE b.visibility IN ('team', 'public') OR b.created_by = ?1
             ORDER BY b.created_at ASC",
        )
        .unwrap();

    let boards: Vec<TeamBoard> = stmt
        .query_map(params![user.id], |r| {
            Ok(TeamBoard {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                visibility: r.get(3)?,
                color: r.get(4)?,
                created_by: r.get(5)?,
                creator_name: r.get(6)?,
                public_token: r.get(7)?,
                created_at: r.get(8)?,
                article_count: r.get(9)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    Json(json!({ "boards": boards })).into_response()
}

pub async fn create_board(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateBoardReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let name = req.name.trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Board name required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let vis = req.visibility.unwrap_or_else(|| "team".to_string());
    let color = req.color.unwrap_or_else(|| "#3b82f6".to_string());
    let desc = req.description.unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let _ = conn.execute(
        "INSERT INTO boards (id, name, description, visibility, color, created_by, public_token, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, name, desc, vis, color, user.id, token, now],
    );
    log_audit(&conn, Some(&user.id), Some(&user.email), "board.create", "board", Some(&id), name);

    (
        StatusCode::CREATED,
        Json(TeamBoard {
            id,
            name: name.to_string(),
            description: desc,
            visibility: vis,
            color,
            created_by: user.id,
            creator_name: user.name,
            article_count: 0,
            public_token: token,
            created_at: now,
        }),
    )
        .into_response()
}

pub async fn delete_board(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    if user.is_admin() {
        let _ = conn.execute("DELETE FROM boards WHERE id = ?1", params![id]);
    } else {
        let _ = conn.execute(
            "DELETE FROM boards WHERE id = ?1 AND created_by = ?2",
            params![id, user.id],
        );
    }
    log_audit(&conn, Some(&user.id), Some(&user.email), "board.delete", "board", Some(&id), "Deleted board");
    StatusCode::NO_CONTENT.into_response()
}

pub async fn toggle_board_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(board_id): Path<String>,
    Json(req): Json<BoardArticleReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM board_articles WHERE board_id = ?1 AND article_id = ?2",
            params![board_id, req.article_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if exists {
        let _ = conn.execute(
            "DELETE FROM board_articles WHERE board_id = ?1 AND article_id = ?2",
            params![board_id, req.article_id],
        );
        Json(json!({"saved": false})).into_response()
    } else {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO board_articles (board_id, article_id, added_by, added_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![board_id, req.article_id, user.id, Utc::now().to_rfc3339()],
        );
        Json(json!({"saved": true})).into_response()
    }
}

/// Public Enterprise RSS Feed output for any Board with visibility = 'public' (just like Feedly Enterprise Team Board RSS!)
pub async fn board_public_rss(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let board: Option<(String, String, String)> = conn
        .query_row(
            "SELECT id, name, description FROM boards WHERE public_token = ?1 AND visibility = 'public'",
            params![token],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();

    let Some((bid, bname, bdesc)) = board else {
        return (StatusCode::NOT_FOUND, "Public board RSS not found").into_response();
    };

    let mut stmt = conn
        .prepare(
            "SELECT a.title, a.url, a.summary, a.published_at
             FROM board_articles ba
             JOIN articles a ON a.id = ba.article_id
             WHERE ba.board_id = ?1
             ORDER BY ba.added_at DESC
             LIMIT 50",
        )
        .unwrap();

    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel><title>{}</title><description>{}</description>",
        escape_xml(&bname),
        escape_xml(&bdesc)
    );

    if let Ok(rows) = stmt.query_map(params![bid], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    }) {
        for (title, url, summary, pub_at) in rows.flatten() {
            xml.push_str(&format!(
                "<item><title>{}</title><link>{}</link><description>{}</description><pubDate>{}</pubDate></item>",
                escape_xml(&title),
                escape_xml(&url),
                escape_xml(&summary),
                escape_xml(&pub_at)
            ));
        }
    }
    xml.push_str("</channel></rss>");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ===================== LEO INTELLIGENCE RULES =====================

#[derive(Deserialize)]
pub struct CreateLeoRuleReq {
    pub name: String,
    pub rule_type: String,
    pub keywords: String,
    pub match_field: Option<String>,
    pub feed_id: Option<String>,
    pub folder_id: Option<String>,
    pub score_delta: Option<i64>,
    pub target_tag: Option<String>,
    pub target_board_id: Option<String>,
    pub target_webhook_id: Option<String>,
}

pub async fn list_leo_rules(State(state): State<AppState>, _user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, rule_type, keywords, match_field, feed_id, folder_id,
                    score_delta, target_tag, target_board_id, target_webhook_id,
                    is_enabled, match_count, created_at
             FROM leo_rules ORDER BY created_at DESC",
        )
        .unwrap();

    let rules: Vec<LeoRule> = stmt
        .query_map([], |r| {
            Ok(LeoRule {
                id: r.get(0)?,
                name: r.get(1)?,
                rule_type: r.get(2)?,
                keywords: r.get(3)?,
                match_field: r.get(4)?,
                feed_id: r.get(5)?,
                folder_id: r.get(6)?,
                score_delta: r.get(7)?,
                target_tag: r.get(8)?,
                target_board_id: r.get(9)?,
                target_webhook_id: r.get(10)?,
                is_enabled: r.get::<_, i64>(11)? != 0,
                match_count: r.get(12)?,
                created_at: r.get(13)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    Json(json!({ "rules": rules })).into_response()
}

pub async fn create_leo_rule(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateLeoRuleReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let match_field = req.match_field.unwrap_or_else(|| "any".to_string());
    let score_delta = req.score_delta.unwrap_or(if req.rule_type == "mute" { -50 } else { 25 });

    let _ = conn.execute(
        "INSERT INTO leo_rules (id, name, rule_type, keywords, match_field, feed_id, folder_id,
                                score_delta, target_tag, target_board_id, target_webhook_id, is_enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12)",
        params![
            id,
            req.name.trim(),
            req.rule_type,
            req.keywords.trim(),
            match_field,
            req.feed_id,
            req.folder_id,
            score_delta,
            req.target_tag,
            req.target_board_id,
            req.target_webhook_id,
            now
        ],
    );

    log_audit(&conn, Some(&user.id), Some(&user.email), "leo.rule.create", "leo_rule", Some(&id), &req.name);
    (StatusCode::CREATED, Json(json!({ "id": id }))).into_response()
}

pub async fn toggle_leo_rule(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let _ = conn.execute(
        "UPDATE leo_rules SET is_enabled = CASE WHEN is_enabled = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        params![id],
    );
    Json(json!({ "ok": true })).into_response()
}

pub async fn delete_leo_rule(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let _ = conn.execute("DELETE FROM leo_rules WHERE id = ?1", params![id]);
    log_audit(&conn, Some(&user.id), Some(&user.email), "leo.rule.delete", "leo_rule", Some(&id), "Deleted rule");
    StatusCode::NO_CONTENT.into_response()
}

/// Re-evaluate all Leo rules across recent articles so newly created rules take effect immediately!
pub async fn rerun_leo_rules(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let rules = leo::load_enabled_rules(&conn);
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.feed_id, fd.folder_id, a.title, a.content_text, a.author
             FROM articles a JOIN feeds fd ON fd.id = a.feed_id
             ORDER BY a.published_at DESC LIMIT 1000",
        )
        .unwrap();

    let rows: Vec<(String, String, Option<String>, String, String, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
        })
        .unwrap()
        .flatten()
        .collect();

    let mut updated = 0usize;
    let now = Utc::now().to_rfc3339();
    for (aid, fid, fol_id, title, text, author) in rows {
        let eval = leo::evaluate_article(&rules, &fid, fol_id.as_deref(), &title, &text, author.as_deref());
        let tags_json = serde_json::to_string(&eval.tags).unwrap_or_else(|_| "[]".to_string());
        let _ = conn.execute(
            "UPDATE articles SET priority_score = ?1, is_muted = ?2, tags_json = ?3 WHERE id = ?4",
            params![eval.priority_score, if eval.is_muted { 1 } else { 0 }, tags_json, aid],
        );
        for bid in eval.auto_board_ids {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO board_articles (board_id, article_id, added_by, added_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![bid, aid, user.id, now],
            );
        }
        updated += 1;
    }

    log_audit(
        &conn,
        Some(&user.id),
        Some(&user.email),
        "leo.rules.rerun",
        "system",
        None,
        &format!("Re-scored {} articles", updated),
    );
    Json(json!({ "rescored": updated })).into_response()
}

// ===================== WEBHOOKS =====================

#[derive(Deserialize)]
pub struct CreateWebhookReq {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub secret: Option<String>,
}

pub async fn list_webhooks(State(state): State<AppState>, _user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, kind, url, secret, is_enabled, last_status, last_triggered_at, created_at
             FROM webhooks ORDER BY created_at DESC",
        )
        .unwrap();

    let items: Vec<WebhookEndpoint> = stmt
        .query_map([], |r| {
            Ok(WebhookEndpoint {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                url: r.get(3)?,
                secret: r.get(4)?,
                is_enabled: r.get::<_, i64>(5)? != 0,
                last_status: r.get(6)?,
                last_triggered_at: r.get(7)?,
                created_at: r.get(8)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    Json(json!({ "webhooks": items })).into_response()
}

pub async fn create_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateWebhookReq>,
) -> impl IntoResponse {
    if !user.can_manage_feeds() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin or editor role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO webhooks (id, name, kind, url, secret, is_enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![id, req.name.trim(), req.kind, req.url.trim(), req.secret, now],
    );
    log_audit(&conn, Some(&user.id), Some(&user.email), "webhook.create", "webhook", Some(&id), &req.name);
    (StatusCode::CREATED, Json(json!({ "id": id }))).into_response()
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_manage_feeds() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin or editor role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let _ = conn.execute("DELETE FROM webhooks WHERE id = ?1", params![id]);
    StatusCode::NO_CONTENT.into_response()
}

pub async fn test_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_manage_feeds() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin or editor role required"}))).into_response();
    }
    webhooks::trigger_webhook(
        &state.http,
        &state.db,
        &id,
        "Test Alert",
        "PulseFeed Enterprise Webhook Connectivity Test",
        "https://pulsefeed.local",
        "PulseFeed System",
        95,
    )
    .await;
    Json(json!({ "ok": true })).into_response()
}

// ===================== ADMIN: USERS, API KEYS, AUDIT LOGS, SETTINGS, TELEMETRY =====================

#[derive(Deserialize)]
pub struct AdminCreateUserReq {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct AdminUpdateUserReq {
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list_users(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, email, name, role, is_active, created_at FROM users ORDER BY created_at ASC")
        .unwrap();
    let users: Vec<User> = stmt
        .query_map([], |r| {
            Ok(User {
                id: r.get(0)?,
                email: r.get(1)?,
                name: r.get(2)?,
                role: r.get(3)?,
                is_active: r.get::<_, i64>(4)? != 0,
                created_at: r.get(5)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();
    Json(json!({ "users": users })).into_response()
}

pub async fn admin_create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<AdminCreateUserReq>,
) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let id = Uuid::new_v4().to_string();
    let hash = match hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e}))).into_response(),
    };
    let role = match req.role.as_str() {
        "admin" | "editor" | "member" | "viewer" => req.role,
        _ => "member".to_string(),
    };
    let now = Utc::now().to_rfc3339();
    if conn
        .execute(
            "INSERT INTO users (id, email, name, password_hash, role, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            params![id, req.email.trim().to_lowercase(), req.name.trim(), hash, role, now],
        )
        .is_err()
    {
        return (StatusCode::CONFLICT, Json(json!({"error": "Email already exists"}))).into_response();
    }
    log_audit(
        &conn,
        Some(&user.id),
        Some(&user.email),
        "admin.user.create",
        "user",
        Some(&id),
        &format!("Created {} ({})", req.email, role),
    );
    (StatusCode::CREATED, Json(json!({ "id": id }))).into_response()
}

pub async fn admin_update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AdminUpdateUserReq>,
) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    if let Some(ref role) = req.role {
        if matches!(role.as_str(), "admin" | "editor" | "member" | "viewer") {
            let _ = conn.execute("UPDATE users SET role = ?1 WHERE id = ?2", params![role, id]);
        }
    }
    if let Some(active) = req.is_active {
        let _ = conn.execute(
            "UPDATE users SET is_active = ?1 WHERE id = ?2",
            params![if active { 1 } else { 0 }, id],
        );
    }
    log_audit(&conn, Some(&user.id), Some(&user.email), "admin.user.update", "user", Some(&id), "Updated role/status");
    Json(json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
pub struct CreateApiKeyReq {
    pub name: String,
    pub scopes: Option<String>,
}

pub async fn list_api_keys(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let sql = if user.is_admin() {
        "SELECT k.id, k.user_id, u.email, k.name, k.key_prefix, k.scopes, k.last_used_at, k.created_at
         FROM api_keys k JOIN users u ON u.id = k.user_id ORDER BY k.created_at DESC"
    } else {
        "SELECT k.id, k.user_id, u.email, k.name, k.key_prefix, k.scopes, k.last_used_at, k.created_at
         FROM api_keys k JOIN users u ON u.id = k.user_id WHERE k.user_id = ?1 ORDER BY k.created_at DESC"
    };

    let mut stmt = conn.prepare(sql).unwrap();
    let mapper = |r: &rusqlite::Row| {
        Ok(ApiKeyRecord {
            id: r.get(0)?,
            user_id: r.get(1)?,
            user_email: r.get(2)?,
            name: r.get(3)?,
            key_prefix: r.get(4)?,
            scopes: r.get(5)?,
            last_used_at: r.get(6)?,
            created_at: r.get(7)?,
        })
    };

    let keys: Vec<ApiKeyRecord> = if user.is_admin() {
        stmt.query_map([], mapper).unwrap().flatten().collect()
    } else {
        stmt.query_map(params![user.id], mapper).unwrap().flatten().collect()
    };

    Json(json!({ "api_keys": keys })).into_response()
}

pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateApiKeyReq>,
) -> impl IntoResponse {
    let raw_bytes: [u8; 24] = rand::thread_rng().gen();
    let raw_key = format!("pf_{}", hex::encode(raw_bytes));
    let prefix = raw_key[..11].to_string();
    let hash = hash_api_key(&raw_key);
    let id = Uuid::new_v4().to_string();
    let scopes = req.scopes.unwrap_or_else(|| "read,write".to_string());
    let now = Utc::now().to_rfc3339();

    let conn = state.db.get().unwrap();
    let _ = conn.execute(
        "INSERT INTO api_keys (id, user_id, name, key_prefix, key_hash, scopes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, user.id, req.name.trim(), prefix, hash, scopes, now],
    );
    log_audit(&conn, Some(&user.id), Some(&user.email), "apikey.create", "api_key", Some(&id), &req.name);

    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "name": req.name.trim(),
            "raw_key": raw_key,
            "key_prefix": prefix
        })),
    )
        .into_response()
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    if user.is_admin() {
        let _ = conn.execute("DELETE FROM api_keys WHERE id = ?1", params![id]);
    } else {
        let _ = conn.execute(
            "DELETE FROM api_keys WHERE id = ?1 AND user_id = ?2",
            params![id, user.id],
        );
    }
    log_audit(&conn, Some(&user.id), Some(&user.email), "apikey.revoke", "api_key", Some(&id), "Revoked API key");
    StatusCode::NO_CONTENT.into_response()
}

pub async fn list_audit_logs(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, user_email, action, target_type, target_id, details, created_at
             FROM audit_logs ORDER BY id DESC LIMIT 200",
        )
        .unwrap();
    let logs: Vec<AuditLogEntry> = stmt
        .query_map([], |r| {
            Ok(AuditLogEntry {
                id: r.get(0)?,
                user_id: r.get(1)?,
                user_email: r.get(2)?,
                action: r.get(3)?,
                target_type: r.get(4)?,
                target_id: r.get(5)?,
                details: r.get(6)?,
                created_at: r.get(7)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();
    Json(json!({ "logs": logs })).into_response()
}

pub async fn get_settings(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let m = leo::load_settings_map(&conn);
    let get = |k: &str, d: &str| m.get(k).cloned().unwrap_or_else(|| d.to_string());

    Json(SystemSettings {
        llm_provider: get("llm_provider", "none"),
        llm_model: get("llm_model", "gemini-2.5-flash"),
        llm_api_key_set: !get("llm_api_key", "").is_empty(),
        llm_base_url: get("llm_base_url", ""),
        sso_enabled: get("sso_enabled", "false") == "true",
        sso_provider_name: get("sso_provider_name", "Microsoft Entra ID / OIDC"),
        sso_issuer_url: get("sso_issuer_url", ""),
        sso_client_id: get("sso_client_id", ""),
        dedup_threshold: get("dedup_threshold", "6").parse().unwrap_or(6),
        default_poll_mins: get("default_poll_mins", "15").parse().unwrap_or(15),
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct UpdateSettingsReq {
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,
    pub sso_enabled: Option<bool>,
    pub sso_provider_name: Option<String>,
    pub sso_issuer_url: Option<String>,
    pub sso_client_id: Option<String>,
    pub dedup_threshold: Option<i64>,
    pub default_poll_mins: Option<i64>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateSettingsReq>,
) -> impl IntoResponse {
    if !user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let set = |k: &str, v: &str| {
        let _ = conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        );
    };

    if let Some(ref v) = req.llm_provider { set("llm_provider", v); }
    if let Some(ref v) = req.llm_model { set("llm_model", v); }
    if let Some(ref v) = req.llm_api_key {
        if !v.is_empty() {
            set("llm_api_key", v);
        }
    }
    if let Some(ref v) = req.llm_base_url { set("llm_base_url", v); }
    if let Some(v) = req.sso_enabled { set("sso_enabled", if v { "true" } else { "false" }); }
    if let Some(ref v) = req.sso_provider_name { set("sso_provider_name", v); }
    if let Some(ref v) = req.sso_issuer_url { set("sso_issuer_url", v); }
    if let Some(ref v) = req.sso_client_id { set("sso_client_id", v); }
    if let Some(v) = req.dedup_threshold { set("dedup_threshold", &v.clamp(1, 16).to_string()); }
    if let Some(v) = req.default_poll_mins { set("default_poll_mins", &v.clamp(5, 1440).to_string()); }

    log_audit(&conn, Some(&user.id), Some(&user.email), "admin.settings.update", "settings", None, "Updated LLM/SSO/Engine settings");
    Json(json!({ "ok": true })).into_response()
}

pub async fn get_telemetry(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let q = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0) };
    let unread: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles a
             LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
             WHERE a.is_muted = 0 AND COALESCE(s.is_read, 0) = 0",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let db_size = std::fs::metadata(&state.config.db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Json(TelemetryStats {
        total_feeds: q("SELECT COUNT(*) FROM feeds"),
        active_feeds: q("SELECT COUNT(*) FROM feeds WHERE is_active = 1"),
        failing_feeds: q("SELECT COUNT(*) FROM feeds WHERE error_count > 0"),
        total_articles: q("SELECT COUNT(*) FROM articles"),
        unread_articles: unread,
        total_users: q("SELECT COUNT(*) FROM users"),
        total_boards: q("SELECT COUNT(*) FROM boards"),
        total_rules: q("SELECT COUNT(*) FROM leo_rules"),
        total_webhooks: q("SELECT COUNT(*) FROM webhooks"),
        db_size_bytes: db_size,
    })
    .into_response()
}
