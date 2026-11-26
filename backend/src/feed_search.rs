use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use url::form_urlencoded;

#[derive(Clone, Debug, Serialize)]
pub struct FeedSearchResult {
    pub title: String,
    pub feed_url: String,
    pub site_url: String,
    pub description: String,
    pub category: String,
    pub icon_url: Option<String>,
    pub subscribers: Option<i64>,
    pub source: String, // "directory" | "live" | "topic_feed"
    pub already_subscribed: bool,
}

struct CuratedEntry {
    title: &'static str,
    feed_url: &'static str,
    site_url: &'static str,
    description: &'static str,
    category: &'static str,
    keywords: &'static str,
}

const CURATED_FEEDS: &[CuratedEntry] = &[
    // Tech & Engineering
    CuratedEntry { title: "Hacker News (Front Page)", feed_url: "https://hnrss.org/frontpage", site_url: "https://news.ycombinator.com", description: "Top technology, startup, and programming links from Hacker News.", category: "Tech", keywords: "tech startup programming software coding yc hacker news" },
    CuratedEntry { title: "Ars Technica", feed_url: "https://feeds.arstechnica.com/arstechnica/index", site_url: "https://arstechnica.com", description: "In-depth technology news, policy, science, and hardware reviews.", category: "Tech", keywords: "tech hardware policy science reviews computers" },
    CuratedEntry { title: "The Verge", feed_url: "https://www.theverge.com/rss/index.xml", site_url: "https://www.theverge.com", description: "Consumer tech, gadgets, culture, and industry news.", category: "Tech", keywords: "gadgets smartphones apple google microsoft consumer tech" },
    CuratedEntry { title: "TechCrunch", feed_url: "https://techcrunch.com/feed/", site_url: "https://techcrunch.com", description: "Startup funding, venture capital, and emerging technology.", category: "Tech", keywords: "startups vc funding apps silicon valley" },
    CuratedEntry { title: "WIRED", feed_url: "https://www.wired.com/feed/rss", site_url: "https://www.wired.com", description: "How emerging technologies affect culture, economy, and politics.", category: "Tech", keywords: "tech culture future science security" },
    CuratedEntry { title: "The Rust Programming Language Blog", feed_url: "https://blog.rust-lang.org/feed.xml", site_url: "https://blog.rust-lang.org", description: "Official announcements and releases from the Rust team.", category: "Programming", keywords: "rust systems programming cargo compiler" },
    CuratedEntry { title: "GitHub Blog", feed_url: "https://github.blog/feed/", site_url: "https://github.blog", description: "Engineering updates, developer tools, and open source stories from GitHub.", category: "Programming", keywords: "github git open source devops coding software" },
    CuratedEntry { title: "Lobsters", feed_url: "https://lobste.rs/rss", site_url: "https://lobste.rs", description: "Computing-focused community centered around link aggregation and discussion.", category: "Programming", keywords: "programming linux systems rust go security" },
    CuratedEntry { title: "Smashing Magazine", feed_url: "https://www.smashingmagazine.com/feed/", site_url: "https://www.smashingmagazine.com", description: "Web design, CSS, UX, and front-end engineering articles.", category: "Programming", keywords: "web design css react javascript frontend ux" },

    // Cybersecurity
    CuratedEntry { title: "Krebs on Security", feed_url: "https://krebsonsecurity.com/feed/", site_url: "https://krebsonsecurity.com", description: "In-depth security news and investigation by Brian Krebs.", category: "Cybersecurity", keywords: "security cybercrime breaches malware ransomware infosec" },
    CuratedEntry { title: "BleepingComputer", feed_url: "https://www.bleepingcomputer.com/feed/", site_url: "https://www.bleepingcomputer.com", description: "Breaking cybersecurity news, zero-days, ransomware, and Windows updates.", category: "Cybersecurity", keywords: "cybersecurity ransomware cve windows patch malware" },
    CuratedEntry { title: "The Hacker News", feed_url: "https://feeds.feedburner.com/TheHackersNews", site_url: "https://thehackernews.com", description: "Daily cybersecurity news, exploits, vulnerabilities, and threat intel.", category: "Cybersecurity", keywords: "cybersecurity hacking exploits cve threat intelligence" },
    CuratedEntry { title: "Dark Reading", feed_url: "https://www.darkreading.com/rss.xml", site_url: "https://www.darkreading.com", description: "Enterprise cybersecurity news, SOC analysis, and defense strategies.", category: "Cybersecurity", keywords: "enterprise security soc cloud security zero trust" },
    CuratedEntry { title: "Schneier on Security", feed_url: "https://www.schneier.com/feed/atom/", site_url: "https://www.schneier.com", description: "Security and cryptography commentary by Bruce Schneier.", category: "Cybersecurity", keywords: "cryptography privacy policy security" },
    CuratedEntry { title: "CISA Cybersecurity Advisories", feed_url: "https://www.cisa.gov/cybersecurity-advisories/all.xml", site_url: "https://www.cisa.gov", description: "Official US CISA alerts, ICS advisories, and vulnerability bulletins.", category: "Cybersecurity", keywords: "cisa government alerts vulnerabilities cve critical infrastructure" },

    // AI & Machine Learning
    CuratedEntry { title: "MIT Technology Review", feed_url: "https://www.technologyreview.com/feed/", site_url: "https://www.technologyreview.com", description: "Authoritative coverage of AI, biotech, climate, and computing.", category: "AI & ML", keywords: "ai artificial intelligence machine learning llm research" },
    CuratedEntry { title: "Google DeepMind Blog", feed_url: "https://deepmind.google/blog/rss.xml", site_url: "https://deepmind.google/blog", description: "AI research breakthroughs from Google DeepMind.", category: "AI & ML", keywords: "ai deepmind gemini research machine learning" },
    CuratedEntry { title: "OpenAI News", feed_url: "https://openai.com/news/rss.xml", site_url: "https://openai.com/news", description: "Official announcements and research releases from OpenAI.", category: "AI & ML", keywords: "openai chatgpt gpt llm ai" },
    CuratedEntry { title: "Hugging Face Blog", feed_url: "https://huggingface.co/blog/feed.xml", site_url: "https://huggingface.co/blog", description: "Open-source AI models, transformers, and ML engineering.", category: "AI & ML", keywords: "ai open source models llm python machine learning" },
    CuratedEntry { title: "Simon Willison's Weblog", feed_url: "https://simonwillison.net/atom/everything/", site_url: "https://simonwillison.net", description: "Practical LLM engineering, SQLite, Python, and AI tooling notes.", category: "AI & ML", keywords: "llm ai sqlite python tools prompt engineering" },

    // Cloud, DevOps & Linux
    CuratedEntry { title: "Cloudflare Blog", feed_url: "https://blog.cloudflare.com/rss/", site_url: "https://blog.cloudflare.com", description: "Networking, DNS, DDoS defense, Workers, and internet infrastructure.", category: "Cloud & Infra", keywords: "cloudflare networking dns edge security internet" },
    CuratedEntry { title: "AWS News Blog", feed_url: "https://aws.amazon.com/blogs/aws/feed/", site_url: "https://aws.amazon.com/blogs/aws/", description: "Official Amazon Web Services launches and cloud architecture.", category: "Cloud & Infra", keywords: "aws amazon cloud devops serverless kubernetes" },
    CuratedEntry { title: "Microsoft Azure Blog", feed_url: "https://azure.microsoft.com/en-us/blog/feed/", site_url: "https://azure.microsoft.com/en-us/blog/", description: "Official Microsoft Azure cloud updates, AI, and enterprise infrastructure.", category: "Cloud & Infra", keywords: "azure microsoft cloud enterprise entra kubernetes" },
    CuratedEntry { title: "Kubernetes Blog", feed_url: "https://kubernetes.io/feed.xml", site_url: "https://kubernetes.io/blog/", description: "Official Kubernetes releases, security, and CNCF engineering.", category: "Cloud & Infra", keywords: "kubernetes k8s containers docker devops cloud native" },
    CuratedEntry { title: "LWN.net", feed_url: "https://lwn.net/headlines/rss", site_url: "https://lwn.net", description: "Deep technical coverage of the Linux kernel and free software.", category: "Cloud & Infra", keywords: "linux kernel open source sysadmin unix" },
    CuratedEntry { title: "OMG! Ubuntu", feed_url: "https://www.omgubuntu.co.uk/feed", site_url: "https://www.omgubuntu.co.uk", description: "Ubuntu, GNOME, and desktop Linux news and guides.", category: "Cloud & Infra", keywords: "ubuntu linux desktop open source" },

    // World News
    CuratedEntry { title: "BBC News — World", feed_url: "https://feeds.bbci.co.uk/news/world/rss.xml", site_url: "https://www.bbc.com/news/world", description: "Global breaking news and international reporting from the BBC.", category: "World News", keywords: "world news global politics breaking international" },
    CuratedEntry { title: "NPR News", feed_url: "https://feeds.npr.org/1001/rss.xml", site_url: "https://www.npr.org/sections/news/", description: "US and world news, policy, and analysis from NPR.", category: "World News", keywords: "news us world politics economy" },
    CuratedEntry { title: "Associated Press (Top News)", feed_url: "https://feedx.net/rss/ap.xml", site_url: "https://apnews.com", description: "Independent global journalism and breaking headlines.", category: "World News", keywords: "ap news headlines breaking world us" },
    CuratedEntry { title: "The Guardian — World", feed_url: "https://www.theguardian.com/world/rss", site_url: "https://www.theguardian.com/world", description: "International news, features, and commentary.", category: "World News", keywords: "world news uk europe global" },

    // Business & Finance
    CuratedEntry { title: "CNBC Top News", feed_url: "https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114", site_url: "https://www.cnbc.com", description: "Stock market, business, economy, and financial news.", category: "Finance", keywords: "finance stocks market economy investing business" },
    CuratedEntry { title: "Yahoo Finance", feed_url: "https://finance.yahoo.com/news/rssindex", site_url: "https://finance.yahoo.com", description: "Market news, earnings, personal finance, and investing.", category: "Finance", keywords: "stocks investing earnings wall street finance" },
    CuratedEntry { title: "Harvard Business Review", feed_url: "https://feeds.hbr.org/harvardbusiness", site_url: "https://hbr.org", description: "Leadership, strategy, management, and enterprise innovation.", category: "Finance", keywords: "business leadership management strategy enterprise" },

    // Science & Space
    CuratedEntry { title: "NASA Breaking News", feed_url: "https://www.nasa.gov/rss/dyn/breaking_news.rss", site_url: "https://www.nasa.gov", description: "Space exploration, missions, astronomy, and science from NASA.", category: "Science", keywords: "nasa space astronomy rockets science mars" },
    CuratedEntry { title: "ScienceDaily — Top Science", feed_url: "https://www.sciencedaily.com/rss/top/science.xml", site_url: "https://www.sciencedaily.com", description: "Latest peer-reviewed research news across physics, biology, and health.", category: "Science", keywords: "science research physics biology medicine health" },
    CuratedEntry { title: "Quanta Magazine", feed_url: "https://api.quantamagazine.org/feed/", site_url: "https://www.quantamagazine.org", description: "Illuminating developments in mathematics, physics, biology, and CS.", category: "Science", keywords: "math physics quantum biology computer science" },

    // Sports & Gaming
    CuratedEntry { title: "ESPN Top Headlines", feed_url: "https://www.espn.com/espn/rss/news", site_url: "https://www.espn.com", description: "Breaking sports news across NFL, NBA, MLB, NHL, and college sports.", category: "Sports", keywords: "sports nfl nba mlb nhl football basketball baseball" },
    CuratedEntry { title: "IGN All News", feed_url: "https://feeds.feedburner.com/ign/all", site_url: "https://www.ign.com", description: "Video games, reviews, movies, and entertainment news.", category: "Gaming", keywords: "gaming video games pc xbox playstation nintendo reviews" },
    CuratedEntry { title: "PC Gamer", feed_url: "https://www.pcgamer.com/rss/", site_url: "https://www.pcgamer.com", description: "PC gaming news, hardware, mods, and reviews.", category: "Gaming", keywords: "pc gaming steam hardware esports" },
    CuratedEntry { title: "Kotaku", feed_url: "https://kotaku.com/rss", site_url: "https://kotaku.com", description: "Gaming culture, industry reporting, and reviews.", category: "Gaming", keywords: "gaming culture indie games reviews" },
];

