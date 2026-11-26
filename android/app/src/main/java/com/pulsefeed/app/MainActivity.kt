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
    val localEngine = remember { LocalPulseFeedEngine(context) }

    // Default to 100% standalone on-device local mode (no server required!)
    var mode by remember { mutableStateOf(prefs.getString("mode", "local") ?: "local") }
    var serverUrl by remember { mutableStateOf(prefs.getString("server_url", "") ?: "") }
    var token by remember { mutableStateOf(prefs.getString("jwt_token", "") ?: "") }
    var showServerSetup by remember { mutableStateOf(false) }

    if (showServerSetup) {
        ConnectScreen(
            initialUrl = serverUrl.ifBlank { "https://your-server.onrender.com" },
            onUseLocal = {
                mode = "local"
                prefs.edit().putString("mode", "local").apply()
                showServerSetup = false
            },
            onConnected = { url, jwt ->
                mode = "remote"
                serverUrl = url
                token = jwt
                prefs.edit()
                    .putString("mode", "remote")
                    .putString("server_url", url)
                    .putString("jwt_token", jwt)
                    .apply()
                showServerSetup = false
            }
        )
    } else {
        val remoteApi = remember(mode, serverUrl, token) {
            if (mode == "remote" && serverUrl.isNotBlank()) PulseFeedApi(serverUrl, token.ifBlank { null }) else null
        }
        MainWorkspace(
            localEngine = localEngine,
            remoteApi = remoteApi,
            modeLabel = if (remoteApi == null) "Personal • On-Device Local" else "Remote • $serverUrl",
            onOpenServerSettings = { showServerSetup = true }
        )
    }
}

