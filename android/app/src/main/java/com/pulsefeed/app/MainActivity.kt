package com.pulsefeed.app

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
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
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch

private val Emerald = Color(0xFF10B981)
private val EmeraldDark = Color(0xFF059669)
private val Violet = Color(0xFF8B5CF6)
private val BlueAccent = Color(0xFF3B82F6)
private val AmberAccent = Color(0xFFF59E0B)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val prefs = remember { getSharedPreferences("pulsefeed_prefs", Context.MODE_PRIVATE) }
            var darkTheme by remember { mutableStateOf(prefs.getBoolean("dark_theme", true)) }

            val scheme = if (darkTheme) {
                darkColorScheme(
                    primary = Emerald,
                    secondary = Violet,
                    background = Color(0xFF020617),
                    surface = Color(0xFF0F172A),
                    surfaceVariant = Color(0xFF1E293B),
                    outline = Color(0xFF1E293B),
                    onPrimary = Color.White,
                    onBackground = Color(0xFFF8FAFC),
                    onSurface = Color(0xFFF8FAFC),
                    onSurfaceVariant = Color(0xFF94A3B8)
                )
            } else {
                lightColorScheme(
                    primary = EmeraldDark,
                    secondary = Violet,
                    background = Color(0xFFFFFFFF),
                    surface = Color(0xFFF8FAFC),
                    surfaceVariant = Color(0xFFF1F5F9),
                    outline = Color(0xFFE2E8F0),
                    onPrimary = Color.White,
                    onBackground = Color(0xFF0F172A),
                    onSurface = Color(0xFF0F172A),
                    onSurfaceVariant = Color(0xFF64748B)
                )
            }

            MaterialTheme(colorScheme = scheme) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    PulseFeedRoot(
                        darkTheme = darkTheme,
                        onToggleTheme = {
                            darkTheme = !darkTheme
                            prefs.edit().putBoolean("dark_theme", darkTheme).apply()
                        }
                    )
                }
            }
        }
    }
}

