use chrono::Utc;
use rusqlite::{params, Connection};

pub fn log_audit(
    conn: &Connection,
    user_id: Option<&str>,
    user_email: Option<&str>,
    action: &str,
    target_type: &str,
    target_id: Option<&str>,
    details: &str,
) {
    let _ = conn.execute(
        "INSERT INTO audit_logs (user_id, user_email, action, target_type, target_id, details, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            user_id,
            user_email,
            action,
            target_type,
            target_id,
            details,
            Utc::now().to_rfc3339()
        ],
    );
}