@Composable
fun ConnectScreen(
    initialUrl: String,
    onUseLocal: () -> Unit,
    onConnected: (String, String) -> Unit
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
        Text("PulseFeed Mode", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Text(
            "Run 100% locally on your phone or connect to a hosted Enterprise server",
            style = MaterialTheme.typography.bodySmall,
            color = Color(0xFF94A3B8)
        )
        Spacer(Modifier.height(20.dp))

        Button(
            onClick = onUseLocal,
            modifier = Modifier.fillMaxWidth().height(50.dp),
            shape = RoundedCornerShape(14.dp)
        ) {
            Icon(Icons.Default.PhoneAndroid, null)
            Spacer(Modifier.width(8.dp))
            Text("Use Standalone Local Mode (No Server)")
        }

        Spacer(Modifier.height(24.dp))
        HorizontalDivider()
        Spacer(Modifier.height(16.dp))
        Text("Or Connect to Remote Server", style = MaterialTheme.typography.labelLarge, color = Color(0xFF94A3B8))
        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = url,
            onValueChange = { url = it },
            label = { Text("PulseFeed Server URL") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth()
        )

        if (needsLogin) {
            Spacer(Modifier.height(10.dp))
            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text("Work Email") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
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
            Spacer(Modifier.height(10.dp))
            Text(error!!, color = Color(0xFFF87171), style = MaterialTheme.typography.bodySmall)
        }

        Spacer(Modifier.height(14.dp))
        OutlinedButton(
            onClick = {
                scope.launch {
                    loading = true
                    error = null
                    try {
                        val api = PulseFeedApi(url)
                        val ed = api.getEdition()
                        if (ed.isPersonal) {
                            onConnected(url, "")
                        } else if (!needsLogin) {
                            needsLogin = true
                        } else {
                            val jwt = api.login(email, password)
                            onConnected(url, jwt)
                        }
                    } catch (e: Exception) {
                        error = e.message ?: "Could not reach server"
                    } finally {
                        loading = false
                    }
                }
            },
            enabled = !loading && url.isNotBlank(),
            modifier = Modifier.fillMaxWidth().height(48.dp),
            shape = RoundedCornerShape(14.dp)
        ) {
            Text(if (loading) "Connecting..." else if (needsLogin) "Sign In" else "Connect Remote Server")
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainWorkspace(
    localEngine: LocalPulseFeedEngine,
    remoteApi: PulseFeedApi?,
    modeLabel: String,
    onOpenServerSettings: () -> Unit
) {
    val scope = rememberCoroutineScope()
    var activeTab by remember { mutableIntStateOf(0) } // 0=Today, 1=Leo, 2=Saved, 3=Discover, 4=Feeds
    var searchQuery by remember { mutableStateOf("") }
    var articles by remember { mutableStateOf<List<ArticleItem>>(emptyList()) }
    var loading by remember { mutableStateOf(false) }
    var selectedArticleId by remember { mutableStateOf<String?>(null) }
    var didInitialSync by remember { mutableStateOf(false) }

    val viewName = when (activeTab) {
        0 -> "today"
        1 -> "leo_priority"
        else -> "read_later"
    }

    fun loadArticles() {
        if (activeTab >= 3) return
        scope.launch {
            loading = true
            try {
                articles = if (remoteApi != null) {
                    remoteApi.getArticles(viewName, searchQuery)
                } else {
                    localEngine.getArticles(viewName, searchQuery)
                }
            } catch (_: Exception) {
            } finally {
                loading = false
            }
        }
    }

    // On first launch in local mode, automatically fetch all starter RSS feeds right on the device!
    LaunchedEffect(remoteApi) {
        if (remoteApi == null && !didInitialSync) {
            didInitialSync = true
            val existing = localEngine.getArticles("today")
            if (existing.isEmpty()) {
                loading = true
                localEngine.refreshAll()
                loading = false
            }
            loadArticles()
        }
    }

    LaunchedEffect(activeTab, searchQuery, remoteApi) {
        loadArticles()
    }

    if (selectedArticleId != null) {
        ArticleReaderScreen(
            localEngine = localEngine,
            remoteApi = remoteApi,
            articleId = selectedArticleId!!,
            onBack = {
                selectedArticleId = null
                loadArticles()
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
                            modeLabel,
                            style = MaterialTheme.typography.labelSmall,
                            color = Emerald,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                },
                actions = {
                    IconButton(onClick = {
                        scope.launch {
                            loading = true
                            try {
                                if (remoteApi != null) remoteApi.refreshAll()
                                else localEngine.refreshAll()
                            } catch (_: Exception) {}
                            loadArticles()
                        }
                    }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh Feeds")
                    }
                    IconButton(onClick = onOpenServerSettings) {
                        Icon(Icons.Default.Settings, contentDescription = "Mode / Server Settings")
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
                if (remoteApi == null) {
                    NavigationBarItem(
                        selected = activeTab == 4,
                        onClick = { activeTab = 4 },
                        icon = { Icon(Icons.Default.RssFeed, null) },
                        label = { Text("Feeds") }
                    )
                }
            }
        }
    ) { padding ->
        when (activeTab) {
            3 -> DiscoverFeedsScreen(
                localEngine = localEngine,
                remoteApi = remoteApi,
                modifier = Modifier.padding(padding)
            )
            4 -> ManageLocalFeedsScreen(
                localEngine = localEngine,
                modifier = Modifier.padding(padding)
            )
            else -> Column(modifier = Modifier.fillMaxSize().padding(padding)) {
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = { searchQuery = it },
                    placeholder = { Text("Search articles...") },
                    leadingIcon = { Icon(Icons.Default.Search, null) },
                    singleLine = true,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp)
                )

                if (loading && articles.isEmpty()) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            CircularProgressIndicator()
                            Spacer(Modifier.height(10.dp))
                            Text("Syncing live RSS feeds...", color = Color(0xFF94A3B8), style = MaterialTheme.typography.bodySmall)
                        }
                    }
                } else if (articles.isEmpty()) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text("All caught up! Tap Refresh or Discover to add feeds.", color = Color(0xFF94A3B8))
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
fun ManageLocalFeedsScreen(localEngine: LocalPulseFeedEngine, modifier: Modifier = Modifier) {
    var feeds by remember { mutableStateOf(localEngine.listFeeds()) }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        item {
            Text(
                "Subscribed Feeds (${feeds.size})",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            Spacer(Modifier.height(4.dp))
        }
        items(feeds, key = { it.id }) { feed ->
            Card(
                shape = RoundedCornerShape(14.dp),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(14.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(feed.title, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(feed.category, style = MaterialTheme.typography.labelSmall, color = Emerald)
                        Text(feed.feedUrl, style = MaterialTheme.typography.bodySmall, color = Color(0xFF94A3B8), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                    IconButton(onClick = {
                        localEngine.deleteFeed(feed.id)
                        feeds = localEngine.listFeeds()
                    }) {
                        Icon(Icons.Default.Delete, contentDescription = "Unfollow", tint = Color(0xFFF87171))
                    }
                }
            }
        }
    }
}

@Composable
fun DiscoverFeedsScreen(
    localEngine: LocalPulseFeedEngine,
    remoteApi: PulseFeedApi?,
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var selectedCat by remember { mutableStateOf("All") }
    var categories by remember { mutableStateOf(listOf("All", "Tech", "Cybersecurity", "AI & ML", "Cloud & Infra", "World News", "Finance", "Science", "Sports", "Gaming")) }
    var results by remember { mutableStateOf<List<FeedSearchItem>>(emptyList()) }
    var loading by remember { mutableStateOf(false) }

    LaunchedEffect(query, selectedCat, remoteApi) {
        loading = true
        try {
            val res = if (remoteApi != null) {
                remoteApi.searchFeeds(query, selectedCat)
            } else {
                localEngine.searchFeeds(query, selectedCat)
            }
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
                                            if (remoteApi != null) {
                                                remoteApi.subscribeFeed(item.feedUrl, item.title)
                                            } else {
                                                localEngine.subscribeFeed(item.feedUrl, item.title, item.category)
                                            }
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
    localEngine: LocalPulseFeedEngine,
    remoteApi: PulseFeedApi?,
    articleId: String,
    onBack: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var detail by remember { mutableStateOf<ArticleDetailResp?>(null) }
    var summarizing by remember { mutableStateOf(false) }

    LaunchedEffect(articleId, remoteApi) {
        try {
            val d = if (remoteApi != null) remoteApi.getArticle(articleId) else localEngine.getArticle(articleId)
            detail = d
            if (d != null && !d.isRead) {
                if (remoteApi != null) remoteApi.updateArticleState(articleId, isRead = true)
                else localEngine.updateArticleState(articleId, isRead = true)
            }
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
                                if (remoteApi != null) remoteApi.updateArticleState(d.id, isReadLater = next)
                                else localEngine.updateArticleState(d.id, isReadLater = next)
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
                                            val res = if (remoteApi != null) {
                                                remoteApi.generateAiSummary(d.id)
                                            } else {
                                                localEngine.generateLocalSummary(d.id)
                                            }
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
