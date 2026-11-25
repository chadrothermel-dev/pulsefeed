use regex::Regex;
use reqwest::Client;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::models::LeoRule;

/// Compute a 64-bit SimHash for fast O(1) near-duplicate detection across unlimited feeds.
pub fn compute_simhash(title: &str, content: &str) -> i64 {
    let mut v = [0i32; 64];

    let process = |text: &str, weight: i32, vec: &mut [i32; 64]| {
        for token in text
            .split(|c: char| !c.is_alphanumeric())
            .map(|t| t.to_lowercase())
            .filter(|t| t.len() >= 3)
        {
            // FNV-1a 64-bit hash
            let mut h: u64 = 0xcbf29ce484222325;
            for b in token.as_bytes() {
                h ^= *b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            for i in 0..64 {
                if (h >> i) & 1 == 1 {
                    vec[i] += weight;
                } else {
                    vec[i] -= weight;
                }
            }
        }
    };

    // Weight title 3x so headline similarity strongly clusters syndicated stories
    process(title, 3, &mut v);
    let snippet: String = content.chars().take(1200).collect();
    process(&snippet, 1, &mut v);

    let mut fingerprint: u64 = 0;
    for i in 0..64 {
        if v[i] > 0 {
            fingerprint |= 1u64 << i;
        }
    }
    fingerprint as i64
}

pub fn hamming_distance(a: i64, b: i64) -> u32 {
    ((a as u64) ^ (b as u64)).count_ones()
}

/// Find an existing duplicate cluster among recent articles (last 7 days).
pub fn find_duplicate_cluster(
    conn: &Connection,
    simhash: i64,
    threshold: u32,
) -> Option<String> {
    if simhash == 0 {
        return None;
    }
    let mut stmt = conn
        .prepare(
            "SELECT id, simhash, duplicate_cluster_id
             FROM articles
             WHERE published_at >= datetime('now', '-7 days')
             ORDER BY published_at DESC
             LIMIT 1500",
        )
        .ok()?;

    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .ok()?;

    for row in rows.flatten() {
        let (other_id, other_hash, existing_cluster) = row;
        if hamming_distance(simhash, other_hash) <= threshold {
            let cluster_id = existing_cluster.unwrap_or_else(|| other_id.clone());
            // Ensure the matched anchor article also has duplicate_cluster_id set
            let _ = conn.execute(
                "UPDATE articles SET duplicate_cluster_id = ?1 WHERE id = ?2 AND duplicate_cluster_id IS NULL",
                params![cluster_id, other_id],
            );
            return Some(cluster_id);
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct LeoEvaluation {
    pub priority_score: i64,
    pub is_muted: bool,
    pub tags: Vec<String>,
    pub auto_board_ids: Vec<String>,
    pub webhook_alerts: Vec<(String, String)>, // (webhook_id, rule_name)
    pub matched_rule_ids: Vec<String>,
}

pub fn load_enabled_rules(conn: &Connection) -> Vec<LeoRule> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, name, rule_type, keywords, match_field, feed_id, folder_id,
                score_delta, target_tag, target_board_id, target_webhook_id,
                is_enabled, match_count, created_at
         FROM leo_rules
         WHERE is_enabled = 1",
    ) else {
        return vec![];
    };

    stmt.query_map([], |r| {
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
    .map(|iter| iter.flatten().collect())
    .unwrap_or_default()
}

fn matches_keywords(haystack: &str, keywords_spec: &str) -> bool {
    let haystack_lower = haystack.to_lowercase();
    for raw_term in keywords_spec.split(',') {
        let term = raw_term.trim();
        if term.is_empty() {
            continue;
        }
        if term.starts_with('/') && term.ends_with('/') && term.len() > 2 {
            let pattern = &term[1..term.len() - 1];
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                if re.is_match(haystack) {
                    return true;
                }
            }
        } else if haystack_lower.contains(&term.to_lowercase()) {
            return true;
        }
    }
    false
}

pub fn evaluate_article(
    rules: &[LeoRule],
    feed_id: &str,
    folder_id: Option<&str>,
    title: &str,
    content_text: &str,
    author: Option<&str>,
) -> LeoEvaluation {
    let mut eval = LeoEvaluation {
        priority_score: 50,
        ..Default::default()
    };
    let mut tag_set: HashSet<String> = HashSet::new();
    let combined = format!("{} {} {}", title, content_text, author.unwrap_or(""));

    for rule in rules {
        if let Some(ref rf) = rule.feed_id {
            if rf != feed_id {
                continue;
            }
        }
        if let Some(ref rfol) = rule.folder_id {
            if folder_id != Some(rfol.as_str()) {
                continue;
            }
        }

        let target_text = match rule.match_field.as_str() {
            "title" => title,
            "content" => content_text,
            "author" => author.unwrap_or(""),
            _ => &combined,
        };

        if matches_keywords(target_text, &rule.keywords) {
            eval.matched_rule_ids.push(rule.id.clone());

            match rule.rule_type.as_str() {
                "mute" => {
                    eval.is_muted = true;
                    eval.priority_score += rule.score_delta.min(-25);
                }
                "priority" => {
                    eval.priority_score += rule.score_delta;
                }
                "auto_tag" => {
                    eval.priority_score += rule.score_delta.max(0);
                }
                "auto_board" => {
                    eval.priority_score += rule.score_delta.max(10);
                }
                "alert" => {
                    eval.priority_score += rule.score_delta.max(20);
                }
                _ => {}
            }

            if let Some(ref tag) = rule.target_tag {
                let clean = tag.trim().to_lowercase();
                if !clean.is_empty() {
                    tag_set.insert(clean);
                }
            }
            if let Some(ref bid) = rule.target_board_id {
                if !bid.is_empty() && !eval.auto_board_ids.contains(bid) {
                    eval.auto_board_ids.push(bid.clone());
                }
            }
            if let Some(ref wid) = rule.target_webhook_id {
                if !wid.is_empty() {
                    eval.webhook_alerts.push((wid.clone(), rule.name.clone()));
                }
            }
        }
    }

    eval.priority_score = eval.priority_score.clamp(0, 100);
    eval.tags = tag_set.into_iter().collect();
    eval.tags.sort();
    eval
}

pub fn estimate_reading_time(content_text: &str) -> i64 {
    let words = content_text.split_whitespace().count();
    ((words as f64 / 220.0).ceil() as i64).max(1)
}

/// Load settings map from SQLite.
pub fn load_settings_map(conn: &Connection) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM settings") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
            for (k, v) in rows.flatten() {
                map.insert(k, v);
            }
        }
    }
    map
}

