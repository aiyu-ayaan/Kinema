package com.aiyu.kinema.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.VideoLibrary
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.SearchOff
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.VideoLibrary
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.compose.foundation.clickable
import androidx.compose.material.icons.rounded.ExpandLess
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.ModalBottomSheet
import com.aiyu.kinema.R
import com.aiyu.kinema.core.Home
import com.aiyu.kinema.core.ItemCard
import com.aiyu.kinema.core.KinemaApi
import com.aiyu.kinema.core.Library
import com.aiyu.kinema.ui.theme.rememberReducedMotion
import kotlin.math.absoluteValue
import kotlinx.coroutines.delay

/** A server read: loading, the value, or why it failed. A reload keeps the last value on screen. */
sealed interface Load<out T> {
    data object Loading : Load<Nothing>
    data class Ok<T>(val value: T) : Load<T>
    data class Failed(val error: Throwable) : Load<Nothing>
}

class Loader<T>(val state: State<Load<T>>, val refreshing: State<Boolean>, val reload: () -> Unit)

/** Runs [block] against the current server; again when [keys], the address in use or the watch state change. */
@Composable
fun <T> rememberLoad(vararg keys: Any?, block: suspend (KinemaApi) -> T): Loader<T> {
    val repository = LocalRepository.current
    val api = LocalApi.current
    var run by remember { mutableIntStateOf(0) }
    val refreshing = remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { repository.changes.collect { run++ } }
    // Back in the app: what was watched on the web or another phone meanwhile shows up.
    var paused by remember { mutableStateOf(false) }
    LifecycleEventEffect(Lifecycle.Event.ON_PAUSE) { paused = true }
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) { if (paused) { paused = false; run++ } }
    val state = produceState<Load<T>>(Load.Loading, api, run, *keys) {
        refreshing.value = true
        val result = repository.request(block)
        value = result.fold({ Load.Ok(it) }, { if (value is Load.Ok) value else Load.Failed(it) })
        refreshing.value = false
    }
    return remember(state) { Loader(state, refreshing) { run++ } }
}

/** Shows the loaded value, a spinner the first time, or the error with a retry. */
@Composable
fun <T> Loaded(loader: Loader<T>, content: @Composable (T) -> Unit) {
    when (val s = loader.state.value) {
        Load.Loading -> LoadingBox()
        is Load.Failed -> ErrorState(errorText(s.error), loader.reload)
        is Load.Ok -> content(s.value)
    }
}

@Composable
fun MainScreen() {
    var tab by rememberSaveable { mutableIntStateOf(0) }
    val tabs = listOf(
        NavTab(stringResource(R.string.tab_home), Icons.Outlined.Home, Icons.Rounded.Home),
        NavTab(stringResource(R.string.tab_library), Icons.Outlined.VideoLibrary, Icons.Rounded.VideoLibrary),
        NavTab(stringResource(R.string.tab_search), Icons.Outlined.Search, Icons.Rounded.Search),
        NavTab(stringResource(R.string.tab_settings), Icons.Outlined.Settings, Icons.Rounded.Settings),
    )
    // Back from another tab returns home first, previewed by the gesture; from home it leaves the app.
    val back = rememberPredictiveBack(tab != 0) { tab = 0 }
    Box(Modifier.fillMaxSize().predictiveBackEffect { back.value }) {
        // Tapping Library while already on it opens the library switcher.
        var switcher by rememberSaveable { mutableIntStateOf(0) }
        FloatingNavScaffold(tabs, tab, { if (it == 1 && tab == 1) switcher++ else tab = it }) { current ->
            when (current) {
                0 -> HomeTab()
                1 -> LibraryTab(switcher)
                2 -> SearchTab()
                else -> SettingsTab()
            }
        }
        // Content scrolls edge to edge; a scrim keeps the status bar icons readable over it.
        Box(
            Modifier.fillMaxWidth().windowInsetsTopHeight(WindowInsets.statusBars)
                .background(Brush.verticalGradient(listOf(MaterialTheme.colorScheme.surface.copy(alpha = 0.9f), MaterialTheme.colorScheme.surface.copy(alpha = 0.6f)))),
        )
    }
}

/** Opens a card: playable titles show their page too, where Play is one tap away with the details. */
@Composable
fun rememberOpen(): (ItemCard) -> Unit {
    val nav = LocalNavigator.current
    return remember(nav) { { item: ItemCard -> nav.open(item.id) } }
}

private enum class FeatureReason { CONTINUE, NEW }

