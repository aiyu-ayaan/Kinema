package com.aiyu.kinema.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.rounded.GraphicEq
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.LocalContentColor
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import com.aiyu.kinema.core.ItemDetail
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.displayCutoutPadding
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.R
import com.aiyu.kinema.core.ItemCard

/** How long before the end the next episode is offered. */
private const val UP_NEXT_S = 60.0

/**
 * The next episode, offered only in the last minute of this one (YouTube's end card). It slides in from the edge on
 * a spring; the shade over its thumbnail draws back as the countdown runs out, and the video ending plays it.
 */
@Composable
internal fun BoxScope.NextEpisodeCard(nextId: Long?, remaining: Double?, lifted: Boolean, compact: Boolean, onPlay: () -> Unit) {
    val api = LocalApi.current
    val repository = LocalRepository.current
    var hidden by remember(nextId) { mutableStateOf(false) }
    // A video inside the watch page is too short for the card and the controls at once: it waits for them to fade.
    val visible = nextId != null && remaining != null && remaining in 0.0..UP_NEXT_S && !hidden && !(compact && lifted)
    // Fetched once the card is due, so a title watched only part way costs no extra request.
    var next by remember(nextId) { mutableStateOf<ItemCard?>(null) }
    LaunchedEffect(nextId, visible) {
        if (visible && next == null && nextId != null) next = repository.request { it.item(nextId) }.getOrNull()?.item
    }
    val motion = MaterialTheme.motionScheme
    val bottom by animateDpAsState(if (lifted) 108.dp else 24.dp, motion.defaultSpatialSpec(), label = "upNextLift")
    val shade by animateFloatAsState(((remaining ?: 0.0) / UP_NEXT_S).toFloat().coerceIn(0f, 1f), motion.slowEffectsSpec(), label = "upNextShade")
    AnimatedVisibility(
        visible,
        Modifier.align(Alignment.BottomEnd).navigationBarsPadding().displayCutoutPadding().padding(end = 24.dp, bottom = bottom),
        enter = slideInHorizontally(motion.defaultSpatialSpec()) { it / 2 } + scaleIn(motion.defaultSpatialSpec(), initialScale = 0.9f) + fadeIn(motion.defaultEffectsSpec()),
        exit = slideOutHorizontally(motion.fastSpatialSpec()) { it / 2 } + fadeOut(motion.fastEffectsSpec()),
    ) {
        Surface(color = Color.Black.copy(alpha = 0.8f), contentColor = Color.White, shape = MaterialTheme.shapes.large, modifier = Modifier.widthIn(max = 380.dp)) {
            Row(Modifier.padding(10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(Modifier.width(128.dp).aspectRatio(16f / 9f).clip(MaterialTheme.shapes.medium).clickable(onClick = onPlay)) {
                    Artwork(api?.imageUrl(next?.backdrop ?: next?.poster), nextId?.let { api?.frameUrl(it) }, next?.title.orEmpty(), Modifier.fillMaxSize(), poster = false)
                    Box(Modifier.align(Alignment.CenterEnd).fillMaxHeight().fillMaxWidth(shade).background(Color.Black.copy(alpha = 0.5f)))
                    Icon(Icons.Rounded.PlayArrow, null, tint = Color.White, modifier = Modifier.align(Alignment.Center).size(32.dp))
                }
                Column(Modifier.weight(1f, fill = false), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(stringResource(R.string.player_up_next_in, clock(remaining ?: 0.0)), style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
                    Text(
                        next?.let { listOfNotNull(subtitleOf(it), it.title).joinToString(" · ") } ?: stringResource(R.string.player_next),
                        style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.size(4.dp))
                    Button(onPlay, contentPadding = ButtonDefaults.SmallContentPadding, modifier = Modifier.padding(top = 2.dp)) {
                        Icon(Icons.Rounded.SkipNext, null, Modifier.size(ButtonDefaults.SmallIconSize))
                        Spacer(Modifier.width(ButtonDefaults.IconSpacing))
                        Text(stringResource(R.string.player_play_now))
                    }
                }
                IconButton({ hidden = true }, Modifier.align(Alignment.Top).size(36.dp)) {
                    Icon(Icons.Rounded.Close, stringResource(R.string.player_dismiss), Modifier.size(20.dp))
                }
            }
        }
    }
}

/**
 * The watch page on tablets and open foldables, after YouTube's: the video with its title and story under it, and
 * the up-next list beside it in a wide window or under it in a tall one. [video] is the player with its controls.
 */
@Composable
internal fun WatchLayout(detail: ItemDetail, onTracks: (() -> Unit)?, video: @Composable (Modifier) -> Unit) {
    BoxWithConstraints(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).systemBarsPadding().displayCutoutPadding()) {
        if (maxWidth >= 840.dp && maxWidth > maxHeight) {
            val side = if (maxWidth >= 1200.dp) 420.dp else 340.dp
            Row(Modifier.fillMaxSize().padding(start = 24.dp, end = 16.dp, top = 16.dp), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(Modifier.weight(1f).fillMaxHeight().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                    video(Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(MaterialTheme.shapes.large))
                    WatchDetails(detail, onTracks)
                }
                UpNextList(detail, Modifier.width(side).fillMaxHeight(), PaddingValues(bottom = 24.dp))
            }
        } else {
            Column(Modifier.fillMaxSize()) {
                video(Modifier.fillMaxWidth().aspectRatio(16f / 9f))
                UpNextList(detail, Modifier.weight(1f), PaddingValues(horizontal = 16.dp, vertical = 8.dp)) { WatchDetails(detail, onTracks) }
            }
        }
    }
}

/** Title, links and story under the video, with Details and the track picker as tonal pills. */
@Composable
private fun WatchDetails(detail: ItemDetail, onTracks: (() -> Unit)?) {
    val item = detail.item
    val nav = LocalNavigator.current
    val c = MaterialTheme.colorScheme
    Column(Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            if (item.kind == "episode") {
                val show = item.showId
                Text(
                    item.showTitle.orEmpty(), style = MaterialTheme.typography.titleMedium, color = c.primary, maxLines = 1, overflow = TextOverflow.Ellipsis,
                    modifier = if (show != null) Modifier.clip(MaterialTheme.shapes.small).clickable(role = Role.Button) { nav.open(show) } else Modifier,
                )
            }
            Text(item.title, style = MaterialTheme.typography.headlineSmall, maxLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.semantics { heading() })
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (item.kind == "episode" && item.episodeNo != null) EpisodeLinks(item, nav.open, c.primary)
                item.rating?.takeIf { it > 0 }?.let {
                    Icon(Icons.Rounded.Star, null, Modifier.size(16.dp), tint = c.tertiary)
                    Text("%.1f".format(it), style = MaterialTheme.typography.labelLarge, color = c.tertiary)
                }
                val meta = listOfNotNull(item.year?.toString(), runtime(item.durationS ?: detail.files.firstOrNull()?.durationS)).joinToString(" · ")
                if (meta.isNotEmpty()) Text(meta, style = MaterialTheme.typography.labelLarge, color = c.onSurfaceVariant)
            }
        }
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FilledTonalButton({ nav.open(item.id) }) {
                Icon(Icons.Rounded.Info, null, Modifier.size(ButtonDefaults.IconSize))
                Spacer(Modifier.width(ButtonDefaults.IconSpacing))
                Text(stringResource(R.string.action_details))
            }
            onTracks?.let {
                FilledTonalButton(it) {
                    Icon(Icons.Rounded.Subtitles, null, Modifier.size(ButtonDefaults.IconSize))
                    Spacer(Modifier.width(ButtonDefaults.IconSpacing))
                    Text(stringResource(R.string.player_tracks))
                }
            }
        }
        if (item.summary != null || detail.genres.isNotEmpty()) {
            Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.large, modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    item.summary?.let { Summary(it) }
                    if (detail.genres.isNotEmpty()) {
                        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) { detail.genres.forEach { Tag(it) } }
                    }
                }
            }
        }
    }
}

