use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rusqlite::{params, types::Value as SqlValue};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    leo,
    models::{ArticleDetail, ArticleListItem, ArticleNote, DuplicateBrief, Highlight},
    AppState,
};

#[derive(Deserialize)]
pub struct ArticleQuery {
    pub view: Option<String>, // all | today | leo_priority | read_later | recently_read | muted
    pub folder_id: Option<String>,
    pub feed_id: Option<String>,
    pub board_id: Option<String>,
    pub tag: Option<String>,
    pub q: Option<String>,
    pub unread_only: Option<bool>,
    pub dedup: Option<bool>,
    pub sort: Option<String>, // newest | oldest | priority
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateArticleStateReq {
    pub is_read: Option<bool>,
    pub is_read_later: Option<bool>,
}

#[derive(Deserialize)]
pub struct MarkAllReadReq {
    pub folder_id: Option<String>,
    pub feed_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateHighlightReq {
    pub selected_text: String,
    pub color: Option<String>,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateNoteReq {
    pub content: String,
}

fn sanitize_fts_query(q: &str) -> String {
    q.split_whitespace()
        .map(|w| {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
            if clean.is_empty() {
                String::new()
            } else {
                format!("\"{}\"*", clean)
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn list_articles(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<ArticleQuery>,
) -> impl IntoResponse {
    let conn = match state.db.get() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "DB error"}))).into_response(),
    };

    let view = q.view.as_deref().unwrap_or("all");
    let limit = q.limit.unwrap_or(40).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);
    let dedup = q.dedup.unwrap_or(true);

    let mut sql = String::from(
        "SELECT a.id, a.feed_id, fd.title, fd.icon_url, fd.folder_id,
                a.title, a.url, a.author, a.summary, a.image_url, a.published_at,
                a.reading_time_mins, a.priority_score, a.is_muted,
                COALESCE(s.is_read, 0), COALESCE(s.is_read_later, 0),
                a.tags_json, a.duplicate_cluster_id, a.ai_summary, a.ai_sentiment,
                (SELECT COUNT(*) FROM highlights h WHERE h.article_id = a.id) as hl_cnt,
                (SELECT COUNT(*) FROM article_notes n WHERE n.article_id = a.id) as note_cnt,
                CASE WHEN a.duplicate_cluster_id IS NULL THEN 1
                     ELSE (SELECT COUNT(*) FROM articles d WHERE d.duplicate_cluster_id = a.duplicate_cluster_id)
                END as dup_cnt
         FROM articles a
         JOIN feeds fd ON fd.id = a.feed_id
         LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1 ",
    );

    let mut sql_params: Vec<SqlValue> = vec![SqlValue::Text(user.id.clone())];
    let mut wheres: Vec<String> = Vec::new();

    if let Some(ref bid) = q.board_id {
        sql.push_str("JOIN board_articles ba ON ba.article_id = a.id ");
        sql_params.push(SqlValue::Text(bid.clone()));
        wheres.push(format!("ba.board_id = ?{}", sql_params.len()));
    }

    if let Some(ref search) = q.q {
        let fts = sanitize_fts_query(search);
        if !fts.is_empty() {
            sql.push_str("JOIN articles_fts fts ON fts.rowid = a.rowid ");
            sql_params.push(SqlValue::Text(fts));
            wheres.push(format!("articles_fts MATCH ?{}", sql_params.len()));
        }
    }

    if view == "muted" {
        wheres.push("a.is_muted = 1".to_string());
    } else if q.board_id.is_none() {
        wheres.push("a.is_muted = 0".to_string());
    }

    match view {
        "today" => wheres.push("a.published_at >= datetime('now', '-24 hours')".to_string()),
        "leo_priority" => wheres.push("a.priority_score >= 65".to_string()),
        "read_later" => wheres.push("COALESCE(s.is_read_later, 0) = 1".to_string()),
        "recently_read" => wheres.push("COALESCE(s.is_read, 0) = 1".to_string()),
        _ => {}
    }

    if let Some(ref fid) = q.folder_id {
        sql_params.push(SqlValue::Text(fid.clone()));
        wheres.push(format!("fd.folder_id = ?{}", sql_params.len()));
    }
    if let Some(ref feed_id) = q.feed_id {
        sql_params.push(SqlValue::Text(feed_id.clone()));
        wheres.push(format!("a.feed_id = ?{}", sql_params.len()));
    }
    if let Some(ref tag) = q.tag {
        sql_params.push(SqlValue::Text(format!("%\"{}\"%", tag.to_lowercase())));
        wheres.push(format!("a.tags_json LIKE ?{}", sql_params.len()));
    }
    if q.unread_only.unwrap_or(false) && view != "recently_read" {
        wheres.push("COALESCE(s.is_read, 0) = 0".to_string());
    }