/** Up to 5 hero slides: what you were watching, then the newest titles with art first (web `pickFeatures`). */
private fun pickFeatures(home: Home): List<Pair<ItemCard, FeatureReason>> {
    val seen = HashSet<Long>()
    val out = ArrayList<Pair<ItemCard, FeatureReason>>()
    fun add(item: ItemCard, reason: FeatureReason) {
        if (out.size < 5 && seen.add(item.id)) out += item to reason
    }
    home.continueWatching.take(2).forEach { add(it, FeatureReason.CONTINUE) }
    home.shelves.filter { it.kind != "episodes" }.flatMap { it.items }
        .sortedWith(compareByDescending<ItemCard> { it.backdrop != null }.thenByDescending { it.addedAt })
        .forEach { add(it, FeatureReason.NEW) }
    return out
}

@Composable
private fun HomeTab() {
    val loader = rememberLoad { it.home() }
    val open = rememberOpen()
    val bottom = LocalBottomBarInset.current
    Loaded(loader) { home ->
        val features = remember(home) { pickFeatures(home) }
        if (features.isEmpty()) {
            MessageState(Icons.Rounded.Movie, stringResource(R.string.home_empty_title), stringResource(R.string.home_empty_body), Modifier.statusBarsPadding())
            return@Loaded
        }
        val multiple = home.shelves.map { it.library.id }.toSet().size > 1
        PullToRefreshBox(loader.refreshing.value, loader.reload) {
            LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = bottom + 32.dp), verticalArrangement = Arrangement.spacedBy(28.dp)) {
                item(key = "hero") { Hero(features) }
                item(key = "continue") { ShelfRow(stringResource(R.string.home_continue), home.continueWatching, open, wide = true) }
                items(home.shelves, key = { "${it.library.id}-${it.kind}" }) { shelf ->
                    val title = stringResource(
                        when (shelf.kind) {
                            "movies" -> R.string.home_shelf_movies
                            "shows" -> R.string.home_shelf_shows
                            else -> R.string.home_shelf_episodes
                        },
                    )
                    ShelfRow(if (multiple) "$title · ${shelf.library.name}" else title, shelf.items, open, wide = shelf.kind == "episodes")
                }
            }
        }
    }
}

/** Full-bleed backdrops with parallax, auto-advancing every 7s until touched. Play and details on each slide. */
@Composable
private fun Hero(features: List<Pair<ItemCard, FeatureReason>>) {
    val pager = rememberPagerState { features.size }
    val reduced = rememberReducedMotion()
    val isDragged by pager.interactionSource.collectIsDraggedAsState()
    var touched by remember { mutableStateOf(false) }
    LaunchedEffect(isDragged) { if (isDragged) touched = true }
    LaunchedEffect(touched, reduced, features.size) {
        if (touched || reduced || features.size < 2) return@LaunchedEffect
        while (true) {
            delay(7_000)
            if (!pager.isScrollInProgress) {
                pager.animateScrollToPage((pager.currentPage + 1) % features.size)
            }
        }
    }
    Box(Modifier.fillMaxWidth().aspectRatio(0.78f).heightIn(max = 620.dp)) {
        HorizontalPager(pager, Modifier.fillMaxSize(), key = { features[it].first.id }) { page ->
            HeroSlide(features[page].first, features[page].second, pager, page)
        }
        PagerDots(pager, Modifier.align(Alignment.BottomCenter).padding(bottom = 4.dp))
    }
}