/**
 * What to watch after this: the season's episodes with season chips for a show (the playing one marked), else more
 * titles of the same kind from the library, the ones in progress and unwatched first. [top] scrolls with the list.
 */
@Composable
internal fun UpNextList(
    detail: ItemDetail,
    modifier: Modifier = Modifier,
    contentPadding: PaddingValues = PaddingValues(),
    background: Color = MaterialTheme.colorScheme.surface,
    top: (@Composable () -> Unit)? = null,
) {
    val item = detail.item
    val nav = LocalNavigator.current
    val episode = item.kind == "episode"
    var season by rememberSaveable(item.id) { mutableStateOf(item.parentId) }
    val seasons = rememberLoad(item.showId) { api -> if (episode) item.showId?.let { api.item(it).children.filter { s -> s.kind == "season" } }.orEmpty() else emptyList() }
    val list = rememberLoad(item.id, season) { api ->
        if (episode) season?.let { api.item(it).children }.orEmpty()
        else api.libraryItems(item.libraryId).filter { it.playable && it.id != item.id && it.kind == item.kind }
            .sortedWith(compareByDescending<ItemCard> { it.progress != null }.thenBy { it.watched }.thenByDescending { it.rating ?: 0.0 })
            .take(40)
    }
    val state = rememberLazyListState()
    val loaded = (list.state.value as? Load.Ok)?.value
    // Beside the video the list opens on the playing episode; under it, the details come first.
    LaunchedEffect(loaded != null) {
        if (top == null) loaded?.indexOfFirst { it.id == item.id }?.takeIf { it > 1 }?.let { state.scrollToItem(it) }
    }
    LazyColumn(modifier, state, contentPadding, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        if (top != null) item(key = "top") { top() }
        // Pinned, so the season chips stay in reach down a long season.
        stickyHeader(key = "head") {
            Column(Modifier.fillMaxWidth().background(background).padding(top = 8.dp, bottom = 4.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(stringResource(if (episode) R.string.watch_episodes else R.string.watch_more), style = MaterialTheme.typography.titleMedium, modifier = Modifier.semantics { heading() })
                val all = (seasons.state.value as? Load.Ok)?.value.orEmpty()
                if (all.size > 1) {
                    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        all.forEach { s ->
                            FilterChip(
                                selected = s.id == season,
                                onClick = { season = s.id },
                                label = { Text(s.seasonNo?.let { stringResource(R.string.item_season_no, it) } ?: s.title) },
                                shape = CircleShape,
                            )
                        }
                    }
                }
            }
        }
        when (val s = list.state.value) {
            Load.Loading -> item(key = "loading") { Box(Modifier.fillMaxWidth().padding(24.dp), contentAlignment = Alignment.Center) { LoadingIndicator() } }
            is Load.Failed -> item(key = "error") { Text(errorText(s.error), color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(8.dp)) }
            is Load.Ok -> {
                if (s.value.isEmpty()) item(key = "empty") { Text(stringResource(R.string.watch_nothing), color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(8.dp)) }
                items(s.value, key = { it.id }) { card -> UpNextRow(card, current = card.id == item.id) { nav.play(card.id, null) } }
            }
        }
    }
}

