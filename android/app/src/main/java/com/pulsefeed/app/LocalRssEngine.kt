package com.pulsefeed.app

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import android.util.Xml
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.OkHttpClient
import okhttp3.Request
import org.xmlpull.v1.XmlPullParser
import java.io.StringReader
import java.net.URLEncoder
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import java.util.UUID
import java.util.concurrent.TimeUnit

data class LocalFeed(
    val id: String,
    val title: String,
    val feedUrl: String,
    val category: String
)

private data class CuratedLocalFeed(
    val title: String,
    val feedUrl: String,
    val description: String,
    val category: String,
    val keywords: String
)

private val LOCAL_DIRECTORY = listOf(
    CuratedLocalFeed("Hacker News", "https://hnrss.org/frontpage", "Top tech, startup, and programming links.", "Tech", "tech startup coding software hacker news"),
    CuratedLocalFeed("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index", "In-depth technology news, science, and reviews.", "Tech", "tech hardware science computers reviews"),
    CuratedLocalFeed("The Verge", "https://www.theverge.com/rss/index.xml", "Consumer tech, gadgets, and industry news.", "Tech", "gadgets apple google microsoft consumer tech"),
    CuratedLocalFeed("TechCrunch", "https://techcrunch.com/feed/", "Startup funding, VC, and emerging technology.", "Tech", "startups vc apps silicon valley"),
    CuratedLocalFeed("Krebs on Security", "https://krebsonsecurity.com/feed/", "In-depth security news and cybercrime investigations.", "Cybersecurity", "security cybercrime breaches malware ransomware"),
    CuratedLocalFeed("BleepingComputer", "https://www.bleepingcomputer.com/feed/", "Breaking cybersecurity news, zero-days, and patches.", "Cybersecurity", "cybersecurity ransomware cve windows malware"),
    CuratedLocalFeed("The Hacker News", "https://feeds.feedburner.com/TheHackersNews", "Daily cybersecurity news, exploits, and threat intel.", "Cybersecurity", "cybersecurity hacking exploits cve"),
    CuratedLocalFeed("MIT Technology Review", "https://www.technologyreview.com/feed/", "Authoritative coverage of AI, biotech, and computing.", "AI & ML", "ai artificial intelligence machine learning llm"),
    CuratedLocalFeed("Hugging Face Blog", "https://huggingface.co/blog/feed.xml", "Open-source AI models and ML engineering.", "AI & ML", "ai open source models llm python"),
    CuratedLocalFeed("Cloudflare Blog", "https://blog.cloudflare.com/rss/", "Networking, DNS, security, and internet infrastructure.", "Cloud & Infra", "cloudflare networking dns cloud security"),
    CuratedLocalFeed("Kubernetes Blog", "https://kubernetes.io/feed.xml", "Official Kubernetes releases and cloud-native engineering.", "Cloud & Infra", "kubernetes k8s docker devops cloud"),
    CuratedLocalFeed("BBC World News", "https://feeds.bbci.co.uk/news/world/rss.xml", "Global breaking news and international reporting.", "World News", "world news global politics breaking"),
    CuratedLocalFeed("NPR News", "https://feeds.npr.org/1001/rss.xml", "US and world news, policy, and analysis.", "World News", "news us world politics economy"),
    CuratedLocalFeed("CNBC Top News", "https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114", "Stock market, business, and financial news.", "Finance", "finance stocks market economy investing"),
    CuratedLocalFeed("NASA Breaking News", "https://www.nasa.gov/rss/dyn/breaking_news.rss", "Space exploration, missions, and astronomy.", "Science", "nasa space astronomy rockets science"),
    CuratedLocalFeed("ScienceDaily", "https://www.sciencedaily.com/rss/top/science.xml", "Latest peer-reviewed research across science & health.", "Science", "science research physics biology health"),
    CuratedLocalFeed("ESPN Top Headlines", "https://www.espn.com/espn/rss/news", "Breaking sports news across NFL, NBA, MLB, NHL.", "Sports", "sports nfl nba mlb nhl football basketball"),
    CuratedLocalFeed("IGN All News", "https://feeds.feedburner.com/ign/all", "Video games, reviews, and entertainment news.", "Gaming", "gaming video games pc xbox playstation nintendo"),
    CuratedLocalFeed("PC Gamer", "https://www.pcgamer.com/rss/", "PC gaming news, hardware, and reviews.", "Gaming", "pc gaming steam hardware esports")
)