@Composable
private fun HeroSlide(item: ItemCard, reason: FeatureReason, pager: PagerState, page: Int) {
    val api = LocalApi.current
    val nav = LocalNavigator.current
    val c = MaterialTheme.colorScheme
    val offset = { (pager.currentPage - page + pager.currentPageOffsetFraction) }
    Box(Modifier.fillMaxSize().clip(MaterialTheme.shapes.extraLarge.copy(topStart = androidx.compose.foundation.shape.CornerSize(0.dp), topEnd = androidx.compose.foundation.shape.CornerSize(0.dp)))) {
        Artwork(
            image = api?.imageUrl(item.backdrop ?: item.poster),
            frame = api?.frameUrl(item.id),
            title = item.showTitle ?: item.title,
            poster = false,
            modifier = Modifier.fillMaxSize().graphicsLayer {
                // Parallax: the art moves at half the page speed and settles with a slight zoom.
                translationX = offset() * size.width * 0.5f
                val s = 1.08f - 0.08f * (1f - offset().absoluteValue.coerceIn(0f, 1f))
                scaleX = s; scaleY = s
            },
        )
        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0f to c.surface.copy(alpha = 0.55f), 0.25f to Color.Transparent, 0.55f to Color.Transparent, 1f to c.surface)))
        Column(
            Modifier.align(Alignment.BottomStart).fillMaxWidth().padding(horizontal = 20.dp).padding(bottom = 28.dp)
                .graphicsLayer { alpha = 1f - offset().absoluteValue.coerceIn(0f, 1f); translationX = offset() * -size.width * 0.15f },
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Tag(stringResource(if (reason == FeatureReason.CONTINUE) R.string.home_hero_continue else R.string.home_hero_new), container = c.primaryContainer, content = c.onPrimaryContainer)
            Text(item.showTitle ?: item.title, style = MaterialTheme.typography.displaySmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
            val meta = listOfNotNull(subtitleOf(item), if (item.kind == "episode") item.title else null).joinToString(" · ")
            if (meta.isNotEmpty()) Text(meta, style = MaterialTheme.typography.titleSmall, color = c.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            item.summary?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis) }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                if (item.playable) {
                    Button({ nav.play(item.id, null) }, contentPadding = ButtonDefaults.MediumContentPadding, modifier = Modifier.heightIn(min = 56.dp)) {
                        Icon(Icons.Rounded.PlayArrow, null)
                        Spacer(Modifier.width(8.dp))
                        Text(stringResource(if (item.progress != null) R.string.action_resume else R.string.action_play), style = MaterialTheme.typography.titleMedium)
                    }
                }
                FilledTonalButton({ nav.open(item.id) }, modifier = Modifier.heightIn(min = 56.dp)) {
                    Icon(Icons.Rounded.Info, null)
                    Spacer(Modifier.width(8.dp))
                    Text(stringResource(R.string.action_details))
                }
            }
        }
    }
}

/** Page dots; the current one stretches into a pill with a spring. */
@Composable
private fun PagerDots(pager: PagerState, modifier: Modifier = Modifier) {
    if (pager.pageCount < 2) return
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        repeat(pager.pageCount) { i ->
            val selected = pager.currentPage == i
            val width by animateDpAsState(if (selected) 22.dp else 8.dp, MaterialTheme.motionScheme.fastSpatialSpec(), label = "dot")
            val color by animateColorAsState(if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f), label = "dotColor")
            Box(Modifier.height(8.dp).width(width).clip(CircleShape).background(color))
        }
    }
}

@Composable
private fun TabHeader(title: String) {
    Text(
        title, style = MaterialTheme.typography.headlineLarge,
        modifier = Modifier.statusBarsPadding().padding(start = 20.dp, end = 20.dp, top = 16.dp, bottom = 8.dp).semantics { heading() },
    )
}

@Composable
private fun LibraryTab(switcher: Int) {
    val libraries = rememberLoad { it.libraries() }
    Column(Modifier.fillMaxSize()) {
        TabHeader(stringResource(R.string.tab_library))
        Loaded(libraries) { libs ->
            if (libs.isEmpty()) {
                MessageState(Icons.Rounded.VideoLibrary, stringResource(R.string.library_none_title), stringResource(R.string.library_none_body))
                return@Loaded
            }
            var selected by rememberSaveable { mutableStateOf(libs.first().id) }
            val open = rememberOpen()
            val shows = rememberLoad(libs) { api -> libs.filter { it.showCount > 0 }.associate { l -> l.id to api.libraryItems(l.id).filter { it.kind == "show" } } }
            val lib = libs.firstOrNull { it.id == selected } ?: libs.first()
            var sheet by rememberSaveable { mutableStateOf(false) }
            var seen by rememberSaveable { mutableIntStateOf(switcher) }
            if (switcher != seen) { seen = switcher; sheet = true }
            if (sheet) {
                LibrarySheet(libs, shows.state.value.let { (it as? Load.Ok)?.value.orEmpty() }, lib.id, { selected = it }, open) { sheet = false }
            }
            LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                items(libs, key = { it.id }) { l ->
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        FilterChip(
                            selected = l.id == lib.id,
                            onClick = { selected = l.id },
                            label = { Text(l.name) },
                            leadingIcon = if (l.id == lib.id) { { Icon(Icons.Rounded.VideoLibrary, null, Modifier.size(FilterChipDefaults.IconSize)) } } else null,
                            shape = CircleShape,
                        )
                        // Shows open their own page, as in the web sidebar.
                        (shows.state.value as? Load.Ok)?.value?.get(l.id).orEmpty().forEach { show ->
                            FilterChip(selected = false, onClick = { open(show) }, label = { Text(show.title) }, shape = CircleShape)
                        }
                    }
                }
            }
            Spacer(Modifier.height(8.dp))
            val items = rememberLoad(lib.id) { it.libraryItems(lib.id) }
            val motion = MaterialTheme.motionScheme
            AnimatedContent(lib.id, label = "library", transitionSpec = { fadeIn(motion.defaultEffectsSpec()).togetherWith(fadeOut(motion.fastEffectsSpec())) }) {
                Loaded(items) { list ->
                    if (list.isEmpty()) MessageState(Icons.Rounded.Movie, stringResource(R.string.library_empty_title), stringResource(R.string.library_empty_body))
                    else {
                        // Like the web sidebar: a mixed library lists its movies, its shows hang below it as chips.
                        val mixed = lib.showCount > 0 && lib.showCount < lib.itemCount
                        val shown = if (mixed) list.filter { it.kind != "show" } else list
                        PullToRefreshBox(items.refreshing.value, items.reload) { PosterGrid(shown, wide = lib.kind == "other") }
                    }
                }
            }
        }
    }
}