    if !wheres.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&wheres.join(" AND "));
        sql.push(' ');
    }

    if dedup {
        sql.push_str("GROUP BY COALESCE(a.duplicate_cluster_id, a.id) ");
    }

    let order_clause = match q.sort.as_deref() {
        Some("oldest") => "ORDER BY a.published_at ASC",
        Some("priority") => "ORDER BY a.priority_score DESC, a.published_at DESC",
        _ => {
            if view == "leo_priority" {
                "ORDER BY a.priority_score DESC, a.published_at DESC"
            } else if view == "recently_read" {
                "ORDER BY s.read_at DESC"
            } else {
                "ORDER BY a.published_at DESC"
            }
        }
    };
    sql.push_str(order_clause);

    sql_params.push(SqlValue::Integer(limit));
    let lim_idx = sql_params.len();
    sql_params.push(SqlValue::Integer(offset));
    let off_idx = sql_params.len();
    sql.push_str(&format!(" LIMIT ?{} OFFSET ?{}", lim_idx, off_idx));

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let items: Vec<ArticleListItem> = stmt
        .query_map(rusqlite::params_from_iter(sql_params), |r| {
            let article_id: String = r.get(0)?;
            let tags_raw: String = r.get(16)?;
            let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
            Ok(ArticleListItem {
                id: article_id,
                feed_id: r.get(1)?,
                feed_title: r.get(2)?,
                feed_icon: r.get(3)?,
                folder_id: r.get(4)?,
                title: r.get(5)?,
                url: r.get(6)?,
                author: r.get(7)?,
                summary: r.get(8)?,
                image_url: r.get(9)?,
                published_at: r.get(10)?,
                reading_time_mins: r.get(11)?,
                priority_score: r.get(12)?,
                is_muted: r.get::<_, i64>(13)? != 0,
                is_read: r.get::<_, i64>(14)? != 0,
                is_read_later: r.get::<_, i64>(15)? != 0,
                tags,
                duplicate_cluster_id: r.get(17)?,
                ai_summary: r.get(18)?,
                ai_sentiment: r.get(19)?,
                highlight_count: r.get(20)?,
                note_count: r.get(21)?,
                duplicate_count: r.get(22)?,
                board_ids: vec![],
            })
        })
        .unwrap()
        .flatten()
        .collect();

    // Populate board_ids for each article
    let mut enriched = Vec::with_capacity(items.len());
    for mut item in items {
        if let Ok(mut bstmt) = conn.prepare("SELECT board_id FROM board_articles WHERE article_id = ?1") {
            item.board_ids = bstmt
                .query_map(params![item.id], |r| r.get(0))
                .map(|it| it.flatten().collect())
                .unwrap_or_default();
        }
        enriched.push(item);
    }

    Json(json!({
        "articles": enriched,
        "count": enriched.len(),
        "offset": offset,
        "limit": limit
    }))
    .into_response()
}