class LocalDbHelper(context: Context) : SQLiteOpenHelper(context, "pulsefeed_local.db", null, 1) {
    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL(
            """
            CREATE TABLE feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                feed_url TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL DEFAULT 'General',
                created_at TEXT NOT NULL
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE articles (
                id TEXT PRIMARY KEY,
                feed_id TEXT NOT NULL,
                feed_title TEXT NOT NULL,
                guid TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                content_text TEXT NOT NULL DEFAULT '',
                published_at TEXT NOT NULL,
                reading_time_mins INTEGER NOT NULL DEFAULT 1,
                priority_score INTEGER NOT NULL DEFAULT 50,
                is_muted INTEGER NOT NULL DEFAULT 0,
                is_read INTEGER NOT NULL DEFAULT 0,
                is_read_later INTEGER NOT NULL DEFAULT 0,
                ai_summary TEXT,
                ai_sentiment TEXT,
                UNIQUE(feed_id, guid)
            )
            """.trimIndent()
        )
        db.execSQL("CREATE INDEX idx_local_art_pub ON articles(published_at DESC)")

        // Seed 6 great starter feeds so the phone app is immediately alive!
        val now = isoNow()
        val starters = listOf(
            Triple("Hacker News", "https://hnrss.org/frontpage", "Tech"),
            Triple("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index", "Tech"),
            Triple("Krebs on Security", "https://krebsonsecurity.com/feed/", "Cybersecurity"),
            Triple("BleepingComputer", "https://www.bleepingcomputer.com/feed/", "Cybersecurity"),
            Triple("MIT Technology Review", "https://www.technologyreview.com/feed/", "AI & ML"),
            Triple("NASA Breaking News", "https://www.nasa.gov/rss/dyn/breaking_news.rss", "Science")
        )
        for ((title, url, cat) in starters) {
            val cv = ContentValues().apply {
                put("id", UUID.randomUUID().toString())
                put("title", title)
                put("feed_url", url)
                put("category", cat)
                put("created_at", now)
            }
            db.insert("feeds", null, cv)
        }
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {}
}

private fun isoNow(): String {
    val fmt = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss'Z'", Locale.US)
    fmt.timeZone = TimeZone.getTimeZone("UTC")
    return fmt.format(Date())
}

