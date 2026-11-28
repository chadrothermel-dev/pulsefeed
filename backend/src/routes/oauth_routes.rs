use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use url::form_urlencoded;
use uuid::Uuid;

use crate::{
    audit::log_audit,
    auth::{create_jwt, hash_password, AuthUser},
    leo::load_settings_map,
    AppState,
};

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

fn resolve_base_url(settings: &std::collections::HashMap<String, String>, headers: &HeaderMap) -> String {
    if let Some(configured) = settings.get("app_base_url").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        return configured.trim_end_matches('/').to_string();
    }
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1:8080");
    format!("{}://{}", proto, host)
}

/// Public endpoint: lists which OAuth2 / OIDC identity providers are enabled for the login screen.
pub async fn list_public_providers(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(conn) = state.db.get() else {
        return Json(json!({ "providers": [] }));
    };
    let m = load_settings_map(&conn);
    let enabled = |prefix: &str| {
        m.get(&format!("oauth_{}_enabled", prefix)).map(|v| v == "true").unwrap_or(false)
            && !m.get(&format!("oauth_{}_client_id", prefix)).map(|s| s.trim()).unwrap_or("").is_empty()
    };

    let mut providers = Vec::new();
    if enabled("google") {
        providers.push(json!({ "id": "google", "name": "Google" }));
    }
    if enabled("entra") {
        providers.push(json!({ "id": "entra", "name": "Microsoft Entra ID" }));
    }
    if enabled("facebook") {
        providers.push(json!({ "id": "facebook", "name": "Facebook" }));
    }

    Json(json!({ "providers": providers }))
}

/// Redirects the browser to Google, Microsoft Entra ID, or Facebook OAuth2 authorization screen.
pub async fn oauth_start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Ok(conn) = state.db.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response();
    };
    let m = load_settings_map(&conn);
    let base_url = resolve_base_url(&m, &headers);
    let redirect_uri = format!("{}/api/auth/oauth/{}/callback", base_url, provider);
    let state_token = Uuid::new_v4().to_string();

    let get = |k: &str| m.get(k).cloned().unwrap_or_default();
    let enc = |v: &str| -> String { form_urlencoded::byte_serialize(v.as_bytes()).collect() };

    let auth_url = match provider.as_str() {
        "google" => {
            let cid = get("oauth_google_client_id");
            if cid.is_empty() {
                return Redirect::temporary("/?oauth_error=Google+OAuth+is+not+configured").into_response();
            }
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&prompt=select_account",
                enc(&cid),
                enc(&redirect_uri),
                enc("openid email profile"),
                enc(&state_token)
            )
        }
        "entra" => {
            let cid = get("oauth_entra_client_id");
            let tenant = {
                let t = get("oauth_entra_tenant_id");
                if t.trim().is_empty() { "common".to_string() } else { t.trim().to_string() }
            };
            if cid.is_empty() {
                return Redirect::temporary("/?oauth_error=Microsoft+Entra+ID+is+not+configured").into_response();
            }
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&response_mode=query&scope={}&state={}",
                enc(&tenant),
                enc(&cid),
                enc(&redirect_uri),
                enc("openid email profile User.Read"),
                enc(&state_token)
            )
        }
        "facebook" => {
            let cid = get("oauth_facebook_client_id");
            if cid.is_empty() {
                return Redirect::temporary("/?oauth_error=Facebook+OAuth+is+not+configured").into_response();
            }
            format!(
                "https://www.facebook.com/v20.0/dialog/oauth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                enc(&cid),
                enc(&redirect_uri),
                enc("email,public_profile"),
                enc(&state_token)
            )
        }
        _ => return (StatusCode::BAD_REQUEST, "Unsupported provider").into_response(),
    };

    Redirect::temporary(&auth_url).into_response()
}

