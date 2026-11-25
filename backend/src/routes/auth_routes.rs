use axum::{
    extract::State,
    http::StatusCode,
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
    auth::{create_jwt, hash_password, verify_password, AuthUser},
    leo::load_settings_map,
    models::User,
    AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let conn = match state.db.get() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "DB error"}))).into_response(),
    };

    let email = req.email.trim().to_lowercase();
    let row: Option<(String, String, String, String, String, i64, String)> = conn
        .query_row(
            "SELECT id, email, name, password_hash, role, is_active, created_at FROM users WHERE lower(email) = ?1",
            params![email],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .ok();

    let Some((id, email, name, hash, role, is_active, created_at)) = row else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid email or password"}))).into_response();
    };

    if is_active == 0 {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Account is deactivated"}))).into_response();
    }

    if !verify_password(&req.password, &hash) {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid email or password"}))).into_response();
    }

    let auth_user = AuthUser {
        id: id.clone(),
        email: email.clone(),
        name: name.clone(),
        role: role.clone(),
    };

    let token = match create_jwt(&auth_user, &state.config.jwt_secret) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Token error"}))).into_response(),
    };

    log_audit(&conn, Some(&id), Some(&email), "auth.login", "user", Some(&id), "User signed in");

    Json(json!({
        "token": token,
        "user": User {
            id,
            email,
            name,
            role,
            is_active: true,
            created_at,
        }
    }))
    .into_response()
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if req.email.trim().is_empty() || req.password.len() < 6 || req.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Name, valid email, and password (6+ chars) required"})),
        )
            .into_response();
    }

    let conn = match state.db.get() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "DB error"}))).into_response(),
    };

    let email = req.email.trim().to_lowercase();
    let hash = match hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let role = "member".to_string();

    if let Err(_) = conn.execute(
        "INSERT INTO users (id, email, name, password_hash, role, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![id, email, req.name.trim(), hash, role, now],
    ) {
        return (StatusCode::CONFLICT, Json(json!({"error": "Email already registered"}))).into_response();
    }

    let auth_user = AuthUser {
        id: id.clone(),
        email: email.clone(),
        name: req.name.trim().to_string(),
        role: role.clone(),
    };

    let token = create_jwt(&auth_user, &state.config.jwt_secret).unwrap_or_default();
    log_audit(&conn, Some(&id), Some(&email), "auth.register", "user", Some(&id), "New user registered");

    (
        StatusCode::CREATED,
        Json(json!({
            "token": token,
            "user": User {
                id,
                email,
                name: req.name.trim().to_string(),
                role,
                is_active: true,
                created_at: now,
            }
        })),
    )
        .into_response()
}

pub async fn me(user: AuthUser) -> impl IntoResponse {
    Json(json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role,
    }))
}

pub async fn sso_public_config(State(state): State<AppState>) -> impl IntoResponse {
    let conn = match state.db.get() {
        Ok(c) => c,
        Err(_) => return Json(json!({"sso_enabled": false})),
    };
    let map = load_settings_map(&conn);
    let enabled = map.get("sso_enabled").map(|v| v == "true").unwrap_or(false);
    Json(json!({
        "sso_enabled": enabled,
        "provider_name": map.get("sso_provider_name").cloned().unwrap_or_default(),
        "issuer_url": map.get("sso_issuer_url").cloned().unwrap_or_default(),
    }))
}