/** A YouTube-style row: 16:9 still with its length and progress, title, then season and episode or year and rating. */
@Composable
private fun UpNextRow(card: ItemCard, current: Boolean, onClick: () -> Unit) {
    val api = LocalApi.current
    val c = MaterialTheme.colorScheme
    Surface(
        onClick = onClick,
        enabled = !current,
        shape = MaterialTheme.shapes.large,
        color = if (current) c.secondaryContainer else Color.Transparent,
        contentColor = if (current) c.onSecondaryContainer else c.onSurface,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Row(Modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.width(152.dp).aspectRatio(16f / 9f).clip(MaterialTheme.shapes.medium)) {
                Artwork(api?.imageUrl(card.backdrop ?: card.poster), api?.frameUrl(card.id), card.title, Modifier.fillMaxSize(), poster = false)
                if (current) {
                    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.55f)), contentAlignment = Alignment.Center) {
                        Icon(Icons.Rounded.GraphicEq, null, tint = Color.White)
                    }
                } else runtime(card.durationS)?.let {
                    Text(
                        it, style = MaterialTheme.typography.labelSmall, color = Color.White,
                        modifier = Modifier.align(Alignment.BottomEnd).padding(4.dp).background(Color.Black.copy(alpha = 0.75f), MaterialTheme.shapes.extraSmall).padding(horizontal = 4.dp, vertical = 1.dp),
                    )
                }
                card.progress?.let { p ->
                    Box(Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(3.dp).background(Color.White.copy(alpha = 0.3f))) {
                        Box(Modifier.fillMaxHeight().fillMaxWidth(p).background(c.primary))
                    }
                }
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                if (current) Text(stringResource(R.string.watch_now_playing), style = MaterialTheme.typography.labelMedium, color = c.primary)
                Text(card.title, style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
                val meta = listOfNotNull(
                    subtitleOf(card),
                    card.rating?.takeIf { it > 0 && card.kind != "episode" }?.let { "★ %.1f".format(it) },
                    stringResource(R.string.item_watched).takeIf { card.watched },
                ).joinToString(" · ")
                if (meta.isNotEmpty()) Text(meta, style = MaterialTheme.typography.bodySmall, color = LocalContentColor.current.copy(alpha = 0.7f), maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** The up-next list over a full-screen video, sliding in from the end edge; a tap outside or back closes it. */
@Composable
internal fun BoxScope.UpNextPanel(open: Boolean, detail: ItemDetail, onClose: () -> Unit) {
    val motion = MaterialTheme.motionScheme
    BackHandler(open, onClose)
    AnimatedVisibility(open, Modifier.fillMaxSize(), enter = fadeIn(motion.defaultEffectsSpec()), exit = fadeOut(motion.fastEffectsSpec())) {
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f)).clickable(interactionSource = null, indication = null, onClick = onClose))
    }
    AnimatedVisibility(
        open,
        Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
        enter = slideInHorizontally(motion.defaultSpatialSpec()) { it } + fadeIn(motion.defaultEffectsSpec()),
        exit = slideOutHorizontally(motion.fastSpatialSpec()) { it } + fadeOut(motion.fastEffectsSpec()),
    ) {
        Surface(
            color = MaterialTheme.colorScheme.surfaceContainer,
            shape = RoundedCornerShape(topStart = 28.dp, bottomStart = 28.dp),
            modifier = Modifier.fillMaxHeight().width(minOf(420.dp, LocalConfiguration.current.screenWidthDp.dp * 0.6f)),
        ) {
            UpNextList(detail, Modifier.fillMaxSize().displayCutoutPadding().navigationBarsPadding(), PaddingValues(16.dp), MaterialTheme.colorScheme.surfaceContainer)
        }
    }
}