/// Exchanges the authorization code with Google / Microsoft Entra ID / Facebook,
/// provisions or loads the local user, issues a PulseFeed JWT, and redirects to `/?oauth_token=...`.
pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Query(q): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = q.error {
        let enc: String = form_urlencoded::byte_serialize(err.as_bytes()).collect();
        return Redirect::temporary(&format!("/?oauth_error={}", enc)).into_response();
    }
    let Some(code) = q.code else {
        return Redirect::temporary("/?oauth_error=Missing+authorization+code").into_response();
    };

    let m = {
        let Ok(conn) = state.db.get() else {
            return Redirect::temporary("/?oauth_error=Database+error").into_response();
        };
        load_settings_map(&conn)
    };
    let get = |k: &str| m.get(k).cloned().unwrap_or_default();
    let base_url = resolve_base_url(&m, &headers);
    let redirect_uri = format!("{}/api/auth/oauth/{}/callback", base_url, provider);

    let (email, name) = match provider.as_str() {
        "google" => {
            let cid = get("oauth_google_client_id");
            let csec = get("oauth_google_client_secret");
            let token_resp = state
                .http
                .post("https://oauth2.googleapis.com/token")
                .form(&[
                    ("code", code.as_str()),
                    ("client_id", cid.as_str()),
                    ("client_secret", csec.as_str()),
                    ("redirect_uri", redirect_uri.as_str()),
                    ("grant_type", "authorization_code"),
                ])
                .send()
                .await;
            let Ok(resp) = token_resp else {
                return Redirect::temporary("/?oauth_error=Google+token+exchange+failed").into_response();
            };
            let val: Value = resp.json().await.unwrap_or_default();
            let Some(access_token) = val["access_token"].as_str() else {
                return Redirect::temporary("/?oauth_error=Google+did+not+return+access+token").into_response();
            };
            match state
                .http
                .get("https://openidconnect.googleapis.com/v1/userinfo")
                .bearer_auth(access_token)
                .send()
                .await
            {
                Ok(r) => {
                    let u: Value = r.json().await.unwrap_or_default();
                    let em = u["email"].as_str().unwrap_or("").to_lowercase();
                    let nm = u["name"].as_str().unwrap_or(&em).to_string();
                    (em, nm)
                }
                Err(_) => return Redirect::temporary("/?oauth_error=Google+userinfo+failed").into_response(),
            }
        }
        "entra" => {
            let cid = get("oauth_entra_client_id");
            let csec = get("oauth_entra_client_secret");
            let tenant = {
                let t = get("oauth_entra_tenant_id");
                if t.trim().is_empty() { "common".to_string() } else { t.trim().to_string() }
            };
            let token_url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant);
            let token_resp = state
                .http
                .post(&token_url)
                .form(&[
                    ("code", code.as_str()),
                    ("client_id", cid.as_str()),
                    ("client_secret", csec.as_str()),
                    ("redirect_uri", redirect_uri.as_str()),
                    ("grant_type", "authorization_code"),
                    ("scope", "openid email profile User.Read"),
                ])
                .send()
                .await;
            let Ok(resp) = token_resp else {
                return Redirect::temporary("/?oauth_error=Entra+ID+token+exchange+failed").into_response();
            };
            let val: Value = resp.json().await.unwrap_or_default();
            let Some(access_token) = val["access_token"].as_str() else {
                return Redirect::temporary("/?oauth_error=Entra+ID+did+not+return+access+token").into_response();
            };
            match state
                .http
                .get("https://graph.microsoft.com/oidc/userinfo")
                .bearer_auth(access_token)
                .send()
                .await
            {
                Ok(r) => {
                    let u: Value = r.json().await.unwrap_or_default();
                    let em = u["email"]
                        .as_str()
                        .or_else(|| u["preferred_username"].as_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let nm = u["name"].as_str().unwrap_or(&em).to_string();
                    (em, nm)
                }
                Err(_) => return Redirect::temporary("/?oauth_error=Entra+ID+userinfo+failed").into_response(),
            }
        }
        "facebook" => {
            let cid = get("oauth_facebook_client_id");
            let csec = get("oauth_facebook_client_secret");
            let token_resp = state
                .http
                .get("https://graph.facebook.com/v20.0/oauth/access_token")
                .query(&[
                    ("client_id", cid.as_str()),
                    ("client_secret", csec.as_str()),
                    ("redirect_uri", redirect_uri.as_str()),
                    ("code", code.as_str()),
                ])
                .send()
                .await;
            let Ok(resp) = token_resp else {
                return Redirect::temporary("/?oauth_error=Facebook+token+exchange+failed").into_response();
            };
            let val: Value = resp.json().await.unwrap_or_default();
            let Some(access_token) = val["access_token"].as_str() else {
                return Redirect::temporary("/?oauth_error=Facebook+did+not+return+access+token").into_response();
            };
            match state
                .http
                .get("https://graph.facebook.com/me")
                .query(&[("fields", "id,name,email"), ("access_token", access_token)])
                .send()
                .await
            {
                Ok(r) => {
                    let u: Value = r.json().await.unwrap_or_default();
                    let fb_id = u["id"].as_str().unwrap_or("user");
                    let em = u["email"]
                        .as_str()
                        .map(|s| s.to_lowercase())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| format!("fb_{}@facebook.local", fb_id));
                    let nm = u["name"].as_str().unwrap_or("Facebook User").to_string();
                    (em, nm)
                }
                Err(_) => return Redirect::temporary("/?oauth_error=Facebook+profile+fetch+failed").into_response(),
            }
        }
        _ => return Redirect::temporary("/?oauth_error=Unsupported+provider").into_response(),
    };

    if email.is_empty() {
        return Redirect::temporary("/?oauth_error=Identity+provider+did+not+return+an+email").into_response();
    }

    let Ok(conn) = state.db.get() else {
        return Redirect::temporary("/?oauth_error=Database+error").into_response();
    };

    // Match existing user or auto-provision (JIT) as member
    let existing: Option<(String, String, String, i64)> = conn
        .query_row(
            "SELECT id, name, role, is_active FROM users WHERE lower(email) = ?1",
            params![email],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .ok();

    let auth_user = if let Some((id, existing_name, role, is_active)) = existing {
        if is_active == 0 {
            return Redirect::temporary("/?oauth_error=Your+account+is+disabled").into_response();
        }
        AuthUser {
            id,
            email: email.clone(),
            name: existing_name,
            role,
        }
    } else {
        let new_id = Uuid::new_v4().to_string();
        let random_pw = Uuid::new_v4().to_string();
        let pw_hash = hash_password(&random_pw).unwrap_or_default();
        let now = Utc::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT INTO users (id, email, name, password_hash, role, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, 'member', 1, ?5)",
            params![new_id, email, name, pw_hash, now],
        );
        AuthUser {
            id: new_id,
            email: email.clone(),
            name,
            role: "member".to_string(),
        }
    };

    let Ok(jwt) = create_jwt(&auth_user, &state.config.jwt_secret) else {
        return Redirect::temporary("/?oauth_error=Failed+to+issue+session+token").into_response();
    };

    log_audit(
        &conn,
        Some(&auth_user.id),
        Some(&auth_user.email),
        "auth.oauth.login",
        "user",
        Some(&auth_user.id),
        &format!("Signed in via {}", provider),
    );

    Redirect::temporary(&format!("/?oauth_token={}", jwt)).into_response()
}