pub async fn get_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();

    let row = conn.query_row(
        "SELECT a.id, a.feed_id, fd.title, fd.icon_url, fd.folder_id,
                a.title, a.url, a.author, a.summary, a.image_url, a.published_at,
                a.reading_time_mins, a.priority_score, a.is_muted,
                COALESCE(s.is_read, 0), COALESCE(s.is_read_later, 0),
                a.tags_json, a.duplicate_cluster_id, a.ai_summary, a.ai_sentiment,
                a.content_html, a.content_text
         FROM articles a
         JOIN feeds fd ON fd.id = a.feed_id
         LEFT JOIN user_article_state s ON s.article_id = a.id AND s.user_id = ?1
         WHERE a.id = ?2",
        params![user.id, id],
        |r| {
            let tags_raw: String = r.get(16)?;
            let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
            Ok((
                ArticleListItem {
                    id: r.get(0)?,
                    feed_id: r.get(1)?,
                    feed_title: r.get(2)?,
                    feed_icon: r.get(3)?,
                    folder_id: r.get(4)?,
                    title: r.get(5)?,
                    url: r.get(6)?,
                    author: r.get(7)?,
                    summary: r.get(8)?,
                    image_url: r.get(9)?,
                    published_at: r.get(10)?,
                    reading_time_mins: r.get(11)?,
                    priority_score: r.get(12)?,
                    is_muted: r.get::<_, i64>(13)? != 0,
                    is_read: r.get::<_, i64>(14)? != 0,
                    is_read_later: r.get::<_, i64>(15)? != 0,
                    tags,
                    duplicate_cluster_id: r.get(17)?,
                    ai_summary: r.get(18)?,
                    ai_sentiment: r.get(19)?,
                    highlight_count: 0,
                    note_count: 0,
                    duplicate_count: 1,
                    board_ids: vec![],
                },
                r.get::<_, String>(20)?,
                r.get::<_, String>(21)?,
            ))
        },
    );

    let Ok((mut item, content_html, content_text)) = row else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "Article not found"}))).into_response();
    };

    if let Ok(mut bstmt) = conn.prepare("SELECT board_id FROM board_articles WHERE article_id = ?1") {
        item.board_ids = bstmt
            .query_map(params![item.id], |r| r.get(0))
            .map(|it| it.flatten().collect())
            .unwrap_or_default();
    }

    let highlights: Vec<Highlight> = conn
        .prepare(
            "SELECT h.id, h.article_id, h.user_id, u.name, h.selected_text, h.color, h.note, h.created_at
             FROM highlights h JOIN users u ON u.id = h.user_id
             WHERE h.article_id = ?1 ORDER BY h.created_at ASC",
        )
        .unwrap()
        .query_map(params![item.id], |r| {
            Ok(Highlight {
                id: r.get(0)?,
                article_id: r.get(1)?,
                user_id: r.get(2)?,
                user_name: r.get(3)?,
                selected_text: r.get(4)?,
                color: r.get(5)?,
                note: r.get(6)?,
                created_at: r.get(7)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let notes: Vec<ArticleNote> = conn
        .prepare(
            "SELECT n.id, n.article_id, n.user_id, u.name, n.content, n.created_at
             FROM article_notes n JOIN users u ON u.id = n.user_id
             WHERE n.article_id = ?1 ORDER BY n.created_at ASC",
        )
        .unwrap()
        .query_map(params![item.id], |r| {
            Ok(ArticleNote {
                id: r.get(0)?,
                article_id: r.get(1)?,
                user_id: r.get(2)?,
                user_name: r.get(3)?,
                content: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .unwrap()
        .flatten()
        .collect();

    let duplicates: Vec<DuplicateBrief> = if let Some(ref cid) = item.duplicate_cluster_id {
        conn.prepare(
            "SELECT a.id, a.title, a.url, fd.title, a.published_at
             FROM articles a JOIN feeds fd ON fd.id = a.feed_id
             WHERE a.duplicate_cluster_id = ?1 AND a.id != ?2
             ORDER BY a.published_at DESC",
        )
        .unwrap()
        .query_map(params![cid, item.id], |r| {
            Ok(DuplicateBrief {
                id: r.get(0)?,
                title: r.get(1)?,
                url: r.get(2)?,
                feed_title: r.get(3)?,
                published_at: r.get(4)?,
            })
        })
        .unwrap()
        .flatten()
        .collect()
    } else {
        vec![]
    };

    item.highlight_count = highlights.len() as i64;
    item.note_count = notes.len() as i64;
    item.duplicate_count = (duplicates.len() + 1) as i64;

    Json(ArticleDetail {
        item,
        content_html,
        content_text,
        highlights,
        notes,
        duplicates,
    })
    .into_response()
}

pub async fn update_article_state(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateArticleStateReq>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let now = Utc::now().to_rfc3339();

    let (cur_read, cur_later): (i64, i64) = conn
        .query_row(
            "SELECT is_read, is_read_later FROM user_article_state WHERE user_id = ?1 AND article_id = ?2",
            params![user.id, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));

    let next_read = req.is_read.map(|b| if b { 1 } else { 0 }).unwrap_or(cur_read);
    let next_later = req.is_read_later.map(|b| if b { 1 } else { 0 }).unwrap_or(cur_later);
    let read_at = if next_read == 1 { Some(now) } else { None };

    let _ = conn.execute(
        "INSERT INTO user_article_state (user_id, article_id, is_read, is_read_later, read_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(user_id, article_id) DO UPDATE SET
           is_read = excluded.is_read,
           is_read_later = excluded.is_read_later,
           read_at = COALESCE(excluded.read_at, user_article_state.read_at)",
        params![user.id, id, next_read, next_later, read_at],
    );

    Json(json!({
        "is_read": next_read != 0,
        "is_read_later": next_later != 0
    }))
    .into_response()
}

pub async fn mark_all_read(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<MarkAllReadReq>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    let now = Utc::now().to_rfc3339();

    let affected = if let Some(ref feed_id) = req.feed_id {
        conn.execute(
            "INSERT INTO user_article_state (user_id, article_id, is_read, read_at)
             SELECT ?1, id, 1, ?2 FROM articles WHERE feed_id = ?3
             ON CONFLICT(user_id, article_id) DO UPDATE SET is_read = 1, read_at = excluded.read_at",
            params![user.id, now, feed_id],
        )
        .unwrap_or(0)
    } else if let Some(ref folder_id) = req.folder_id {
        conn.execute(
            "INSERT INTO user_article_state (user_id, article_id, is_read, read_at)
             SELECT ?1, a.id, 1, ?2 FROM articles a JOIN feeds f ON f.id = a.feed_id WHERE f.folder_id = ?3
             ON CONFLICT(user_id, article_id) DO UPDATE SET is_read = 1, read_at = excluded.read_at",
            params![user.id, now, folder_id],
        )
        .unwrap_or(0)
    } else {
        conn.execute(
            "INSERT INTO user_article_state (user_id, article_id, is_read, read_at)
             SELECT ?1, id, 1, ?2 FROM articles
             ON CONFLICT(user_id, article_id) DO UPDATE SET is_read = 1, read_at = excluded.read_at",
            params![user.id, now],
        )
        .unwrap_or(0)
    };

    Json(json!({"marked_read": affected})).into_response()
}

pub async fn generate_ai_summary(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (title, content_text, settings) = {
        let conn = state.db.get().unwrap();
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT title, content_text FROM articles WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((t, c)) = row else {
            return (StatusCode::NOT_FOUND, Json(json!({"error": "Article not found"}))).into_response();
        };
        let settings = leo::load_settings_map(&conn);
        (t, c, settings)
    };

    let (summary, sentiment) = leo::generate_ai_summary(&state.http, &settings, &title, &content_text).await;

    if let Ok(conn) = state.db.get() {
        let _ = conn.execute(
            "UPDATE articles SET ai_summary = ?1, ai_sentiment = ?2 WHERE id = ?3",
            params![summary, sentiment, id],
        );
    }

    Json(json!({
        "ai_summary": summary,
        "ai_sentiment": sentiment
    }))
    .into_response()
}

pub async fn add_highlight(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateHighlightReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Read-only role"}))).into_response();
    }
    let text = req.selected_text.trim();
    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Highlight text required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let hid = Uuid::new_v4().to_string();
    let color = req.color.unwrap_or_else(|| "yellow".to_string());
    let now = Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO highlights (id, article_id, user_id, selected_text, color, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![hid, id, user.id, text, color, req.note, now],
    );
    (
        StatusCode::CREATED,
        Json(Highlight {
            id: hid,
            article_id: id,
            user_id: user.id,
            user_name: user.name,
            selected_text: text.to_string(),
            color,
            note: req.note,
            created_at: now,
        }),
    )
        .into_response()
}

pub async fn delete_highlight(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_article_id, highlight_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let conn = state.db.get().unwrap();
    if user.is_admin() {
        let _ = conn.execute("DELETE FROM highlights WHERE id = ?1", params![highlight_id]);
    } else {
        let _ = conn.execute(
            "DELETE FROM highlights WHERE id = ?1 AND user_id = ?2",
            params![highlight_id, user.id],
        );
    }
    StatusCode::NO_CONTENT.into_response()
}

pub async fn add_note(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateNoteReq>,
) -> impl IntoResponse {
    if !user.can_edit() {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Read-only role"}))).into_response();
    }
    let content = req.content.trim();
    if content.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Note content required"}))).into_response();
    }
    let conn = state.db.get().unwrap();
    let nid = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO article_notes (id, article_id, user_id, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![nid, id, user.id, content, now],
    );
    (
        StatusCode::CREATED,
        Json(ArticleNote {
            id: nid,
            article_id: id,
            user_id: user.id,
            user_name: user.name,
            content: content.to_string(),
            created_at: now,
        }),
    )
        .into_response()
}