/// Hybrid AI Summary: uses configured LLM (Gemini / OpenAI / Ollama) or falls back to
/// fast local extractive intelligence so it always works cleanly even offline!
pub async fn generate_ai_summary(
    client: &Client,
    settings: &HashMap<String, String>,
    title: &str,
    content_text: &str,
) -> (String, String) {
    let provider = settings
        .get("llm_provider")
        .map(|s| s.as_str())
        .unwrap_or("none");
    let model = settings
        .get("llm_model")
        .map(|s| s.as_str())
        .unwrap_or("gemini-2.5-flash");
    let api_key = settings
        .get("llm_api_key")
        .map(|s| s.as_str())
        .unwrap_or("");
    let base_url = settings
        .get("llm_base_url")
        .map(|s| s.as_str())
        .unwrap_or("");

    let truncated: String = content_text.chars().take(6000).collect();
    let prompt = format!(
        "Summarize the following article in 3 concise executive bullet points and classify its overall tone as one word (Positive, Neutral, Critical, or Urgent). Return JSON with keys \"summary\" (string with markdown bullets) and \"sentiment\" (one word).\n\nTitle: {}\n\nContent:\n{}",
        title, truncated
    );

    if provider == "gemini" && !api_key.is_empty() {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );
        let body = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "generationConfig": { "responseMimeType": "application/json" }
        });
        if let Ok(resp) = client.post(&url).json(&body).send().await {
            if let Ok(val) = resp.json::<Value>().await {
                if let Some(text) = val["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                        let summary = parsed["summary"].as_str().unwrap_or(text).to_string();
                        let sentiment = parsed["sentiment"].as_str().unwrap_or("Neutral").to_string();
                        return (summary, sentiment);
                    }
                    return (text.to_string(), "Neutral".to_string());
                }
            }
        }
    } else if provider == "openai" && !api_key.is_empty() {
        let endpoint = if base_url.is_empty() {
            "https://api.openai.com/v1/chat/completions".to_string()
        } else {
            format!("{}/chat/completions", base_url.trim_end_matches('/'))
        };
        let body = json!({
            "model": model,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": "You are an executive intelligence analyst." },
                { "role": "user", "content": prompt }
            ]
        });
        if let Ok(resp) = client
            .post(&endpoint)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
        {
            if let Ok(val) = resp.json::<Value>().await {
                if let Some(text) = val["choices"][0]["message"]["content"].as_str() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                        let summary = parsed["summary"].as_str().unwrap_or(text).to_string();
                        let sentiment = parsed["sentiment"].as_str().unwrap_or("Neutral").to_string();
                        return (summary, sentiment);
                    }
                    return (text.to_string(), "Neutral".to_string());
                }
            }
        }
    } else if provider == "ollama" {
        let endpoint = if base_url.is_empty() {
            "http://127.0.0.1:11434/api/generate".to_string()
        } else {
            format!("{}/api/generate", base_url.trim_end_matches('/'))
        };
        let body = json!({
            "model": if model.is_empty() { "llama3.2" } else { model },
            "prompt": prompt,
            "format": "json",
            "stream": false
        });
        if let Ok(resp) = client.post(&endpoint).json(&body).send().await {
            if let Ok(val) = resp.json::<Value>().await {
                if let Some(text) = val["response"].as_str() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                        let summary = parsed["summary"].as_str().unwrap_or(text).to_string();
                        let sentiment = parsed["sentiment"].as_str().unwrap_or("Neutral").to_string();
                        return (summary, sentiment);
                    }
                    return (text.to_string(), "Neutral".to_string());
                }
            }
        }
    }

    // Fast Local Extractive Intelligence Fallback (zero external API required)
    local_extractive_summary(title, content_text)
}

