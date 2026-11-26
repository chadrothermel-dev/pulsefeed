package com.pulsefeed.app

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

private val Emerald = Color(0xFF10B981)
private val EmeraldDark = Color(0xFF059669)
private val Violet = Color(0xFF8B5CF6)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = Emerald,
                    secondary = Violet,
                    background = Color(0xFF020617),
                    surface = Color(0xFF0F172A),
                    surfaceVariant = Color(0xFF1E293B),
                    onPrimary = Color.White,
                    onBackground = Color(0xFFF8FAFC),
                    onSurface = Color(0xFFF8FAFC)
                )
            ) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    PulseFeedRoot()
                }
            }
        }
    }
}

@Composable
fun PulseFeedRoot() {
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("pulsefeed_prefs", Context.MODE_PRIVATE) }
    var serverUrl by remember { mutableStateOf(prefs.getString("server_url", "") ?: "") }
    var token by remember { mutableStateOf(prefs.getString("jwt_token", "") ?: "") }
    var edition by remember { mutableStateOf(prefs.getString("edition", "personal") ?: "personal") }
    var connected by remember { mutableStateOf(serverUrl.isNotBlank()) }

    if (!connected) {
        ConnectScreen(
            initialUrl = serverUrl.ifBlank { "http://10.0.2.2:8080" },
            onConnected = { url, jwt, ed ->
                serverUrl = url
                token = jwt
                edition = ed
                prefs.edit()
                    .putString("server_url", url)
                    .putString("jwt_token", jwt)
                    .putString("edition", ed)
                    .apply()
                connected = true
            }
        )
    } else {
        MainWorkspace(
            api = remember(serverUrl, token) { PulseFeedApi(serverUrl, token.ifBlank { null }) },
            edition = edition,
            serverUrl = serverUrl,
            onDisconnect = {
                prefs.edit().remove("jwt_token").apply()
                connected = false
            }
        )
    }
}