/** Library switcher: each library collapses to hide the shows under it, like the web sidebar. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LibrarySheet(libs: List<Library>, shows: Map<Long, List<ItemCard>>, current: Long, onPick: (Long) -> Unit, onShow: (ItemCard) -> Unit, onDismiss: () -> Unit) {
    var collapsed by rememberSaveable { mutableStateOf(emptyList<Long>()) }
    ModalBottomSheet(onDismiss) {
        LazyColumn(contentPadding = PaddingValues(bottom = 24.dp)) {
            libs.forEach { l ->
                val children = shows[l.id].orEmpty()
                val open = l.id !in collapsed
                item(key = "lib${l.id}") {
                    ListItem(
                        headlineContent = { Text(l.name) },
                        leadingContent = { Icon(Icons.Rounded.VideoLibrary, null) },
                        trailingContent = if (children.isEmpty()) null else {
                            {
                                IconButton({ collapsed = if (open) collapsed + l.id else collapsed - l.id }) {
                                    Icon(if (open) Icons.Rounded.ExpandLess else Icons.Rounded.ExpandMore, null)
                                }
                            }
                        },
                        colors = ListItemDefaults.colors(containerColor = if (l.id == current) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent),
                        modifier = Modifier.clickable { onPick(l.id); onDismiss() },
                    )
                }
                if (open) items(children, key = { "show${it.id}" }) { show ->
                    ListItem(
                        headlineContent = { Text(show.title, maxLines = 1) },
                        modifier = Modifier.padding(start = 40.dp).clickable { onShow(show); onDismiss() },
                    )
                }
            }
        }
    }
}

@Composable
fun PosterGrid(items: List<ItemCard>, wide: Boolean = false, header: (@Composable () -> Unit)? = null) {
    val open = rememberOpen()
    val bottom = LocalBottomBarInset.current
    LazyVerticalGrid(
        GridCells.Adaptive(if (wide) 220.dp else 112.dp),
        Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = bottom + 32.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        if (header != null) item(span = { GridItemSpan(maxLineSpan) }) { header() }
        items(items, key = { it.id }) { PosterCard(it, { open(it) }, Modifier.animateItem(), wide = wide) }
    }
}

@Composable
private fun SearchTab() {
    var query by rememberSaveable { mutableStateOf("") }
    var debounced by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(query) {
        delay(300)
        debounced = query.trim()
    }
    Column(Modifier.fillMaxSize()) {
        TabHeader(stringResource(R.string.tab_search))
        TextField(
            query, { query = it },
            placeholder = { Text(stringResource(R.string.search_hint)) },
            leadingIcon = { Icon(Icons.Rounded.Search, null) },
            trailingIcon = { if (query.isNotEmpty()) IconButton({ query = "" }) { Icon(Icons.Rounded.Close, stringResource(R.string.search_clear)) } },
            singleLine = true,
            shape = CircleShape,
            colors = TextFieldDefaults.colors(focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            keyboardActions = KeyboardActions(onSearch = { debounced = query.trim() }),
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp).heightIn(min = 56.dp),
        )
        Spacer(Modifier.height(8.dp))
        if (debounced.length < 2) {
            MessageState(Icons.Rounded.Search, stringResource(R.string.search_idle_title), stringResource(R.string.search_idle_body))
            return@Column
        }
        val results = rememberLoad(debounced) { it.search(debounced) }
        Loaded(results) { list ->
            if (list.isEmpty()) MessageState(Icons.Rounded.SearchOff, stringResource(R.string.search_none_title), stringResource(R.string.search_none_body, debounced))
            else PosterGrid(list)
        }
    }
}
