use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn can_edit(&self) -> bool {
        matches!(self.role.as_str(), "admin" | "editor" | "member")
    }

    pub fn can_manage_feeds(&self) -> bool {
        matches!(self.role.as_str(), "admin" | "editor")
    }
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn create_jwt(user: &AuthUser, secret: &str) -> Result<String, String> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

pub fn hash_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    hex::encode(hasher.finalize())
}

pub struct AuthError(pub StatusCode, pub String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AuthError(StatusCode::UNAUTHORIZED, "Missing Authorization header".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthError(StatusCode::UNAUTHORIZED, "Invalid Authorization scheme".into()))?
            .trim();

        // Support Enterprise API Keys (prefix pf_)
        if token.starts_with("pf_") {
            let key_hash = hash_api_key(token);
            let conn = state
                .db
                .get()
                .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?;

            let row: Option<(String, String, String, String, String)> = conn
                .query_row(
                    "SELECT k.id, u.id, u.email, u.name, u.role
                     FROM api_keys k
                     JOIN users u ON u.id = k.user_id
                     WHERE k.key_hash = ?1 AND u.is_active = 1",
                    params![key_hash],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )
                .ok();

            if let Some((key_id, id, email, name, role)) = row {
                let _ = conn.execute(
                    "UPDATE api_keys SET last_used_at = ?1 WHERE id = ?2",
                    params![Utc::now().to_rfc3339(), key_id],
                );
                return Ok(AuthUser { id, email, name, role });
            }
            return Err(AuthError(StatusCode::UNAUTHORIZED, "Invalid API key".into()));
        }

        // Otherwise verify JWT
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError(StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))?;

        let conn = state
            .db
            .get()
            .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?;

        let active_user: Option<(String, String, String, String)> = conn
            .query_row(
                "SELECT id, email, name, role FROM users WHERE id = ?1 AND is_active = 1",
                params![decoded.claims.sub],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();

        match active_user {
            Some((id, email, name, role)) => Ok(AuthUser { id, email, name, role }),
            None => Err(AuthError(StatusCode::UNAUTHORIZED, "Account disabled or not found".into())),
        }
    }
}