@Composable
fun PulseFeedRoot(darkTheme: Boolean, onToggleTheme: () -> Unit) {
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("pulsefeed_prefs", Context.MODE_PRIVATE) }
    val localEngine = remember { LocalPulseFeedEngine(context) }

    var mode by remember { mutableStateOf(prefs.getString("mode", "local") ?: "local") }
    var serverUrl by remember { mutableStateOf(prefs.getString("server_url", "") ?: "") }
    var token by remember { mutableStateOf(prefs.getString("jwt_token", "") ?: "") }
    var showServerSetup by remember { mutableStateOf(false) }

    if (showServerSetup) {
        ConnectScreen(
            initialUrl = serverUrl.ifBlank { "http://35.211.206.254" },
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
            darkTheme = darkTheme,
            onToggleTheme = onToggleTheme,
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
        Spacer(Modifier.height(14.dp))
        Text("PulseFeed", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Text(
            "Standalone Local Mode or Hosted Enterprise Server",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(Modifier.height(22.dp))

        Button(
            onClick = onUseLocal,
            modifier = Modifier.fillMaxWidth().height(50.dp),
            shape = RoundedCornerShape(14.dp),
            colors = ButtonDefaults.buttonColors(containerColor = EmeraldDark)
        ) {
            Icon(Icons.Default.PhoneAndroid, null)
            Spacer(Modifier.width(8.dp))
            Text("Use Personal Local Mode (No Server)", fontWeight = FontWeight.SemiBold)
        }

        Spacer(Modifier.height(22.dp))
        HorizontalDivider()
        Spacer(Modifier.height(16.dp))
        Text("Or Connect to Remote Enterprise Server", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = url,
            onValueChange = { url = it },
            label = { Text("Server URL") },
            singleLine = true,
            shape = RoundedCornerShape(12.dp),
            modifier = Modifier.fillMaxWidth()
        )

        if (needsLogin) {
            Spacer(Modifier.height(10.dp))
            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text("Work Email") },
                singleLine = true,
                shape = RoundedCornerShape(12.dp),
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text("Password") },
                visualTransformation = PasswordVisualTransformation(),
                singleLine = true,
                shape = RoundedCornerShape(12.dp),
                modifier = Modifier.fillMaxWidth()
            )
        }

        if (error != null) {
            Spacer(Modifier.height(10.dp))
            Text(error!!, color = Color(0xFFEF4444), style = MaterialTheme.typography.bodySmall)
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
    darkTheme: Boolean,
    onToggleTheme: () -> Unit,
    onOpenServerSettings: () -> Unit
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val drawerState = rememberDrawerState(DrawerValue.Closed)

    var activeView by remember { mutableStateOf("today") } // today | leo_priority | all | read_later | discover
    var activeFeedId by remember { mutableStateOf<String?>(null) }
    var activeTitle by remember { mutableStateOf("Today") }

    var searchQuery by remember { mutableStateOf("") }
    var unreadOnly by remember { mutableStateOf(false) }
    var sortBy by remember { mutableStateOf("newest") } // newest | priority
    var compactTitleView by remember { mutableStateOf(false) }

    var articles by remember { mutableStateOf<List<ArticleItem>>(emptyList()) }
    var localFeeds by remember { mutableStateOf<List<LocalFeed>>(emptyList()) }
    var counts by remember { mutableStateOf(Triple(0, 0, 0)) }
    var loading by remember { mutableStateOf(false) }
    var selectedArticleId by remember { mutableStateOf<String?>(null) }
    var didInitialSync by remember { mutableStateOf(false) }

    fun refreshState() {
        if (remoteApi == null) {
            localFeeds = localEngine.listFeeds()
            counts = localEngine.getCounts()
        }
        if (activeView == "discover") return
        scope.launch {
            loading = true
            try {
                articles = if (remoteApi != null) {
                    remoteApi.getArticles(activeView, searchQuery)
                } else {
                    localEngine.getArticles(activeView, searchQuery, unreadOnly, sortBy, activeFeedId)
                }
            } catch (_: Exception) {
            } finally {
                loading = false
            }
        }
    }

    LaunchedEffect(remoteApi) {
        if (remoteApi == null && !didInitialSync) {
            didInitialSync = true
            if (localEngine.getArticles("today").isEmpty()) {
                loading = true
                localEngine.refreshAll()
                loading = false
            }
            refreshState()
        }
    }

    LaunchedEffect(activeView, activeFeedId, searchQuery, unreadOnly, sortBy, remoteApi) {
        refreshState()
    }

    if (selectedArticleId != null) {
        ArticleReaderScreen(
            localEngine = localEngine,
            remoteApi = remoteApi,
            articleId = selectedArticleId!!,
            onBack = {
                selectedArticleId = null
                refreshState()
            }
        )
        return
    }

    // Feedly-style Sidebar Navigation Drawer matching Web App
    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet(
                drawerContainerColor = MaterialTheme.colorScheme.surface,
                modifier = Modifier.width(300.dp)
            ) {
                // Brand Header (mirrors Web Sidebar)
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Surface(
                            color = EmeraldDark,
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier.size(34.dp)
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(Icons.Default.RssFeed, null, tint = Color.White, modifier = Modifier.size(18.dp))
                            }
                        }
                        Spacer(Modifier.width(10.dp))
                        Column {
                            Text("PulseFeed", fontWeight = FontWeight.Bold, fontSize = 16.sp)
                            Text(
                                if (remoteApi == null) "PERSONAL LOCAL" else "ENTERPRISE",
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                color = Emerald
                            )
                        }
                    }
                    Button(
                        onClick = {
                            activeView = "discover"
                            activeFeedId = null
                            activeTitle = "Discover & Follow"
                            scope.launch { drawerState.close() }
                        },
                        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
                        shape = RoundedCornerShape(10.dp),
                        colors = ButtonDefaults.buttonColors(containerColor = EmeraldDark),
                        modifier = Modifier.height(32.dp)
                    ) {
                        Icon(Icons.Default.Add, null, modifier = Modifier.size(14.dp))
                        Spacer(Modifier.width(4.dp))
                        Text("Follow", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
                HorizontalDivider(color = MaterialTheme.colorScheme.outline)

                LazyColumn(
                    modifier = Modifier.weight(1f).padding(horizontal = 10.dp, vertical = 10.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    item {
                        DrawerItem(
                            label = "Today",
                            icon = Icons.Default.WbSunny,
                            iconTint = AmberAccent,
                            badge = counts.first.takeIf { it > 0 && remoteApi == null },
                            selected = activeView == "today" && activeFeedId == null
                        ) {
                            activeView = "today"; activeFeedId = null; activeTitle = "Today"
                            scope.launch { drawerState.close() }
                        }
                        DrawerItem(
                            label = "Leo Priority",
                            icon = Icons.Default.AutoAwesome,
                            iconTint = Violet,
                            badge = counts.second.takeIf { it > 0 && remoteApi == null },
                            selected = activeView == "leo_priority" && activeFeedId == null
                        ) {
                            activeView = "leo_priority"; activeFeedId = null; activeTitle = "Leo Priority"
                            scope.launch { drawerState.close() }
                        }
                        DrawerItem(
                            label = "All Articles",
                            icon = Icons.Default.Inbox,
                            iconTint = Emerald,
                            badge = counts.first.takeIf { it > 0 && remoteApi == null },
                            selected = activeView == "all" && activeFeedId == null
                        ) {
                            activeView = "all"; activeFeedId = null; activeTitle = "All Articles"
                            scope.launch { drawerState.close() }
                        }
                        DrawerItem(
                            label = "Read Later",
                            icon = Icons.Default.Bookmark,
                            iconTint = BlueAccent,
                            badge = counts.third.takeIf { it > 0 && remoteApi == null },
                            selected = activeView == "read_later" && activeFeedId == null
                        ) {
                            activeView = "read_later"; activeFeedId = null; activeTitle = "Read Later"
                            scope.launch { drawerState.close() }
                        }
                    }

                    if (remoteApi == null && localFeeds.isNotEmpty()) {
                        item {
                            Spacer(Modifier.height(12.dp))
                            Text(
                                "FEEDS (${localFeeds.size})",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp)
                            )
                        }
                        items(localFeeds, key = { it.id }) { feed ->
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .background(
                                        if (activeFeedId == feed.id) Emerald.copy(alpha = 0.14f) else Color.Transparent,
                                        RoundedCornerShape(10.dp)
                                    )
                                    .clickable {
                                        activeView = "all"
                                        activeFeedId = feed.id
                                        activeTitle = feed.title
                                        scope.launch { drawerState.close() }
                                    }
                                    .padding(horizontal = 12.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(Icons.Default.RssFeed, null, tint = Emerald, modifier = Modifier.size(15.dp))
                                Spacer(Modifier.width(10.dp))
                                Text(
                                    feed.title,
                                    fontSize = 13.sp,
                                    fontWeight = if (activeFeedId == feed.id) FontWeight.Bold else FontWeight.Medium,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = Modifier.weight(1f)
                                )
                                IconButton(
                                    onClick = {
                                        localEngine.deleteFeed(feed.id)
                                        if (activeFeedId == feed.id) {
                                            activeFeedId = null
                                            activeTitle = "Today"
                                        }
                                        refreshState()
                                    },
                                    modifier = Modifier.size(28.dp)
                                ) {
                                    Icon(
                                        Icons.Default.DeleteOutline,
                                        contentDescription = "Unfollow",
                                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                        modifier = Modifier.size(16.dp)
                                    )
                                }
                            }
                        }
                    }
                }

                // Drawer Footer (Theme + Server Mode)
                HorizontalDivider(color = MaterialTheme.colorScheme.outline)
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 10.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TextButton(onClick = {
                        scope.launch { drawerState.close() }
                        onOpenServerSettings()
                    }) {
                        Icon(Icons.Default.Settings, null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(6.dp))
                        Text(if (remoteApi == null) "Local Mode" else "Remote Mode", fontSize = 12.sp)
                    }
                    IconButton(onClick = onToggleTheme) {
                        Icon(if (darkTheme) Icons.Default.LightMode else Icons.Default.DarkMode, "Toggle Theme")
                    }
                }
            }
        }
    ) {
        Scaffold(
            topBar = {
                TopAppBar(
                    navigationIcon = {
                        IconButton(onClick = { scope.launch { drawerState.open() } }) {
                            Icon(Icons.Default.Menu, contentDescription = "Open Sidebar")
                        }
                    },
                    title = {
                        Text(activeTitle, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    },
                    actions = {
                        if (activeView != "discover") {
                            IconButton(onClick = { compactTitleView = !compactTitleView }) {
                                Icon(
                                    if (compactTitleView) Icons.Default.ViewAgenda else Icons.Default.ViewHeadline,
                                    contentDescription = "Toggle Magazine / Compact View"
                                )
                            }
                            if (remoteApi == null) {
                                IconButton(onClick = {
                                    localEngine.markAllRead(activeFeedId)
                                    refreshState()
                                }) {
                                    Icon(Icons.Default.DoneAll, contentDescription = "Mark All Read")
                                }
                            }
                            IconButton(onClick = {
                                scope.launch {
                                    loading = true
                                    try {
                                        if (remoteApi != null) remoteApi.refreshAll()
                                        else localEngine.refreshAll()
                                    } catch (_: Exception) {}
                                    refreshState()
                                }
                            }) {
                                Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                            }
                        }
                    }
                )
            },
            bottomBar = {
                NavigationBar(containerColor = MaterialTheme.colorScheme.surface) {
                    NavigationBarItem(
                        selected = activeView == "today" && activeFeedId == null,
                        onClick = { activeView = "today"; activeFeedId = null; activeTitle = "Today" },
                        icon = { Icon(Icons.Default.WbSunny, null) },
                        label = { Text("Today") }
                    )
                    NavigationBarItem(
                        selected = activeView == "leo_priority",
                        onClick = { activeView = "leo_priority"; activeFeedId = null; activeTitle = "Leo Priority" },
                        icon = { Icon(Icons.Default.AutoAwesome, null) },
                        label = { Text("Leo AI") }
                    )
                    NavigationBarItem(
                        selected = activeView == "read_later",
                        onClick = { activeView = "read_later"; activeFeedId = null; activeTitle = "Saved" },
                        icon = { Icon(Icons.Default.Bookmark, null) },
                        label = { Text("Saved") }
                    )
                    NavigationBarItem(
                        selected = activeView == "discover",
                        onClick = { activeView = "discover"; activeFeedId = null; activeTitle = "Discover & Follow" },
                        icon = { Icon(Icons.Default.Explore, null) },
                        label = { Text("Discover") }
                    )
                }
            }
        ) { padding ->
            if (activeView == "discover") {
                DiscoverFeedsScreen(
                    localEngine = localEngine,
                    remoteApi = remoteApi,
                    onSubscribed = { refreshState() },
                    modifier = Modifier.padding(padding)
                )
            } else {
                Column(modifier = Modifier.fillMaxSize().padding(padding)) {
                    // Search + Filter Row (mirrors Web App Toolbar)
                    OutlinedTextField(
                        value = searchQuery,
                        onValueChange = { searchQuery = it },
                        placeholder = { Text("Search articles...") },
                        leadingIcon = { Icon(Icons.Default.Search, null, modifier = Modifier.size(18.dp)) },
                        singleLine = true,
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 6.dp)
                    )

                    Row(
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 2.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        FilterChip(
                            selected = unreadOnly,
                            onClick = { unreadOnly = !unreadOnly },
                            label = { Text("Unread", fontSize = 12.sp) }
                        )
                        FilterChip(
                            selected = sortBy == "priority",
                            onClick = { sortBy = if (sortBy == "priority") "newest" else "priority" },
                            label = { Text(if (sortBy == "priority") "Sort: Leo Priority" else "Sort: Newest", fontSize = 12.sp) }
                        )
                    }

                    if (loading && articles.isEmpty()) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                CircularProgressIndicator(color = Emerald)
                                Spacer(Modifier.height(10.dp))
                                Text("Syncing live RSS feeds...", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp)
                            }
                        }
                    } else if (articles.isEmpty()) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Text("All caught up!", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    } else {
                        LazyColumn(
                            contentPadding = PaddingValues(14.dp),
                            verticalArrangement = Arrangement.spacedBy(if (compactTitleView) 6.dp else 12.dp)
                        ) {
                            items(articles, key = { it.id }) { art ->
                                if (compactTitleView) {
                                    // Web-matched Title-Only Compact Row
                                    Surface(
                                        shape = RoundedCornerShape(12.dp),
                                        color = MaterialTheme.colorScheme.surface,
                                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
                                        modifier = Modifier.fillMaxWidth().clickable { selectedArticleId = art.id }
                                    ) {
                                        Row(
                                            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
                                            verticalAlignment = Alignment.CenterVertically
                                        ) {
                                            if (art.priorityScore >= 65) {
                                                Surface(
                                                    color = Violet.copy(alpha = 0.16f),
                                                    shape = RoundedCornerShape(6.dp)
                                                ) {
                                                    Text(
                                                        "${art.priorityScore}",
                                                        color = Violet,
                                                        fontSize = 10.sp,
                                                        fontWeight = FontWeight.Bold,
                                                        modifier = Modifier.padding(horizontal = 5.dp, vertical = 2.dp)
                                                    )
                                                }
                                                Spacer(Modifier.width(8.dp))
                                            }
                                            Column(Modifier.weight(1f)) {
                                                Text(
                                                    art.title,
                                                    fontSize = 14.sp,
                                                    fontWeight = if (art.isRead) FontWeight.Normal else FontWeight.SemiBold,
                                                    color = if (art.isRead) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis
                                                )
                                                Text(art.feedTitle, fontSize = 11.sp, color = Emerald)
                                            }
                                        }
                                    }
                                } else {
                                    // Web-matched Feedly Magazine Card
                                    Card(
                                        shape = RoundedCornerShape(16.dp),
                                        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
                                        modifier = Modifier.fillMaxWidth().clickable { selectedArticleId = art.id }
                                    ) {
                                        Column(Modifier.padding(14.dp)) {
                                            Row(
                                                verticalAlignment = Alignment.CenterVertically,
                                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                                            ) {
                                                if (art.priorityScore >= 65) {
                                                    Surface(
                                                        color = Violet.copy(alpha = 0.16f),
                                                        shape = RoundedCornerShape(99.dp)
                                                    ) {
                                                        Text(
                                                            "Leo ${art.priorityScore}",
                                                            color = Violet,
                                                            fontSize = 10.sp,
                                                            fontWeight = FontWeight.Bold,
                                                            modifier = Modifier.padding(horizontal = 7.dp, vertical = 2.dp)
                                                        )
                                                    }
                                                }
                                                Text(
                                                    art.feedTitle,
                                                    fontSize = 12.sp,
                                                    fontWeight = FontWeight.SemiBold,
                                                    color = Emerald,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis,
                                                    modifier = Modifier.weight(1f)
                                                )
                                                Text(
                                                    "${art.readingTimeMins}m read",
                                                    fontSize = 11.sp,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                            Spacer(Modifier.height(6.dp))
                                            Text(
                                                art.title,
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = if (art.isRead) FontWeight.Normal else FontWeight.Bold,
                                                color = if (art.isRead) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface
                                            )
                                            if (art.summary.isNotBlank()) {
                                                Spacer(Modifier.height(4.dp))
                                                Text(
                                                    art.summary,
                                                    style = MaterialTheme.typography.bodySmall,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                    maxLines = 2,
                                                    overflow = TextOverflow.Ellipsis
                                                )
                                            }
                                            Spacer(Modifier.height(8.dp))
                                            Row(
                                                modifier = Modifier.fillMaxWidth(),
                                                horizontalArrangement = Arrangement.End
                                            ) {
                                                IconButton(
                                                    onClick = {
                                                        val next = !art.isReadLater
                                                        scope.launch {
                                                            if (remoteApi != null) remoteApi.updateArticleState(art.id, isReadLater = next)
                                                            else localEngine.updateArticleState(art.id, isReadLater = next)
                                                            refreshState()
                                                        }
                                                    },
                                                    modifier = Modifier.size(32.dp)
                                                ) {
                                                    Icon(
                                                        if (art.isReadLater) Icons.Default.Bookmark else Icons.Default.BookmarkBorder,
                                                        contentDescription = "Read Later",
                                                        tint = if (art.isReadLater) BlueAccent else MaterialTheme.colorScheme.onSurfaceVariant,
                                                        modifier = Modifier.size(18.dp)
                                                    )
                                                }
                                                IconButton(
                                                    onClick = {
                                                        val next = !art.isRead
                                                        scope.launch {
                                                            if (remoteApi != null) remoteApi.updateArticleState(art.id, isRead = next)
                                                            else localEngine.updateArticleState(art.id, isRead = next)
                                                            refreshState()
                                                        }
                                                    },
                                                    modifier = Modifier.size(32.dp)
                                                ) {
                                                    Icon(
                                                        Icons.Default.Check,
                                                        contentDescription = "Toggle Read",
                                                        tint = if (art.isRead) Emerald else MaterialTheme.colorScheme.onSurfaceVariant,
                                                        modifier = Modifier.size(18.dp)
                                                    )
                                                }
                                                IconButton(
                                                    onClick = {
                                                        context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(art.url)))
                                                    },
                                                    modifier = Modifier.size(32.dp)
                                                ) {
                                                    Icon(
                                                        Icons.Default.OpenInNew,
                                                        contentDescription = "Open Original",
                                                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                                        modifier = Modifier.size(17.dp)
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
        }
    }
}

@Composable
private fun DrawerItem(
    label: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    iconTint: Color,
    badge: Int?,
    selected: Boolean,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(
                if (selected) Emerald.copy(alpha = 0.14f) else Color.Transparent,
                RoundedCornerShape(12.dp)
            )
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(icon, null, tint = iconTint, modifier = Modifier.size(18.dp))
        Spacer(Modifier.width(12.dp))
        Text(
            label,
            fontSize = 14.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
            color = if (selected) Emerald else MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f)
        )
        if (badge != null) {
            Surface(
                color = MaterialTheme.colorScheme.surfaceVariant,
                shape = RoundedCornerShape(6.dp)
            ) {
                Text(
                    "$badge",
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                )
            }
        }
    }
}

@Composable
fun DiscoverFeedsScreen(
    localEngine: LocalPulseFeedEngine,
    remoteApi: PulseFeedApi?,
    onSubscribed: () -> Unit,
    modifier: Modifier = Modifier
) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var directUrl by remember { mutableStateOf("") }
    var selectedCat by remember { mutableStateOf("All") }
    var categories by remember { mutableStateOf(listOf("All", "Tech", "Cybersecurity", "AI & ML", "Cloud & Infra", "World News", "Finance", "Science", "Sports", "Gaming")) }
    var results by remember { mutableStateOf<List<FeedSearchItem>>(emptyList()) }
    var loading by remember { mutableStateOf(false) }

    LaunchedEffect(query, selectedCat, remoteApi) {
        loading = true
        try {
            val res = if (remoteApi != null) remoteApi.searchFeeds(query, selectedCat)
            else localEngine.searchFeeds(query, selectedCat)
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
            modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 6.dp)
        )

        // Quick Direct RSS URL Add Row
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            OutlinedTextField(
                value = directUrl,
                onValueChange = { directUrl = it },
                placeholder = { Text("Or paste RSS/Atom URL...") },
                singleLine = true,
                shape = RoundedCornerShape(12.dp),
                modifier = Modifier.weight(1f)
            )
            Button(
                enabled = directUrl.startsWith("http"),
                onClick = {
                    val u = directUrl.trim()
                    directUrl = ""
                    scope.launch {
                        runCatching {
                            if (remoteApi != null) remoteApi.subscribeFeed(u, u)
                            else localEngine.subscribeFeed(u, u, "Custom")
                            onSubscribed()
                        }
                    }
                },
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(containerColor = EmeraldDark)
            ) {
                Text("Add")
            }
        }

        Row(
            modifier = Modifier
                .horizontalScroll(rememberScrollState())
                .padding(horizontal = 14.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            categories.forEach { cat ->
                FilterChip(
                    selected = selectedCat == cat,
                    onClick = { selectedCat = cat },
                    label = { Text(cat, fontSize = 12.sp) }
                )
            }
        }

        if (loading && results.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Emerald)
            }
        } else {
            LazyColumn(
                contentPadding = PaddingValues(14.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                items(results, key = { it.feedUrl }) { item ->
                    Card(
                        shape = RoundedCornerShape(14.dp),
                        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(14.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(Modifier.weight(1f)) {
                                Text(item.title, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                                    Text(item.category, style = MaterialTheme.typography.labelSmall, color = Emerald)
                                    if (item.source == "topic_feed") {
                                        Text("• Live Topic", style = MaterialTheme.typography.labelSmall, color = Violet)
                                    }
                                }
                                if (item.description.isNotBlank()) {
                                    Text(
                                        item.description,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        maxLines = 2,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                            Spacer(Modifier.width(10.dp))
                            if (item.alreadySubscribed) {
                                Text("Following ✓", color = Emerald, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                            } else {
                                Button(
                                    onClick = {
                                        scope.launch {
                                            runCatching {
                                                if (remoteApi != null) remoteApi.subscribeFeed(item.feedUrl, item.title)
                                                else localEngine.subscribeFeed(item.feedUrl, item.title, item.category)
                                                results = results.map {
                                                    if (it.feedUrl == item.feedUrl) it.copy(alreadySubscribed = true) else it
                                                }
                                                onSubscribed()
                                            }
                                        }
                                    },
                                    shape = RoundedCornerShape(10.dp),
                                    colors = ButtonDefaults.buttonColors(containerColor = EmeraldDark),
                                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp)
                                ) {
                                    Text("Follow", fontSize = 12.sp)
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
    var serif by remember { mutableStateOf(true) }
    var sepia by remember { mutableStateOf(false) }

    LaunchedEffect(articleId, remoteApi) {
        runCatching {
            val d = if (remoteApi != null) remoteApi.getArticle(articleId) else localEngine.getArticle(articleId)
            detail = d
            if (d != null && !d.isRead) {
                if (remoteApi != null) remoteApi.updateArticleState(articleId, isRead = true)
                else localEngine.updateArticleState(articleId, isRead = true)
            }
        }
    }

    val bgColor = if (sepia) Color(0xFFF8F3E6) else MaterialTheme.colorScheme.background
    val fgColor = if (sepia) Color(0xFF2C251D) else MaterialTheme.colorScheme.onBackground

    Scaffold(
        containerColor = bgColor,
        topBar = {
            TopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(containerColor = bgColor, titleContentColor = fgColor, navigationIconContentColor = fgColor, actionIconContentColor = fgColor),
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                title = { Text(detail?.feedTitle ?: "Reader", fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                actions = {
                    TextButton(onClick = { serif = !serif }) {
                        Text(if (serif) "Serif" else "Sans", color = fgColor, fontSize = 12.sp)
                    }
                    IconButton(onClick = { sepia = !sepia }) {
                        Icon(Icons.Default.MenuBook, contentDescription = "Sepia Mode", tint = if (sepia) AmberAccent else fgColor)
                    }
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
                                if (d.isReadLater) Icons.Default.Bookmark else Icons.Default.BookmarkBorder,
                                contentDescription = "Read Later",
                                tint = if (d.isReadLater) BlueAccent else fgColor
                            )
                        }
                        IconButton(onClick = {
                            context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(d.url)))
                        }) {
                            Icon(Icons.Default.OpenInBrowser, contentDescription = "Open Original", tint = Emerald)
                        }
                    }
                }
            )
        }
    ) { padding ->
        val d = detail
        if (d == null) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Emerald)
            }
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(20.dp)
            ) {
                Text(
                    "${d.feedTitle} • ${d.readingTimeMins} min read • Leo ${d.priorityScore}",
                    style = MaterialTheme.typography.labelMedium,
                    color = Emerald
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    d.title,
                    style = MaterialTheme.typography.headlineSmall,
                    fontFamily = if (serif) FontFamily.Serif else FontFamily.SansSerif,
                    fontWeight = FontWeight.Bold,
                    color = fgColor
                )
                Spacer(Modifier.height(16.dp))

                // Web-matched Leo Intelligence Brief Card
                Card(
                    shape = RoundedCornerShape(14.dp),
                    colors = CardDefaults.cardColors(containerColor = Violet.copy(alpha = 0.12f)),
                    border = BorderStroke(1.dp, Violet.copy(alpha = 0.3f)),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(Modifier.padding(14.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text("LEO INTELLIGENCE BRIEF", fontSize = 11.sp, fontWeight = FontWeight.Bold, color = Violet)
                                d.aiSentiment?.let { sent ->
                                    Surface(color = Violet, shape = RoundedCornerShape(99.dp)) {
                                        Text(sent, color = Color.White, fontSize = 10.sp, fontWeight = FontWeight.Bold, modifier = Modifier.padding(horizontal = 7.dp, vertical = 2.dp))
                                    }
                                }
                            }
                            Button(
                                enabled = !summarizing,
                                onClick = {
                                    scope.launch {
                                        summarizing = true
                                        runCatching {
                                            val res = if (remoteApi != null) remoteApi.generateAiSummary(d.id)
                                            else localEngine.generateLocalSummary(d.id)
                                            detail = d.copy(aiSummary = res.aiSummary, aiSentiment = res.aiSentiment)
                                        }
                                        summarizing = false
                                    }
                                },
                                colors = ButtonDefaults.buttonColors(containerColor = Violet),
                                shape = RoundedCornerShape(10.dp),
                                contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
                                modifier = Modifier.height(30.dp)
                            ) {
                                Text(if (summarizing) "..." else "AI Summary", fontSize = 11.sp)
                            }
                        }
                        if (!d.aiSummary.isNullOrBlank()) {
                            Spacer(Modifier.height(8.dp))
                            Text(d.aiSummary, style = MaterialTheme.typography.bodyMedium, color = fgColor)
                        }
                    }
                }

                Spacer(Modifier.height(20.dp))
                Text(
                    d.contentText.ifBlank { d.summary },
                    fontSize = 17.sp,
                    lineHeight = 27.sp,
                    fontFamily = if (serif) FontFamily.Serif else FontFamily.SansSerif,
                    color = fgColor
                )
            }
        }
    }
}
