use ammonia::Builder;
use chrono::Utc;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use reqwest::Client;
use rusqlite::params;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

use crate::db::DbPool;
use crate::leo;
use crate::webhooks;

#[derive(Clone, Debug, serde::Serialize)]
pub struct DiscoveredFeed {
    pub title: String,
    pub feed_url: String,
    pub site_url: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RefreshSummary {
    pub feeds_checked: usize,
    pub feeds_updated: usize,
    pub new_articles: usize,
    pub errors: usize,
}

fn sanitize_html(raw: &str) -> String {
    let mut tags = HashSet::new();
    for t in [
        "p", "br", "b", "i", "em", "strong", "a", "ul", "ol", "li", "blockquote", "code", "pre",
        "h1", "h2", "h3", "h4", "h5", "h6", "img", "figure", "figcaption", "table", "thead",
        "tbody", "tr", "th", "td", "hr", "span", "div", "sup", "sub",
    ] {
        tags.insert(t);
    }
    let mut builder = Builder::new();
    builder
        .tags(tags)
        .add_tag_attributes("a", &["href", "title"])
        .add_tag_attributes("img", &["src", "alt", "title", "width", "height"])
        .link_rel(Some("noopener noreferrer"));
    builder.clean(raw).to_string()
}

fn html_to_plain_text(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let text: Vec<&str> = doc.root_element().text().collect();
    text.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_first_image(html: &str) -> Option<String> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img").ok()?;
    for el in doc.select(&sel) {
        if let Some(src) = el.value().attr("src") {
            let s = src.trim();
            if s.starts_with("http://") || s.starts_with("https://") {
                if !s.contains("pixel") && !s.contains("1x1") && !s.contains("feedburner") {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

pub async fn discover_feeds_from_url(client: &Client, raw_url: &str) -> Result<Vec<DiscoveredFeed>, String> {
    let normalized = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
        raw_url.to_string()
    } else {
        format!("https://{}", raw_url)
    };

    let base = Url::parse(&normalized).map_err(|e| format!("Invalid URL: {}", e))?;
    let resp = client
        .get(base.as_str())
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let final_url = resp.url().clone();
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // Check if the URL itself is already a valid RSS / Atom / JSON feed
    if let Ok(parsed) = feed_rs::parser::parse(&bytes[..]) {
        let title = parsed
            .title
            .map(|t| t.content)
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| final_url.host_str().unwrap_or("RSS Feed").to_string());
        let site = parsed
            .links
            .first()
            .map(|l| l.href.clone())
            .unwrap_or_else(|| final_url.origin().ascii_serialization());
        return Ok(vec![DiscoveredFeed {
            title,
            feed_url: final_url.to_string(),
            site_url: site,
        }]);
    }

    // Otherwise parse HTML <link rel="alternate" type="application/rss+xml|atom+xml|feed+json">
    let html = String::from_utf8_lossy(&bytes);
    let doc = Html::parse_document(&html);
    let link_sel = Selector::parse("link[rel~='alternate']").unwrap();
    let mut found = Vec::new();
    let mut seen = HashSet::new();

    for el in doc.select(&link_sel) {
        let typ = el.value().attr("type").unwrap_or("").to_lowercase();
        if typ.contains("rss") || typ.contains("atom") || typ.contains("feed+json") || typ.contains("xml") {
            if let Some(href) = el.value().attr("href") {
                if let Ok(resolved) = final_url.join(href) {
                    let u = resolved.to_string();
                    if seen.insert(u.clone()) {
                        let title = el
                            .value()
                            .attr("title")
                            .map(|s| s.to_string())
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or_else(|| final_url.host_str().unwrap_or("Discovered Feed").to_string());
                        found.push(DiscoveredFeed {
                            title,
                            feed_url: u,
                            site_url: final_url.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(found)
}

pub async fn refresh_single_feed(
    client: &Client,
    pool: &DbPool,
    feed_id: &str,
) -> Result<usize, String> {
    let feed_meta: (String, Option<String>, Option<String>, Option<String>, String) = {
        let conn = pool.get().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT feed_url, folder_id, etag, last_modified, title FROM feeds WHERE id = ?1",
            params![feed_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|e| format!("Feed not found: {}", e))?
    };

    let (feed_url, folder_id, etag, last_modified, feed_title) = feed_meta;
    let now = Utc::now().to_rfc3339();

    let mut req = client
        .get(&feed_url)
        .header("User-Agent", "PulseFeed/1.0 (+https://pulsefeed.local)")
        .timeout(std::time::Duration::from_secs(20));

    if let Some(ref e) = etag {
        if !e.is_empty() {
            req = req.header("If-None-Match", e);
        }
    }
    if let Some(ref lm) = last_modified {
        if !lm.is_empty() {
            req = req.header("If-Modified-Since", lm);
        }
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(err) => {
            if let Ok(conn) = pool.get() {
                let _ = conn.execute(
                    "UPDATE feeds SET last_fetched_at = ?1, last_error = ?2, error_count = error_count + 1 WHERE id = ?3",
                    params![now, err.to_string(), feed_id],
                );
            }
            return Err(err.to_string());
        }
    };

    // HTTP 304 Not Modified — super fast zero-parse path!
    if resp.status().as_u16() == 304 {
        if let Ok(conn) = pool.get() {
            let _ = conn.execute(
                "UPDATE feeds SET last_fetched_at = ?1, last_error = NULL, error_count = 0 WHERE id = ?2",
                params![now, feed_id],
            );
        }
        return Ok(0);
    }

    if !resp.status().is_success() {
        let err_msg = format!("HTTP {}", resp.status());
        if let Ok(conn) = pool.get() {
            let _ = conn.execute(
                "UPDATE feeds SET last_fetched_at = ?1, last_error = ?2, error_count = error_count + 1 WHERE id = ?3",
                params![now, err_msg, feed_id],
            );
        }
        return Err(err_msg);
    }

    let new_etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let new_lm = resp
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let parsed = match feed_rs::parser::parse(&bytes[..]) {
        Ok(p) => p,
        Err(err) => {
            let err_msg = format!("Parse error: {}", err);
            if let Ok(conn) = pool.get() {
                let _ = conn.execute(
                    "UPDATE feeds SET last_fetched_at = ?1, last_error = ?2, error_count = error_count + 1 WHERE id = ?3",
                    params![now, err_msg, feed_id],
                );
            }
            return Err(err_msg);
        }
    };

    let site_url = parsed.links.first().map(|l| l.href.clone());
    let description = parsed.description.map(|d| html_to_plain_text(&d.content));
    let icon_url = parsed
        .icon
        .map(|i| i.uri)
        .or_else(|| parsed.logo.map(|l| l.uri))
        .or_else(|| {
            site_url.as_deref().and_then(|su| {
                Url::parse(su)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| format!("https://www.google.com/s2/favicons?domain={}&sz=64", h)))
            })
        });

    let mut webhooks_to_fire: Vec<(String, String, String, String, i64)> = Vec::new();
    let mut new_count = 0usize;

    {
        let conn = pool.get().map_err(|e| e.to_string())?;
        let rules = leo::load_enabled_rules(&conn);
        let settings = leo::load_settings_map(&conn);
        let dedup_threshold: u32 = settings
            .get("dedup_threshold")
            .and_then(|v| v.parse().ok())
            .unwrap_or(6);

        let admin_id: Option<String> = conn
            .query_row("SELECT id FROM users WHERE role = 'admin' LIMIT 1", [], |r| r.get(0))
            .ok();

        let _ = conn.execute(
            "UPDATE feeds
             SET etag = COALESCE(?1, etag),
                 last_modified = COALESCE(?2, last_modified),
                 site_url = COALESCE(?3, site_url),
                 description = COALESCE(?4, description),
                 icon_url = COALESCE(?5, icon_url),
                 last_fetched_at = ?6,
                 last_error = NULL,
                 error_count = 0
             WHERE id = ?7",
            params![new_etag, new_lm, site_url, description, icon_url, now, feed_id],
        );

        for entry in parsed.entries.into_iter().take(60) {
            let guid = if entry.id.trim().is_empty() {
                entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_else(|| Uuid::new_v4().to_string())
            } else {
                entry.id.clone()
            };

            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM articles WHERE feed_id = ?1 AND guid = ?2",
                    params![feed_id, guid],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if exists {
                continue;
            }

            let title = entry
                .title
                .map(|t| html_to_plain_text(&t.content))
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "Untitled Article".to_string());

            let url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_else(|| feed_url.clone());

            let author = entry.authors.first().map(|a| a.name.clone());

            let raw_html = entry
                .content
                .and_then(|c| c.body)
                .or_else(|| entry.summary.as_ref().map(|s| s.content.clone()))
                .unwrap_or_default();

            let safe_html = sanitize_html(&raw_html);
            let plain_text = html_to_plain_text(&safe_html);
            let summary: String = plain_text.chars().take(280).collect();

            let image_url = entry
                .media
                .first()
                .and_then(|m| {
                    m.thumbnails
                        .first()
                        .map(|t| t.image.uri.clone())
                        .or_else(|| {
                            m.content
                                .iter()
                                .find(|c| {
                                    c.content_type
                                        .as_ref()
                                        .map(|ct| ct.ty().as_str() == "image")
                                        .unwrap_or(false)
                                })
                                .and_then(|c| c.url.as_ref().map(|u| u.to_string()))
                        })
                })
                .or_else(|| extract_first_image(&safe_html));

            let published_at = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| now.clone());

            let reading_time = leo::estimate_reading_time(&plain_text);
            let simhash = leo::compute_simhash(&title, &plain_text);
            let dup_cluster = leo::find_duplicate_cluster(&conn, simhash, dedup_threshold);

            let eval = leo::evaluate_article(
                &rules,
                feed_id,
                folder_id.as_deref(),
                &title,
                &plain_text,
                author.as_deref(),
            );

            let article_id = Uuid::new_v4().to_string();
            let tags_json = serde_json::to_string(&eval.tags).unwrap_or_else(|_| "[]".to_string());

            let inserted = conn.execute(
                "INSERT OR IGNORE INTO articles
                 (id, feed_id, guid, title, url, author, summary, content_html, content_text,
                  image_url, published_at, fetched_at, reading_time_mins, simhash,
                  duplicate_cluster_id, priority_score, is_muted, tags_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    article_id,
                    feed_id,
                    guid,
                    title,
                    url,
                    author,
                    summary,
                    safe_html,
                    plain_text,
                    image_url,
                    published_at,
                    now,
                    reading_time,
                    simhash,
                    dup_cluster,
                    eval.priority_score,
                    if eval.is_muted { 1 } else { 0 },
                    tags_json
                ],
            );

            if matches!(inserted, Ok(n) if n > 0) {
                new_count += 1;

                for rid in &eval.matched_rule_ids {
                    let _ = conn.execute(
                        "UPDATE leo_rules SET match_count = match_count + 1 WHERE id = ?1",
                        params![rid],
                    );
                }

                if let Some(ref uid) = admin_id {
                    for bid in &eval.auto_board_ids {
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO board_articles (board_id, article_id, added_by, added_at)
                             VALUES (?1, ?2, ?3, ?4)",
                            params![bid, article_id, uid, now],
                        );
                    }
                }

                for (wid, rname) in eval.webhook_alerts {
                    webhooks_to_fire.push((wid, rname, title.clone(), url.clone(), eval.priority_score));
                }
            }
        }
    }

    for (wid, rname, title, url, score) in webhooks_to_fire {
        webhooks::trigger_webhook(client, pool, &wid, &rname, &title, &url, &feed_title, score).await;
    }

    Ok(new_count)
}

pub async fn refresh_all_feeds(
    client: Client,
    pool: DbPool,
    max_concurrency: usize,
) -> RefreshSummary {
    let feed_ids: Vec<String> = {
        let Ok(conn) = pool.get() else {
            return RefreshSummary {
                feeds_checked: 0,
                feeds_updated: 0,
                new_articles: 0,
                errors: 1,
            };
        };
        let Ok(mut stmt) = conn.prepare("SELECT id FROM feeds WHERE is_active = 1") else {
            return RefreshSummary {
                feeds_checked: 0,
                feeds_updated: 0,
                new_articles: 0,
                errors: 1,
            };
        };
        stmt.query_map([], |r| r.get(0))
            .map(|iter| iter.flatten().collect())
            .unwrap_or_default()
    };

    let total = feed_ids.len();
    let sem = Arc::new(Semaphore::new(max_concurrency.max(1)));
    let mut handles = Vec::with_capacity(total);

    for fid in feed_ids {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let c = client.clone();
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            refresh_single_feed(&c, &p, &fid).await
        }));
    }

    let mut updated = 0usize;
    let mut new_articles = 0usize;
    let mut errors = 0usize;

    for h in handles {
        match h.await {
            Ok(Ok(n)) => {
                if n > 0 {
                    updated += 1;
                    new_articles += n;
                }
            }
            Ok(Err(e)) => {
                warn!("Feed refresh error: {}", e);
                errors += 1;
            }
            Err(_) => {
                errors += 1;
            }
        }
    }

    info!(
        "Feed refresh completed: checked={}, updated={}, new_articles={}, errors={}",
        total, updated, new_articles, errors
    );

    RefreshSummary {
        feeds_checked: total,
        feeds_updated: updated,
        new_articles,
        errors,
    }
}

/// Import unlimited feeds & folders from an OPML 2.0 XML document.
pub fn import_opml(pool: &DbPool, xml_content: &str) -> Result<(usize, usize), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut folder_stack: Vec<Option<String>> = Vec::new();
    let mut folder_cache: HashMap<String, String> = HashMap::new();

    if let Ok(mut stmt) = conn.prepare("SELECT name, id FROM folders") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
            for (name, id) in rows.flatten() {
                folder_cache.insert(name.to_lowercase(), id);
            }
        }
    }