fn local_extractive_summary(title: &str, content_text: &str) -> (String, String) {
    let sentences: Vec<&str> = content_text
        .split(['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| s.len() > 35 && s.len() < 320)
        .collect();

    let title_words: HashSet<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 4)
        .collect();

    let mut scored: Vec<(usize, usize, &str)> = sentences
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let lower = s.to_lowercase();
            let overlap = title_words
                .iter()
                .filter(|w| lower.contains(w.as_str()))
                .count();
            let pos_bonus = if idx < 3 { 2 } else { 0 };
            (overlap * 3 + pos_bonus, idx, *s)
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut top: Vec<(usize, &str)> = scored.into_iter().take(3).map(|(_, i, s)| (i, s)).collect();
    top.sort_by_key(|(i, _)| *i);

    let summary = if top.is_empty() {
        format!("• {}", title)
    } else {
        top.into_iter()
            .map(|(_, s)| format!("• {}.", s))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let lower_all = format!("{} {}", title, content_text).to_lowercase();
    let sentiment = if ["zero-day", "breach", "ransomware", "critical", "exploit", "outage", "cve-"]
        .iter()
        .any(|k| lower_all.contains(k))
    {
        "Urgent"
    } else if ["warning", "risk", "vulnerability", "decline", "layoff", "lawsuit"]
        .iter()
        .any(|k| lower_all.contains(k))
    {
        "Critical"
    } else if ["launch", "breakthrough", "release", "growth", "record", "milestone"]
        .iter()
        .any(|k| lower_all.contains(k))
    {
        "Positive"
    } else {
        "Neutral"
    };

    (summary, sentiment.to_string())
}