pub fn list_categories() -> Vec<&'static str> {
    vec![
        "All",
        "Tech",
        "Programming",
        "Cybersecurity",
        "AI & ML",
        "Cloud & Infra",
        "World News",
        "Finance",
        "Science",
        "Sports",
        "Gaming",
    ]
}

pub async fn search_feeds(
    client: &Client,
    query: &str,
    category: Option<&str>,
    subscribed_urls: &HashSet<String>,
) -> Vec<FeedSearchResult> {
    let q = query.trim();
    let q_lower = q.to_lowercase();
    let cat_filter = category
        .map(|c| c.trim())
        .filter(|c| !c.is_empty() && !c.eq_ignore_ascii_case("all"));

    let mut results: Vec<FeedSearchResult> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // 1. Instant local curated directory match
    for entry in CURATED_FEEDS {
        if let Some(cf) = cat_filter {
            if !entry.category.eq_ignore_ascii_case(cf) {
                continue;
            }
        }
        if !q_lower.is_empty() {
            let haystack = format!(
                "{} {} {} {}",
                entry.title, entry.description, entry.category, entry.keywords
            )
            .to_lowercase();
            let all_terms_match = q_lower
                .split_whitespace()
                .all(|term| haystack.contains(term));
            if !all_terms_match {
                continue;
            }
        }

        let norm = entry.feed_url.to_lowercase();
        if seen.insert(norm.clone()) {
            let host = url::Url::parse(entry.site_url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()));
            results.push(FeedSearchResult {
                title: entry.title.to_string(),
                feed_url: entry.feed_url.to_string(),
                site_url: entry.site_url.to_string(),
                description: entry.description.to_string(),
                category: entry.category.to_string(),
                icon_url: host.map(|h| format!("https://www.google.com/s2/favicons?domain={}&sz=64", h)),
                subscribers: None,
                source: "directory".to_string(),
                already_subscribed: subscribed_urls.contains(&norm),
            });
        }
    }

    // 2. Live Topic Search via Feedly's public Cloud Feed Search API when user enters a query
    if !q.is_empty() {
        let encoded: String = form_urlencoded::byte_serialize(q.as_bytes()).collect();
        let search_url = format!(
            "https://cloud.feedly.com/v3/search/feeds?query={}&count=15",
            encoded
        );

        if let Ok(resp) = client
            .get(&search_url)
            .timeout(std::time::Duration::from_secs(6))
            .send()
            .await
        {
            if let Ok(val) = resp.json::<Value>().await {
                if let Some(arr) = val.get("results").and_then(|v| v.as_array()) {
                    for item in arr {
                        let raw_fid = item.get("feedId").and_then(|v| v.as_str()).unwrap_or("");
                        let feed_url = raw_fid.strip_prefix("feed/").unwrap_or(raw_fid).trim();
                        if !feed_url.starts_with("http://") && !feed_url.starts_with("https://") {
                            continue;
                        }
                        let norm = feed_url.to_lowercase();
                        if !seen.insert(norm.clone()) {
                            continue;
                        }
                        let title = item
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("RSS Feed")
                            .to_string();
                        let site_url = item
                            .get("website")
                            .and_then(|v| v.as_str())
                            .unwrap_or(feed_url)
                            .to_string();
                        let description = item
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .chars()
                            .take(220)
                            .collect();
                        let icon_url = item
                            .get("iconUrl")
                            .or_else(|| item.get("visualUrl"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let subscribers = item.get("subscribers").and_then(|v| v.as_i64());
                        let topic = item
                            .get("topics")
                            .and_then(|v| v.as_array())
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("Discovered")
                            .to_string();

                        results.push(FeedSearchResult {
                            title,
                            feed_url: feed_url.to_string(),
                            site_url,
                            description,
                            category: topic,
                            icon_url,
                            subscribers,
                            source: "live".to_string(),
                            already_subscribed: subscribed_urls.contains(&norm),
                        });
                    }
                }
            }
        }

        // 3. Always include dynamic Topic Feeds (Google News & Reddit) so ANY niche topic works!
        let gnews_url = format!(
            "https://news.google.com/rss/search?q={}&hl=en-US&gl=US&ceid=US:en",
            encoded
        );
        let gnews_norm = gnews_url.to_lowercase();
        if seen.insert(gnews_norm.clone()) {
            results.push(FeedSearchResult {
                title: format!("Google News: \"{}\"", q),
                feed_url: gnews_url,
                site_url: format!("https://news.google.com/search?q={}", encoded),
                description: format!("Live news headlines from across the web matching \"{}\".", q),
                category: "Topic Feed".to_string(),
                icon_url: Some("https://www.google.com/s2/favicons?domain=news.google.com&sz=64".to_string()),
                subscribers: None,
                source: "topic_feed".to_string(),
                already_subscribed: subscribed_urls.contains(&gnews_norm),
            });
        }

        let reddit_url = format!("https://www.reddit.com/search.rss?q={}&sort=new", encoded);
        let reddit_norm = reddit_url.to_lowercase();
        if seen.insert(reddit_norm.clone()) {
            results.push(FeedSearchResult {
                title: format!("Reddit Discussions: \"{}\"", q),
                feed_url: reddit_url,
                site_url: format!("https://www.reddit.com/search/?q={}", encoded),
                description: format!("Real-time Reddit posts and community discussions about \"{}\".", q),
                category: "Topic Feed".to_string(),
                icon_url: Some("https://www.google.com/s2/favicons?domain=reddit.com&sz=64".to_string()),
                subscribers: None,
                source: "topic_feed".to_string(),
                already_subscribed: subscribed_urls.contains(&reddit_norm),
            });
        }
    }

    results
}
