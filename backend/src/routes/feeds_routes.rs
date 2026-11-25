use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    audit::log_audit,
    auth::AuthUser,
    models::{Feed, Folder, SidebarCounts},
    rss_engine,
    AppState,
};

#[derive(Deserialize)]
pub struct CreateFolderReq {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateFeedReq {
    pub feed_url: String,
    pub title: Option<String>,
    pub folder_id: Option<String>,
    pub refresh_interval_mins: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateFeedReq {
    pub title: Option<String>,
    pub folder_id: Option<Option<String>>,
    pub refresh_interval_mins: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct DiscoverReq {
    pub url: String,
}

#[derive(Deserialize)]
pub struct OpmlImportReq {
    pub xml: String,
}

pub async fn get_sidebar(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let conn = match state.db.get() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "DB error"}))).into_response(),
    };

    let all_unread: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles a
             LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
             WHERE a.is_muted = 0 AND COALESCE(s.is_read, 0) = 0",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let today_unread: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles a
             LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
             WHERE a.is_muted = 0 AND COALESCE(s.is_read, 0) = 0
               AND a.published_at >= datetime('now', '-24 hours')",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let leo_priority: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles a
             LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
             WHERE a.is_muted = 0 AND a.priority_score >= 65 AND COALESCE(s.is_read, 0) = 0",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let read_later: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_article_state WHERE user_id = ?1 AND is_read_later = 1",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let recently_read: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_article_state WHERE user_id = ?1 AND is_read = 1",
            params![user.id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut folder_stmt = conn
        .prepare(
            "SELECT f.id, f.name, f.color, f.icon, f.sort_order,
                    (SELECT COUNT(*) FROM feeds fd WHERE fd.folder_id = f.id) as feed_count,
                    (SELECT COUNT(*) FROM articles a
                     JOIN feeds fd ON fd.id = a.feed_id
                     LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
                     WHERE fd.folder_id = f.id AND a.is_muted = 0 AND COALESCE(s.is_read, 0) = 0) as unread_count
             FROM folders f
             ORDER BY f.sort_order ASC, f.name ASC",
        )
        .unwrap();

    let folders: Vec<Folder> = folder_stmt
        .query_map(params![user.id], |r| {
            Ok(Folder {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                icon: r.get(3)?,
                sort_order: r.get(4)?,
                feed_count: r.get(5)?,
                unread_count: r.get(6)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let mut feed_stmt = conn
        .prepare(
            "SELECT fd.id, fd.folder_id, fd.title, fd.feed_url, fd.site_url, fd.description,
                    fd.icon_url, fd.refresh_interval_mins, fd.last_fetched_at, fd.last_error,
                    fd.error_count, fd.is_active, fd.created_at,
                    (SELECT COUNT(*) FROM articles a
                     LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
                     WHERE a.feed_id = fd.id AND a.is_muted = 0 AND COALESCE(s.is_read, 0) = 0) as unread_count,
                    (SELECT COUNT(*) FROM articles a WHERE a.feed_id = fd.id) as total_articles
             FROM feeds fd
             ORDER BY fd.title ASC",
        )
        .unwrap();

    let feeds: Vec<Feed> = feed_stmt
        .query_map(params![user.id], |r| {
            Ok(Feed {
                id: r.get(0)?,
                folder_id: r.get(1)?,
                title: r.get(2)?,
                feed_url: r.get(3)?,
                site_url: r.get(4)?,
                description: r.get(5)?,
                icon_url: r.get(6)?,
                refresh_interval_mins: r.get(7)?,
                last_fetched_at: r.get(8)?,
                last_error: r.get(9)?,
                error_count: r.get(10)?,
                is_active: r.get::<_, i64>(11)? != 0,
                created_at: r.get(12)?,
                unread_count: r.get(13)?,
                total_articles: r.get(14)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    Json(json!({
        "counts": SidebarCounts {
            all_unread,
            today_unread,
            leo_priority,
            read_later,
            recently_read,
        },
        "folders": folders,
        "feeds": feeds,
    }))
    .into_response()
}

pub async fn create_folder(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateFolderReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let name = req.name.trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Folder name required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let id = Uuid::new_v4().to_string();
    let color = req.color.unwrap_or_else(|| "#10b981".to_string());
    let icon = req.icon.unwrap_or_else(|| "folder".to_string());
    let now = Utc::now().to_rfc3339();

    if let Err(_) = conn.execute(
        "INSERT INTO folders (id, name, color, icon, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, 20, ?5)",
        params![id, name, color, icon, now],
    ) {
        return (StatusCode::CONFLICT, Json(json!({"error": "Folder already exists"}))).into_response();
    }

    log_audit(&conn, Some(&user.id), Some(&user.email), "folder.create", "folder", Some(&id), name);
    (StatusCode::CREATED, Json(json!({"id": id, "name": name, "color": color, "icon": icon}))).into_response()
}

pub async fn delete_folder(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_manage_feeds() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin or editor role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let _ = conn.execute("DELETE FROM folders WHERE id = ?1", params![id]);
    log_audit(&conn, Some(&user.id), Some(&user.email), "folder.delete", "folder", Some(&id), "Deleted folder");
    StatusCode::NO_CONTENT.into_response()
}

pub async fn discover_feeds(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(req): Json<DiscoverReq>,
) -> impl IntoResponse {
    match rss_engine::discover_feeds_from_url(&state.http, &req.url).await {
        Ok(list) => Json(json!({"feeds": list})).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))).into_response(),
    }
}

pub async fn create_feed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateFeedReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let feed_url = req.feed_url.trim().to_string();
    if feed_url.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "feed_url is required"}))).into_response();
    }

    let id = Uuid::new_v4().to_string();
    let title = req
        .title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| feed_url.clone());
    let interval = req.refresh_interval_mins.unwrap_or(15).clamp(5, 1440);
    let now = Utc::now().to_rfc3339();

    {
        let conn = state.db.get().unwrap();
        if let Err(_) = conn.execute(
            "INSERT INTO feeds (id, folder_id, title, feed_url, refresh_interval_mins, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            params![id, req.folder_id, title, feed_url, interval, now],
        ) {
            return (StatusCode::CONFLICT, Json(json!({"error": "Feed URL already subscribed"}))).into_response();
        }
        log_audit(&conn, Some(&user.id), Some(&user.email), "feed.subscribe", "feed", Some(&id), &feed_url);
    }

    // Immediately fetch articles in the background so the user sees content right away
    let http = state.http.clone();
    let db = state.db.clone();
    let fid = id.clone();
    let new_articles = rss_engine::refresh_single_feed(&http, &db, &fid).await.unwrap_or(0);

    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "title": title,
            "feed_url": feed_url,
            "new_articles": new_articles
        })),
    )
        .into_response()
}

