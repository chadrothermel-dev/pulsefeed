package com.pulsefeed.app

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.net.URLEncoder
import java.util.concurrent.TimeUnit

@Serializable
data class EditionResp(
    val edition: String = "personal",
    @SerialName("is_personal") val isPersonal: Boolean = true
)

@Serializable
data class LoginReq(val email: String, val password: String)

@Serializable
data class LoginResp(val token: String)

@Serializable
data class ArticleItem(
    val id: String,
    @SerialName("feed_title") val feedTitle: String = "",
    val title: String = "",
    val url: String = "",
    val author: String? = null,
    val summary: String = "",
    @SerialName("published_at") val publishedAt: String = "",
    @SerialName("reading_time_mins") val readingTimeMins: Int = 1,
    @SerialName("priority_score") val priorityScore: Int = 50,
    @SerialName("is_read") val isRead: Boolean = false,
    @SerialName("is_read_later") val isReadLater: Boolean = false,
    @SerialName("ai_summary") val aiSummary: String? = null,
    @SerialName("ai_sentiment") val aiSentiment: String? = null
)

@Serializable
data class ArticlesResp(val articles: List<ArticleItem> = emptyList())

@Serializable
data class ArticleDetailResp(
    val id: String,
    @SerialName("feed_title") val feedTitle: String = "",
    val title: String = "",
    val url: String = "",
    val author: String? = null,
    val summary: String = "",
    @SerialName("content_text") val contentText: String = "",
    @SerialName("published_at") val publishedAt: String = "",
    @SerialName("reading_time_mins") val readingTimeMins: Int = 1,
    @SerialName("priority_score") val priorityScore: Int = 50,
    @SerialName("is_read") val isRead: Boolean = false,
    @SerialName("is_read_later") val isReadLater: Boolean = false,
    @SerialName("ai_summary") val aiSummary: String? = null,
    @SerialName("ai_sentiment") val aiSentiment: String? = null
)

@Serializable
data class AiSummaryResp(
    @SerialName("ai_summary") val aiSummary: String = "",
    @SerialName("ai_sentiment") val aiSentiment: String = "Neutral"
)

@Serializable
data class FeedSearchItem(
    val title: String = "",
    @SerialName("feed_url") val feedUrl: String = "",
    val description: String = "",
    val category: String = "",
    val source: String = "",
    @SerialName("already_subscribed") val alreadySubscribed: Boolean = false
)

@Serializable
data class FeedSearchResp(
    val categories: List<String> = emptyList(),
    val results: List<FeedSearchItem> = emptyList()
)

@Serializable
data class CreateFeedPayload(
    @SerialName("feed_url") val feedUrl: String,
    val title: String? = null
)

@Serializable
data class StatePatch(
    @SerialName("is_read") val isRead: Boolean? = null,
    @SerialName("is_read_later") val isReadLater: Boolean? = null
)

class PulseFeedApi(
    private val baseUrl: String,
    private val token: String? = null
) {
    private val json = Json { ignoreUnknownKeys = true }
    private val mediaJson = "application/json; charset=utf-8".toMediaType()
    private val client = OkHttpClient.Builder()
        .connectTimeout(12, TimeUnit.SECONDS)
        .readTimeout(25, TimeUnit.SECONDS)
        .build()

    private fun cleanBase(): String = baseUrl.trim().trimEnd('/')

    private fun reqBuilder(path: String): Request.Builder {
        val b = Request.Builder().url("${cleanBase()}/api$path")
        if (!token.isNullOrBlank()) {
            b.header("Authorization", "Bearer $token")
        }
        return b
    }

    suspend fun getEdition(): EditionResp = withContext(Dispatchers.IO) {
        val req = reqBuilder("/edition").get().build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("HTTP ${resp.code}")
            json.decodeFromString(resp.body?.string() ?: "{}")
        }
    }

    suspend fun login(email: String, password: String): String = withContext(Dispatchers.IO) {
        val body = json.encodeToString(LoginReq.serializer(), LoginReq(email, password))
            .toRequestBody(mediaJson)
        val req = reqBuilder("/auth/login").post(body).build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("Login failed (HTTP ${resp.code})")
            json.decodeFromString<LoginResp>(resp.body?.string() ?: "{}").token
        }
    }

    suspend fun getArticles(view: String, query: String = ""): List<ArticleItem> = withContext(Dispatchers.IO) {
        val qPart = if (query.isNotBlank()) "&q=${URLEncoder.encode(query, "UTF-8")}" else ""
        val req = reqBuilder("/articles?view=$view&limit=50$qPart").get().build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("HTTP ${resp.code}")
            json.decodeFromString<ArticlesResp>(resp.body?.string() ?: "{}").articles
        }
    }

    suspend fun getArticle(id: String): ArticleDetailResp = withContext(Dispatchers.IO) {
        val req = reqBuilder("/articles/$id").get().build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("HTTP ${resp.code}")
            json.decodeFromString(resp.body?.string() ?: "{}")
        }
    }

    suspend fun updateArticleState(id: String, isRead: Boolean? = null, isReadLater: Boolean? = null) =
        withContext(Dispatchers.IO) {
            val body = json.encodeToString(StatePatch.serializer(), StatePatch(isRead, isReadLater))
                .toRequestBody(mediaJson)
            val req = reqBuilder("/articles/$id/state").patch(body).build()
            client.newCall(req).execute().close()
        }

    suspend fun generateAiSummary(id: String): AiSummaryResp = withContext(Dispatchers.IO) {
        val req = reqBuilder("/articles/$id/ai-summary").post("{}".toRequestBody(mediaJson)).build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("HTTP ${resp.code}")
            json.decodeFromString(resp.body?.string() ?: "{}")
        }
    }

    suspend fun refreshAll() = withContext(Dispatchers.IO) {
        val req = reqBuilder("/feeds/refresh-all").post("{}".toRequestBody(mediaJson)).build()
        client.newCall(req).execute().close()
    }

    suspend fun searchFeeds(query: String, category: String): FeedSearchResp = withContext(Dispatchers.IO) {
        val qEnc = URLEncoder.encode(query, "UTF-8")
        val cEnc = URLEncoder.encode(category, "UTF-8")
        val req = reqBuilder("/feeds/search?q=$qEnc&category=$cEnc").get().build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) error("HTTP ${resp.code}")
            json.decodeFromString(resp.body?.string() ?: "{}")
        }
    }

    suspend fun subscribeFeed(feedUrl: String, title: String) = withContext(Dispatchers.IO) {
        val body = json.encodeToString(CreateFeedPayload.serializer(), CreateFeedPayload(feedUrl, title))
            .toRequestBody(mediaJson)
        val req = reqBuilder("/feeds").post(body).build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful && resp.code != 409) error("Subscribe failed (HTTP ${resp.code})")
        }
    }
}