@Composable
fun ConnectScreen(
    initialUrl: String,
    onConnected: (String, String, String) -> Unit
) {
    val scope = rememberCoroutineScope()
    var url by remember { mutableStateOf(initialUrl) }
    var needsLogin by remember { mutableStateOf(false) }
    var email by remember { mutableStateOf("admin@pulsefeed.local") }
    var password by remember { mutableStateOf("admin123") }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Surface(
            color = EmeraldDark,
            shape = RoundedCornerShape(18.dp),
            modifier = Modifier.size(64.dp)
        ) {
            Box(contentAlignment = Alignment.Center) {
                Icon(Icons.Default.RssFeed, contentDescription = null, tint = Color.White, modifier = Modifier.size(34.dp))
            }
        }
        Spacer(Modifier.height(16.dp))
        Text("PulseFeed", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Text(
            "Connect to your local Personal PC or hosted Enterprise server",
            style = MaterialTheme.typography.bodySmall,
            color = Color(0xFF94A3B8)
        )
        Spacer(Modifier.height(24.dp))

        OutlinedTextField(
            value = url,
            onValueChange = { url = it },
            label = { Text("PulseFeed Server URL") },
            placeholder = { Text("https://your-app.onrender.com or http://192.168.1.50:8080") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth()
        )

        if (needsLogin) {
            Spacer(Modifier.height(12.dp))
            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text("Work Email") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(10.dp))
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text("Password") },
                visualTransformation = PasswordVisualTransformation(),
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
        }

        if (error != null) {
            Spacer(Modifier.height(12.dp))
            Text(error!!, color = Color(0xFFF87171), style = MaterialTheme.typography.bodySmall)
        }

        Spacer(Modifier.height(20.dp))
        Button(
            onClick = {
                scope.launch {
                    loading = true
                    error = null
                    try {
                        val api = PulseFeedApi(url)
                        val ed = api.getEdition()
                        if (ed.isPersonal) {
                            onConnected(url, "", "personal")
                        } else if (!needsLogin) {
                            needsLogin = true
                        } else {
                            val jwt = api.login(email, password)
                            onConnected(url, jwt, "enterprise")
                        }
                    } catch (e: Exception) {
                        error = e.message ?: "Could not reach server"
                    } finally {
                        loading = false
                    }
                }
            },
            enabled = !loading && url.isNotBlank(),
            modifier = Modifier.fillMaxWidth().height(50.dp),
            shape = RoundedCornerShape(14.dp)
        ) {
            Text(if (loading) "Connecting..." else if (needsLogin) "Sign In" else "Connect")
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainWorkspace(
    api: PulseFeedApi,
    edition: String,
    serverUrl: String,
    onDisconnect: () -> Unit
) {
    val scope = rememberCoroutineScope()
    var activeTab by remember { mutableIntStateOf(0) } // 0=Today, 1=Leo, 2=Later, 3=Discover
    var searchQuery by remember { mutableStateOf("") }
    var articles by remember { mutableStateOf<List<ArticleItem>>(emptyList()) }
    var loading by remember { mutableStateOf(false) }
    var selectedArticleId by remember { mutableStateOf<String?>(null) }

    val viewName = when (activeTab) {
        0 -> "today"
        1 -> "leo_priority"
        else -> "read_later"
    }

    fun loadFeeds() {
        if (activeTab == 3) return
        scope.launch {
            loading = true
            try {
                articles = api.getArticles(viewName, searchQuery)
            } catch (_: Exception) {
            } finally {
                loading = false
            }
        }
    }

    LaunchedEffect(activeTab, searchQuery) {
        loadFeeds()
    }

    if (selectedArticleId != null) {
        ArticleReaderScreen(
            api = api,
            articleId = selectedArticleId!!,
            onBack = {
                selectedArticleId = null
                loadFeeds()
            }
        )
        return
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("PulseFeed", fontWeight = FontWeight.Bold)
                        Text(
                            "${edition.replaceFirstChar { it.uppercase() }} • $serverUrl",
                            style = MaterialTheme.typography.labelSmall,
                            color = Color(0xFF94A3B8),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                },
                actions = {
                    IconButton(onClick = {
                        scope.launch {
                            loading = true
                            try { api.refreshAll() } catch (_: Exception) {}
                            loadFeeds()
                        }
                    }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                    IconButton(onClick = onDisconnect) {
                        Icon(Icons.Default.Settings, contentDescription = "Switch Server")
                    }
                }
            )
        },
        bottomBar = {
            NavigationBar {
                NavigationBarItem(
                    selected = activeTab == 0,
                    onClick = { activeTab = 0 },
                    icon = { Icon(Icons.Default.WbSunny, null) },
                    label = { Text("Today") }
                )
                NavigationBarItem(
                    selected = activeTab == 1,
                    onClick = { activeTab = 1 },
                    icon = { Icon(Icons.Default.AutoAwesome, null) },
                    label = { Text("Leo AI") }
                )
                NavigationBarItem(
                    selected = activeTab == 2,
                    onClick = { activeTab = 2 },
                    icon = { Icon(Icons.Default.Bookmark, null) },
                    label = { Text("Saved") }
                )
                NavigationBarItem(
                    selected = activeTab == 3,
                    onClick = { activeTab = 3 },
                    icon = { Icon(Icons.Default.Explore, null) },
                    label = { Text("Discover") }
                )
            }
        }
    ) { padding ->
        if (activeTab == 3) {
            DiscoverFeedsScreen(api = api, modifier = Modifier.padding(padding))
        } else {
            Column(modifier = Modifier.fillMaxSize().padding(padding)) {
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = { searchQuery = it },
                    placeholder = { Text("Search articles (FTS5)...") },
                    leadingIcon = { Icon(Icons.Default.Search, null) },
                    singleLine = true,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)
                )

                if (loading && articles.isEmpty()) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator()
                    }
                } else if (articles.isEmpty()) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text("All caught up!", color = Color(0xFF94A3B8))
                    }
                } else {
                    LazyColumn(
                        contentPadding = PaddingValues(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        items(articles, key = { it.id }) { art ->
                            Card(
                                shape = RoundedCornerShape(16.dp),
                                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clickable { selectedArticleId = art.id }
                            ) {
                                Column(Modifier.padding(14.dp)) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        modifier = Modifier.fillMaxWidth()
                                    ) {
                                        Text(
                                            art.feedTitle,
                                            style = MaterialTheme.typography.labelMedium,
                                            color = Emerald,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.weight(1f)
                                        )
                                        if (art.priorityScore >= 65) {
                                            Surface(
                                                color = Violet.copy(alpha = 0.2f),
                                                shape = RoundedCornerShape(8.dp)
                                            ) {
                                                Text(
                                                    "Leo ${art.priorityScore}",
                                                    style = MaterialTheme.typography.labelSmall,
                                                    color = Violet,
                                                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                                                )
                                            }
                                        }
                                    }
                                    Spacer(Modifier.height(6.dp))
                                    Text(
                                        art.title,
                                        style = MaterialTheme.typography.titleMedium,
                                        fontWeight = if (art.isRead) FontWeight.Normal else FontWeight.Bold,
                                        color = if (art.isRead) Color(0xFF94A3B8) else Color.White
                                    )
                                    if (art.summary.isNotBlank()) {
                                        Spacer(Modifier.height(4.dp))
                                        Text(
                                            art.summary,
                                            style = MaterialTheme.typography.bodySmall,
                                            color = Color(0xFF94A3B8),
                                            maxLines = 2,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun DiscoverFeedsScreen(api: PulseFeedApi, modifier: Modifier = Modifier) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var selectedCat by remember { mutableStateOf("All") }
    var categories by remember { mutableStateOf(listOf("All", "Tech", "Cybersecurity", "AI & ML", "World News", "Finance", "Science", "Sports", "Gaming")) }
    var results by remember { mutableStateOf<List<FeedSearchItem>>(emptyList()) }
    var loading by remember { mutableStateOf(false) }

    LaunchedEffect(query, selectedCat) {
        loading = true
        try {
            val res = api.searchFeeds(query, selectedCat)
            if (res.categories.isNotEmpty()) categories = res.categories
            results = res.results
        } catch (_: Exception) {
        } finally {
            loading = false
        }
    }

    Column(modifier = modifier.fillMaxSize()) {
        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            placeholder = { Text("Search any topic (AI, gardening, NFL, security)...") },
            leadingIcon = { Icon(Icons.Default.Search, null) },
            singleLine = true,
            shape = RoundedCornerShape(14.dp),
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)
        )

        Row(
            modifier = Modifier
                .horizontalScroll(rememberScrollState())
                .padding(horizontal = 16.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            categories.forEach { cat ->
                FilterChip(
                    selected = selectedCat == cat,
                    onClick = { selectedCat = cat },
                    label = { Text(cat) }
                )
            }
        }

        if (loading && results.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            LazyColumn(
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                items(results, key = { it.feedUrl }) { item ->
                    Card(
                        shape = RoundedCornerShape(14.dp),
                        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(14.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(Modifier.weight(1f)) {
                                Text(item.title, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(item.category, style = MaterialTheme.typography.labelSmall, color = Emerald)
                                if (item.description.isNotBlank()) {
                                    Text(
                                        item.description,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Color(0xFF94A3B8),
                                        maxLines = 2,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                            Spacer(Modifier.width(10.dp))
                            if (item.alreadySubscribed) {
                                Text("Following", color = Emerald, style = MaterialTheme.typography.labelMedium)
                            } else {
                                FilledTonalButton(onClick = {
                                    scope.launch {
                                        try {
                                            api.subscribeFeed(item.feedUrl, item.title)
                                            results = results.map {
                                                if (it.feedUrl == item.feedUrl) it.copy(alreadySubscribed = true) else it
                                            }
                                        } catch (_: Exception) {}
                                    }
                                }) {
                                    Text("Follow")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ArticleReaderScreen(
    api: PulseFeedApi,
    articleId: String,
    onBack: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var detail by remember { mutableStateOf<ArticleDetailResp?>(null) }
    var summarizing by remember { mutableStateOf(false) }

    LaunchedEffect(articleId) {
        try {
            val d = api.getArticle(articleId)
            detail = d
            if (!d.isRead) api.updateArticleState(articleId, isRead = true)
        } catch (_: Exception) {}
    }

    Scaffold(
        topBar = {
            TopAppBar(
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                title = { Text(detail?.feedTitle ?: "Reader", maxLines = 1, overflow = TextOverflow.Ellipsis) },
                actions = {
                    detail?.let { d ->
                        IconButton(onClick = {
                            scope.launch {
                                val next = !d.isReadLater
                                api.updateArticleState(d.id, isReadLater = next)
                                detail = d.copy(isReadLater = next)
                            }
                        }) {
                            Icon(
                                Icons.Default.Bookmark,
                                contentDescription = "Read Later",
                                tint = if (d.isReadLater) Emerald else Color.White
                            )
                        }
                        IconButton(onClick = {
                            context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(d.url)))
                        }) {
                            Icon(Icons.Default.OpenInBrowser, contentDescription = "Open Original")
                        }
                    }
                }
            )
        }
    ) { padding ->
        val d = detail
        if (d == null) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(18.dp)
            ) {
                Text(d.title, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
                Spacer(Modifier.height(6.dp))
                Text(
                    "${d.feedTitle} • ${d.readingTimeMins} min read • Leo ${d.priorityScore}",
                    style = MaterialTheme.typography.labelMedium,
                    color = Emerald
                )
                Spacer(Modifier.height(16.dp))

                // Leo AI Summary Card
                Card(
                    shape = RoundedCornerShape(14.dp),
                    colors = CardDefaults.cardColors(containerColor = Violet.copy(alpha = 0.14f)),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(Modifier.padding(14.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text("Leo AI Brief", fontWeight = FontWeight.Bold, color = Violet)
                            TextButton(
                                enabled = !summarizing,
                                onClick = {
                                    scope.launch {
                                        summarizing = true
                                        try {
                                            val res = api.generateAiSummary(d.id)
                                            detail = d.copy(aiSummary = res.aiSummary, aiSentiment = res.aiSentiment)
                                        } catch (_: Exception) {}
                                        summarizing = false
                                    }
                                }
                            ) {
                                Text(if (summarizing) "Analyzing..." else "Summarize")
                            }
                        }
                        if (!d.aiSummary.isNullOrBlank()) {
                            Text(d.aiSummary, style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }

                Spacer(Modifier.height(18.dp))
                Text(
                    d.contentText.ifBlank { d.summary },
                    style = MaterialTheme.typography.bodyLarge,
                    lineHeight = MaterialTheme.typography.bodyLarge.lineHeight * 1.15f
                )
            }
        }
    }
}