    let now = Utc::now().to_rfc3339();
    let mut folders_created = 0usize;
    let mut feeds_imported = 0usize;

    let get_attr = |e: &BytesStart, key: &[u8]| -> Option<String> {
        e.attributes()
            .flatten()
            .find(|a| a.key.as_ref().eq_ignore_ascii_case(key))
            .and_then(|a| a.unescape_value().ok())
            .map(|v| v.into_owned())
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref().eq_ignore_ascii_case(b"outline") => {
                let xml_url = get_attr(e, b"xmlUrl");
                let title = get_attr(e, b"title")
                    .or_else(|| get_attr(e, b"text"))
                    .unwrap_or_else(|| "Untitled".to_string());

                if let Some(url) = xml_url {
                    let html_url = get_attr(e, b"htmlUrl");
                    let current_folder = folder_stack.last().cloned().flatten();
                    let res = conn.execute(
                        "INSERT OR IGNORE INTO feeds (id, folder_id, title, feed_url, site_url, is_active, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                        params![Uuid::new_v4().to_string(), current_folder, title, url, html_url, now],
                    );
                    if matches!(res, Ok(n) if n > 0) {
                        feeds_imported += 1;
                    }
                    folder_stack.push(current_folder);
                } else {
                    // Folder outline
                    let key = title.to_lowercase();
                    let fid = if let Some(existing) = folder_cache.get(&key) {
                        existing.clone()
                    } else {
                        let new_id = Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO folders (id, name, color, icon, sort_order, created_at)
                             VALUES (?1, ?2, '#10b981', 'folder', 50, ?3)",
                            params![new_id, title, now],
                        );
                        folder_cache.insert(key, new_id.clone());
                        folders_created += 1;
                        new_id
                    };
                    folder_stack.push(Some(fid));
                }
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref().eq_ignore_ascii_case(b"outline") => {
                if let Some(url) = get_attr(e, b"xmlUrl") {
                    let title = get_attr(e, b"title")
                        .or_else(|| get_attr(e, b"text"))
                        .unwrap_or_else(|| url.clone());
                    let html_url = get_attr(e, b"htmlUrl");
                    let current_folder = folder_stack.last().cloned().flatten();
                    let res = conn.execute(
                        "INSERT OR IGNORE INTO feeds (id, folder_id, title, feed_url, site_url, is_active, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                        params![Uuid::new_v4().to_string(), current_folder, title, url, html_url, now],
                    );
                    if matches!(res, Ok(n) if n > 0) {
                        feeds_imported += 1;
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref().eq_ignore_ascii_case(b"outline") => {
                folder_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Invalid OPML XML: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok((folders_created, feeds_imported))
}

/// Export all folders and feeds as a standards-compliant OPML 2.0 XML file.
pub fn export_opml(pool: &DbPool) -> Result<String, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    let mut opml = BytesStart::new("opml");
    opml.push_attribute(("version", "2.0"));
    writer.write_event(Event::Start(opml)).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("head"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::Start(BytesStart::new("title"))).map_err(|e| e.to_string())?;
    writer
        .write_event(Event::Text(quick_xml::events::BytesText::new("PulseFeed Subscriptions")))
        .map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("title"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("head"))).map_err(|e| e.to_string())?;

    writer.write_event(Event::Start(BytesStart::new("body"))).map_err(|e| e.to_string())?;

    let mut folder_stmt = conn
        .prepare("SELECT id, name FROM folders ORDER BY sort_order ASC, name ASC")
        .map_err(|e| e.to_string())?;
    let folders: Vec<(String, String)> = folder_stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    for (fid, fname) in folders {
        let mut f_el = BytesStart::new("outline");
        f_el.push_attribute(("text", fname.as_str()));
        f_el.push_attribute(("title", fname.as_str()));
        writer.write_event(Event::Start(f_el)).map_err(|e| e.to_string())?;

        let mut feed_stmt = conn
            .prepare("SELECT title, feed_url, site_url FROM feeds WHERE folder_id = ?1 ORDER BY title ASC")
            .map_err(|e| e.to_string())?;
        let feeds: Vec<(String, String, Option<String>)> = feed_stmt
            .query_map(params![fid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| e.to_string())?
            .flatten()
            .collect();

        for (title, furl, surl) in feeds {
            let mut el = BytesStart::new("outline");
            el.push_attribute(("type", "rss"));
            el.push_attribute(("text", title.as_str()));
            el.push_attribute(("title", title.as_str()));
            el.push_attribute(("xmlUrl", furl.as_str()));
            if let Some(ref s) = surl {
                el.push_attribute(("htmlUrl", s.as_str()));
            }
            writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
        }

        writer.write_event(Event::End(BytesEnd::new("outline"))).map_err(|e| e.to_string())?;
    }

    // Uncategorized feeds
    let mut uncat_stmt = conn
        .prepare("SELECT title, feed_url, site_url FROM feeds WHERE folder_id IS NULL ORDER BY title ASC")
        .map_err(|e| e.to_string())?;
    let uncat: Vec<(String, String, Option<String>)> = uncat_stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    for (title, furl, surl) in uncat {
        let mut el = BytesStart::new("outline");
        el.push_attribute(("type", "rss"));
        el.push_attribute(("text", title.as_str()));
        el.push_attribute(("title", title.as_str()));
        el.push_attribute(("xmlUrl", furl.as_str()));
        if let Some(ref s) = surl {
            el.push_attribute(("htmlUrl", s.as_str()));
        }
        writer.write_event(Event::Empty(el)).map_err(|e| e.to_string())?;
    }

    writer.write_event(Event::End(BytesEnd::new("body"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("opml"))).map_err(|e| e.to_string())?;

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
