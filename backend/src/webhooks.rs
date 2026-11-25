use chrono::Utc;
use reqwest::Client;
use rusqlite::params;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::db::DbPool;

pub async fn trigger_webhook(
    client: &Client,
    pool: &DbPool,
    webhook_id: &str,
    rule_name: &str,
    article_title: &str,
    article_url: &str,
    feed_title: &str,
    priority_score: i64,
) {
    let wh: Option<(String, String, String, Option<String>)> = {
        let Ok(conn) = pool.get() else { return };
        conn.query_row(
            "SELECT name, kind, url, secret FROM webhooks WHERE id = ?1 AND is_enabled = 1",
            params![webhook_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .ok()
    };

    let Some((_name, kind, url, secret)) = wh else {
        return;
    };

    let payload = match kind.as_str() {
        "slack" => json!({
            "text": format!("*[PulseFeed Alert: {}]* (Score: {})\n<{}|{}>\n_Source: {}_", rule_name, priority_score, article_url, article_title, feed_title)
        }),
        "teams" => json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": "10B981",
            "summary": format!("PulseFeed Alert: {}", article_title),
            "sections": [{
                "activityTitle": article_title,
                "activitySubtitle": format!("Rule: {} | Source: {} | Priority: {}", rule_name, feed_title, priority_score),
                "markdown": true
            }],
            "potentialAction": [{
                "@type": "OpenUri",
                "name": "Read Article",
                "targets": [{ "os": "default", "uri": article_url }]
            }]
        }),
        _ => json!({
            "event": "leo.rule.matched",
            "rule": rule_name,
            "priority_score": priority_score,
            "article": {
                "title": article_title,
                "url": article_url,
                "feed": feed_title
            },
            "timestamp": Utc::now().to_rfc3339()
        }),
    };

    let body_str = serde_json::to_string(&payload).unwrap_or_default();
    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "PulseFeed-Enterprise/1.0")
        .body(body_str.clone());

    if let Some(sec) = secret.filter(|s| !s.is_empty()) {
        let mut hasher = Sha256::new();
        hasher.update(sec.as_bytes());
        hasher.update(body_str.as_bytes());
        let sig = hex::encode(hasher.finalize());
        req = req.header("X-PulseFeed-Signature", format!("sha256={}", sig));
    }

    let status_text = match req.timeout(std::time::Duration::from_secs(10)).send().await {
        Ok(resp) => format!("HTTP {}", resp.status().as_u16()),
        Err(e) => format!("Error: {}", e),
    };

    if let Ok(conn) = pool.get() {
        let _ = conn.execute(
            "UPDATE webhooks SET last_status = ?1, last_triggered_at = ?2 WHERE id = ?3",
            params![status_text, Utc::now().to_rfc3339(), webhook_id],
        );
    }
}