private fun stripHtml(raw: String): String {
    return raw
        .replace(Regex("<script[\\s\\S]*?</script>", RegexOption.IGNORE_CASE), " ")
        .replace(Regex("<style[\\s\\S]*?</style>", RegexOption.IGNORE_CASE), " ")
        .replace(Regex("<[^>]+>"), " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace(Regex("\\s+"), " ")
        .trim()
}

private fun computeLocalLeoScore(title: String, body: String): Pair<Int, Boolean> {
    val text = "$title $body".lowercase(Locale.US)
    val muteTerms = listOf("sponsored", "promo code", "affiliate deal", "advertisement")
    if (muteTerms.any { title.lowercase(Locale.US).contains(it) }) {
        return 10 to true
    }
    var score = 50
    val highTerms = listOf("cve-", "zero-day", "ransomware", "vulnerability", "exploit", "breach", "critical")
    val aiTerms = listOf("llm", "gemini", "openai", "claude", "anthropic", "ai ", "benchmark", "breakthrough", "rust")
    if (highTerms.any { text.contains(it) }) score += 30
    if (aiTerms.any { text.contains(it) }) score += 20
    return score.coerceIn(0, 100) to false
}

class LocalPulseFeedEngine(context: Context) {
    private val dbHelper = LocalDbHelper(context.applicationContext)
    private val json = Json { ignoreUnknownKeys = true }
    private val client = OkHttpClient.Builder()
        .connectTimeout(12, TimeUnit.SECONDS)
        .readTimeout(20, TimeUnit.SECONDS)
        .build()

    fun listFeeds(): List<LocalFeed> {
        val list = mutableListOf<LocalFeed>()
        dbHelper.readableDatabase.rawQuery(
            "SELECT id, title, feed_url, category FROM feeds ORDER BY title ASC",
            null
        ).use { c ->
            while (c.moveToNext()) {
                list.add(LocalFeed(c.getString(0), c.getString(1), c.getString(2), c.getString(3)))
            }
        }
        return list
    }

    fun deleteFeed(feedId: String) {
        val db = dbHelper.writableDatabase
        db.delete("articles", "feed_id = ?", arrayOf(feedId))
        db.delete("feeds", "id = ?", arrayOf(feedId))
    }

    suspend fun subscribeFeed(feedUrl: String, title: String, category: String = "General") =
        withContext(Dispatchers.IO) {
            val id = UUID.randomUUID().toString()
            val cv = ContentValues().apply {
                put("id", id)
                put("title", title.ifBlank { feedUrl })
                put("feed_url", feedUrl.trim())
                put("category", category)
                put("created_at", isoNow())
            }
            val inserted = dbHelper.writableDatabase.insertWithOnConflict(
                "feeds", null, cv, SQLiteDatabase.CONFLICT_IGNORE
            )
            if (inserted != -1L) {
                fetchSingleFeed(LocalFeed(id, title, feedUrl.trim(), category))
            }
        }

    suspend fun refreshAll(): Int = coroutineScope {
        val feeds = listFeeds()
        feeds.map { feed ->
            async(Dispatchers.IO) {
                runCatching { fetchSingleFeed(feed) }.getOrDefault(0)
            }
        }.awaitAll().sum()
    }

    private fun fetchSingleFeed(feed: LocalFeed): Int {
        val req = Request.Builder()
            .url(feed.feedUrl)
            .header("User-Agent", "PulseFeed-Android/1.3")
            .get()
            .build()
        val xml = client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) return 0
            resp.body?.string() ?: return 0
        }

        val parser = Xml.newPullParser()
        parser.setFeature(XmlPullParser.FEATURE_PROCESS_NAMESPACES, false)
        parser.setInput(StringReader(xml))

        var event = parser.eventType
        var inEntry = false
        var title = ""
        var link = ""
        var guid = ""
        var desc = ""
        var pubDate = ""
        var added = 0
        val db = dbHelper.writableDatabase

        while (event != XmlPullParser.END_DOCUMENT && added < 35) {
            when (event) {
                XmlPullParser.START_TAG -> {
                    val tag = parser.name.lowercase(Locale.US)
                    if (tag == "item" || tag == "entry") {
                        inEntry = true
                        title = ""; link = ""; guid = ""; desc = ""; pubDate = ""
                    } else if (inEntry) {
                        when (tag) {
                            "title" -> title = runCatching { parser.nextText() }.getOrDefault("")
                            "link" -> {
                                val href = parser.getAttributeValue(null, "href")
                                link = if (!href.isNullOrBlank()) href else runCatching { parser.nextText() }.getOrDefault("")
                            }
                            "guid", "id" -> guid = runCatching { parser.nextText() }.getOrDefault("")
                            "description", "summary", "content", "content:encoded" -> {
                                val txt = runCatching { parser.nextText() }.getOrDefault("")
                                if (txt.length > desc.length) desc = txt
                            }
                            "pubdate", "published", "updated" -> {
                                if (pubDate.isBlank()) pubDate = runCatching { parser.nextText() }.getOrDefault("")
                            }
                        }
                    }
                }
                XmlPullParser.END_TAG -> {
                    val tag = parser.name.lowercase(Locale.US)
                    if ((tag == "item" || tag == "entry") && inEntry) {
                        inEntry = false
                        val cleanTitle = stripHtml(title).ifBlank { "Untitled" }
                        val cleanBody = stripHtml(desc)
                        val cleanUrl = link.trim().ifBlank { feed.feedUrl }
                        val cleanGuid = guid.trim().ifBlank { cleanUrl }
                        val (score, muted) = computeLocalLeoScore(cleanTitle, cleanBody)
                        val words = cleanBody.split(Regex("\\s+")).count { it.isNotBlank() }
                        val mins = ((words + 219) / 220).coerceAtLeast(1)

                        val cv = ContentValues().apply {
                            put("id", UUID.randomUUID().toString())
                            put("feed_id", feed.id)
                            put("feed_title", feed.title)
                            put("guid", cleanGuid)
                            put("title", cleanTitle)
                            put("url", cleanUrl)
                            put("summary", cleanBody.take(260))
                            put("content_text", cleanBody)
                            put("published_at", isoNow())
                            put("reading_time_mins", mins)
                            put("priority_score", score)
                            put("is_muted", if (muted) 1 else 0)
                        }
                        if (db.insertWithOnConflict("articles", null, cv, SQLiteDatabase.CONFLICT_IGNORE) != -1L) {
                            added++
                        }
                    }
                }
            }
            event = parser.next()
        }
        return added
    }

    fun getArticles(view: String, query: String = ""): List<ArticleItem> {
        val wheres = mutableListOf("is_muted = 0")
        val args = mutableListOf<String>()

        when (view) {
            "leo_priority" -> wheres.add("priority_score >= 65")
            "read_later" -> wheres.add("is_read_later = 1")
        }
        if (query.isNotBlank()) {
            wheres.add("(lower(title) LIKE ? OR lower(content_text) LIKE ? OR lower(feed_title) LIKE ?)")
            val pat = "%${query.trim().lowercase(Locale.US)}%"
            args.add(pat); args.add(pat); args.add(pat)
        }

        val order = if (view == "leo_priority") "priority_score DESC, published_at DESC" else "published_at DESC"
        val sql = "SELECT id, feed_title, title, url, summary, published_at, reading_time_mins, priority_score, is_read, is_read_later, ai_summary, ai_sentiment FROM articles WHERE ${wheres.joinToString(" AND ")} ORDER BY $order LIMIT 60"

        val out = mutableListOf<ArticleItem>()
        dbHelper.readableDatabase.rawQuery(sql, args.toTypedArray()).use { c ->
            while (c.moveToNext()) {
                out.add(
                    ArticleItem(
                        id = c.getString(0),
                        feedTitle = c.getString(1),
                        title = c.getString(2),
                        url = c.getString(3),
                        summary = c.getString(4),
                        publishedAt = c.getString(5),
                        readingTimeMins = c.getInt(6),
                        priorityScore = c.getInt(7),
                        isRead = c.getInt(8) != 0,
                        isReadLater = c.getInt(9) != 0,
                        aiSummary = c.getString(10),
                        aiSentiment = c.getString(11)
                    )
                )
            }
        }
        return out
    }

    fun getArticle(id: String): ArticleDetailResp? {
        dbHelper.readableDatabase.rawQuery(
            "SELECT id, feed_title, title, url, summary, content_text, published_at, reading_time_mins, priority_score, is_read, is_read_later, ai_summary, ai_sentiment FROM articles WHERE id = ?",
            arrayOf(id)
        ).use { c ->
            if (!c.moveToFirst()) return null
            return ArticleDetailResp(
                id = c.getString(0),
                feedTitle = c.getString(1),
                title = c.getString(2),
                url = c.getString(3),
                summary = c.getString(4),
                contentText = c.getString(5),
                publishedAt = c.getString(6),
                readingTimeMins = c.getInt(7),
                priorityScore = c.getInt(8),
                isRead = c.getInt(9) != 0,
                isReadLater = c.getInt(10) != 0,
                aiSummary = c.getString(11),
                aiSentiment = c.getString(12)
            )
        }
    }

    fun updateArticleState(id: String, isRead: Boolean? = null, isReadLater: Boolean? = null) {
        val cv = ContentValues()
        if (isRead != null) cv.put("is_read", if (isRead) 1 else 0)
        if (isReadLater != null) cv.put("is_read_later", if (isReadLater) 1 else 0)
        if (cv.size() > 0) {
            dbHelper.writableDatabase.update("articles", cv, "id = ?", arrayOf(id))
        }
    }

    fun generateLocalSummary(id: String): AiSummaryResp {
        val art = getArticle(id) ?: return AiSummaryResp("No content available.", "Neutral")
        val sentences = art.contentText
            .split(Regex("[.!?]+"))
            .map { it.trim() }
            .filter { it.length in 30..280 }
        val top = if (sentences.isEmpty()) listOf(art.title) else sentences.take(3)
        val summary = top.joinToString("\n") { "• $it." }
        val lower = "${art.title} ${art.contentText}".lowercase(Locale.US)
        val sentiment = when {
            listOf("zero-day", "breach", "ransomware", "critical", "exploit", "cve-").any { lower.contains(it) } -> "Urgent"
            listOf("launch", "release", "breakthrough", "milestone").any { lower.contains(it) } -> "Positive"
            else -> "Neutral"
        }
        val cv = ContentValues().apply {
            put("ai_summary", summary)
            put("ai_sentiment", sentiment)
        }
        dbHelper.writableDatabase.update("articles", cv, "id = ?", arrayOf(id))
        return AiSummaryResp(summary, sentiment)
    }

    suspend fun searchFeeds(query: String, category: String): FeedSearchResp = withContext(Dispatchers.IO) {
        val subscribed = listFeeds().map { it.feedUrl.lowercase(Locale.US) }.toSet()
        val q = query.trim()
        val qLower = q.lowercase(Locale.US)
        val seen = mutableSetOf<String>()
        val results = mutableListOf<FeedSearchItem>()

        // 1. Local curated directory
        for (item in LOCAL_DIRECTORY) {
            if (category != "All" && !item.category.equals(category, ignoreCase = true)) continue
            if (qLower.isNotEmpty()) {
                val hay = "${item.title} ${item.description} ${item.category} ${item.keywords}".lowercase(Locale.US)
                if (!qLower.split(" ").all { hay.contains(it) }) continue
            }
            val norm = item.feedUrl.lowercase(Locale.US)
            if (seen.add(norm)) {
                results.add(
                    FeedSearchItem(
                        title = item.title,
                        feedUrl = item.feedUrl,
                        description = item.description,
                        category = item.category,
                        source = "directory",
                        alreadySubscribed = subscribed.contains(norm)
                    )
                )
            }
        }

        // 2. Live Feedly Search API + Google News / Reddit topic RSS when user types a query
        if (q.isNotEmpty()) {
            val enc = URLEncoder.encode(q, "UTF-8")
            runCatching {
                val req = Request.Builder()
                    .url("https://cloud.feedly.com/v3/search/feeds?query=$enc&count=12")
                    .get()
                    .build()
                client.newCall(req).execute().use { resp ->
                    if (resp.isSuccessful) {
                        val root = json.parseToJsonElement(resp.body?.string() ?: "{}").jsonObject
                        val arr = root["results"]?.jsonArray ?: emptyList()
                        for (el in arr) {
                            val obj = el.jsonObject
                            val rawId = obj["feedId"]?.jsonPrimitive?.content ?: continue
                            val url = rawId.removePrefix("feed/").trim()
                            if (!url.startsWith("http")) continue
                            val norm = url.lowercase(Locale.US)
                            if (!seen.add(norm)) continue
                            results.add(
                                FeedSearchItem(
                                    title = obj["title"]?.jsonPrimitive?.content ?: "RSS Feed",
                                    feedUrl = url,
                                    description = (obj["description"]?.jsonPrimitive?.content ?: "").take(200),
                                    category = "Live Search",
                                    source = "live",
                                    alreadySubscribed = subscribed.contains(norm)
                                )
                            )
                        }
                    }
                }
            }

            val gnews = "https://news.google.com/rss/search?q=$enc&hl=en-US&gl=US&ceid=US:en"
            if (seen.add(gnews.lowercase(Locale.US))) {
                results.add(
                    FeedSearchItem(
                        title = "Google News: \"$q\"",
                        feedUrl = gnews,
                        description = "Live headlines from across the web matching \"$q\".",
                        category = "Topic Feed",
                        source = "topic_feed",
                        alreadySubscribed = subscribed.contains(gnews.lowercase(Locale.US))
                    )
                )
            }
            val reddit = "https://www.reddit.com/search.rss?q=$enc&sort=new"
            if (seen.add(reddit.lowercase(Locale.US))) {
                results.add(
                    FeedSearchItem(
                        title = "Reddit Discussions: \"$q\"",
                        feedUrl = reddit,
                        description = "Real-time community posts about \"$q\".",
                        category = "Topic Feed",
                        source = "topic_feed",
                        alreadySubscribed = subscribed.contains(reddit.lowercase(Locale.US))
                    )
                )
            }
        }

        FeedSearchResp(
            categories = listOf("All", "Tech", "Cybersecurity", "AI & ML", "Cloud & Infra", "World News", "Finance", "Science", "Sports", "Gaming"),
            results = results
        )
    }
}