pub async fn update_feed(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateFeedReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    if let Some(ref t) = req.title {
        let _ = conn.execute("UPDATE feeds SET title = ?1 WHERE id = ?2", params![t.trim(), id]);
    }
    if let Some(ref fid) = req.folder_id {
        let _ = conn.execute("UPDATE feeds SET folder_id = ?1 WHERE id = ?2", params![fid, id]);
    }
    if let Some(interval) = req.refresh_interval_mins {
        let _ = conn.execute(
            "UPDATE feeds SET refresh_interval_mins = ?1 WHERE id = ?2",
            params![interval.clamp(5, 1440), id],
        );
    }
    if let Some(active) = req.is_active {
        let _ = conn.execute(
            "UPDATE feeds SET is_active = ?1 WHERE id = ?2",
            params![if active { 1 } else { 0 }, id],
        );
    }
    log_audit(&conn, Some(&user.id), Some(&user.email), "feed.update", "feed", Some(&id), "Updated feed settings");
    Json(json!({"ok": true})).into_response()
}

pub async fn delete_feed(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.can_manage_feeds() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Admin or editor role required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let _ = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id]);
    log_audit(&conn, Some(&user.id), Some(&user.email), "feed.unsubscribe", "feed", Some(&id), "Removed feed");
    StatusCode::NO_CONTENT.into_response()
}

pub async fn refresh_feed(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match rss_engine::refresh_single_feed(&state.http, &state.db, &id).await {
        Ok(n) => Json(json!({"new_articles": n})).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({"error": e}))).into_response(),
    }
}

pub async fn refresh_all(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let summary = rss_engine::refresh_all_feeds(
        state.http.clone(),
        state.db.clone(),
        state.config.max_concurrent_fetches,
    )
    .await;

    if let Ok(conn) = state.db.get() {
        log_audit(
            &conn,
            Some(&user.id),
            Some(&user.email),
            "feeds.refresh_all",
            "system",
            None,
            &format!("Checked {} feeds, +{} articles", summary.feeds_checked, summary.new_articles),
        );
    }
    Json(summary).into_response()
}

pub async fn import_opml(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<OpmlImportReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))).into_response();
    }
    match rss_engine::import_opml(&state.db, &req.xml) {
        Ok((folders, feeds)) => {
            if let Ok(conn) = state.db.get() {
                log_audit(
                    &conn,
                    Some(&user.id),
                    Some(&user.email),
                    "opml.import",
                    "system",
                    None,
                    &format!("Imported {} folders and {} feeds", folders, feeds),
                );
            }
            // Trigger background refresh for newly imported feeds
            let http = state.http.clone();
            let db = state.db.clone();
            let conc = state.config.max_concurrent_fetches;
            tokio::spawn(async move {
                let _ = rss_engine::refresh_all_feeds(http, db, conc).await;
            });
            Json(json!({
                "folders_created": folders,
                "feeds_imported": feeds
            }))
            .into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))).into_response(),
    }
}

pub async fn export_opml(State(state): State<AppState>, _user: AuthUser) -> impl IntoResponse {
    match rss_engine::export_opml(&state.db) {
        Ok(xml) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"pulsefeed-subscriptions.opml\""),
            ],
            xml,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    }
}
