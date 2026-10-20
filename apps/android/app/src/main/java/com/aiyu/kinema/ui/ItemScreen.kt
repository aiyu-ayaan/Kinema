package com.aiyu.kinema.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Replay
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.SkipPrevious
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconToggleButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LinearWavyProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.aiyu.kinema.R
import com.aiyu.kinema.core.CastMember
import com.aiyu.kinema.core.ItemCard
import com.aiyu.kinema.core.ItemDetail
import com.aiyu.kinema.core.MediaFile
import kotlinx.coroutines.launch

/** "1h 30m" or "45m"; null without a length. */
@Composable
fun runtime(seconds: Double?): String? {
    if (seconds == null || seconds <= 0) return null
    val m = maxOf(1, Math.round(seconds / 60).toInt())
    return if (m >= 60) stringResource(R.string.format_hours_minutes, m / 60, m % 60) else stringResource(R.string.format_minutes, m)
}

/** A title's page: backdrop with parallax, poster, actions, story, seasons or episodes, cast and file details. */
@Composable
fun ItemScreen(id: Long) {
    val loader = rememberLoad(id) { it.item(id) }
    val nav = LocalNavigator.current
    Box(Modifier.fillMaxSize()) {
        Loaded(loader) { detail -> ItemContent(detail) }
        FilledTonalIconButton(
            nav.back,
            Modifier.statusBarsPadding().padding(12.dp).size(48.dp),
            colors = IconButtonDefaults.filledTonalIconButtonColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.85f)),
        ) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, stringResource(R.string.action_back)) }
    }
}

@Composable
private fun ItemContent(detail: ItemDetail) {
    val item = detail.item
    val api = LocalApi.current
    val nav = LocalNavigator.current
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }
    val c = MaterialTheme.colorScheme
    val list = rememberLazyListState()
    var watched by remember(item) { mutableStateOf(item.watched) }
    val failed = stringResource(R.string.error_generic)

    // What Play starts: the title itself, or the show's / season's next episode.
    val target: ItemCard? = if (item.playable) item else detail.nextUp
    val resume = target?.progress != null

    Box(Modifier.fillMaxSize()) {
        LazyColumn(state = list, modifier = Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 48.dp)) {
            item(key = "header") { Header(detail, list) }
            item(key = "actions") {
                Column(Modifier.padding(horizontal = 20.dp).enterStaggered(1), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                        if (target != null) {
                            Button(
                                { nav.play(target.id, null) },
                                Modifier.weight(1f).heightIn(min = 64.dp),
                                contentPadding = ButtonDefaults.MediumContentPadding,
                            ) {
                                Icon(Icons.Rounded.PlayArrow, null, Modifier.size(28.dp))
                                Spacer(Modifier.width(8.dp))
                                val label = when {
                                    target.id != item.id -> stringResource(R.string.item_play_episode, target.seasonNo ?: 0, target.episodeNo ?: 0)
                                    resume -> stringResource(R.string.action_resume)
                                    else -> stringResource(R.string.action_play)
                                }
                                Text(label, style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            if (resume && target.id == item.id) {
                                FilledTonalIconButton({ nav.play(item.id, 0.0) }, Modifier.size(64.dp)) { Icon(Icons.Rounded.Replay, stringResource(R.string.item_from_start)) }
                            }
                        }
                        FilledIconToggleButton(
                            watched,
                            { on ->
                                watched = on
                                scope.launch {
                                    repository.request { it.setWatched(item.id, on) }
                                        .onSuccess { repository.notifyChanged() }
                                        .onFailure { watched = !on; snackbar.showSnackbar(failed) }
                                }
                            },
                            modifier = Modifier.size(64.dp),
                            shapes = IconButtonDefaults.toggleableShapes(),
                        ) {
                            Icon(if (watched) Icons.Rounded.CheckCircle else Icons.Rounded.Check, stringResource(if (watched) R.string.item_mark_unwatched else R.string.item_mark_watched))
                        }
                    }
                    target?.progress?.let { p ->
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            LinearWavyProgressIndicator(progress = { p }, modifier = Modifier.weight(1f))
                            val left = (target.durationS ?: 0.0) - (target.positionS ?: 0.0)
                            runtime(left)?.let { Text(stringResource(R.string.item_left, it), style = MaterialTheme.typography.labelMedium, color = c.onSurfaceVariant) }
                        }
                    }
                    item.summary?.let { Summary(it) }
                    if (detail.genres.isNotEmpty()) {
                        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            detail.genres.forEach { Tag(it) }
                        }
                    }
                }
            }
            if (item.kind == "show" && detail.children.isNotEmpty()) {
                item(key = "seasons") {
                    Spacer(Modifier.height(28.dp))
                    ShelfRow(stringResource(R.string.item_seasons), detail.children, { nav.open(it.id) })
                }
            }
            if (item.kind == "season" && detail.children.isNotEmpty()) {
                item(key = "episodes-title") {
                    Spacer(Modifier.height(28.dp))
                    SectionTitle(stringResource(R.string.item_episodes))
                    Spacer(Modifier.height(8.dp))
                }
                items(detail.children, key = { it.id }) { ep -> EpisodeRow(ep, { nav.play(ep.id, null) }, { nav.open(ep.id) }) }
            }
            if (item.kind == "episode" && (detail.prevId != null || detail.nextId != null)) {
                item(key = "siblings") {
                    Row(Modifier.padding(horizontal = 20.dp, vertical = 20.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        detail.prevId?.let { FilledTonalButton({ nav.open(it) }, Modifier.weight(1f)) { Icon(Icons.Rounded.SkipPrevious, null); Spacer(Modifier.width(6.dp)); Text(stringResource(R.string.item_previous)) } }
                        detail.nextId?.let { FilledTonalButton({ nav.open(it) }, Modifier.weight(1f)) { Text(stringResource(R.string.item_next)); Spacer(Modifier.width(6.dp)); Icon(Icons.Rounded.SkipNext, null) } }
                    }
                }
            }
            if (detail.cast.isNotEmpty()) {
                item(key = "cast") {
                    Spacer(Modifier.height(28.dp))
                    SectionTitle(stringResource(R.string.item_cast))
                    Spacer(Modifier.height(12.dp))
                    LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        items(detail.cast, key = { it.id }) { CastCard(it, api?.imageUrl(it.image)) }
                    }
                }
            }
            detail.files.firstOrNull()?.let { file -> item(key = "file") { FileInfo(file) } }
        }
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).navigationBarsPadding())
    }
}

/** Backdrop that scrolls away slower than the page and fades, with the poster and title overlapping its foot. */
@Composable
private fun Header(detail: ItemDetail, list: LazyListState) {
    val item = detail.item
    val api = LocalApi.current
    val c = MaterialTheme.colorScheme
    val title = if (item.kind == "season" || item.kind == "episode") item.showTitle ?: item.title else item.title
    Box(Modifier.fillMaxWidth()) {
        Box(
            Modifier.fillMaxWidth().aspectRatio(16f / 11f).graphicsLayer {
                val scrolled = if (list.firstVisibleItemIndex == 0) list.firstVisibleItemScrollOffset.toFloat() else size.height
                translationY = scrolled * 0.5f
                alpha = 1f - (scrolled / size.height).coerceIn(0f, 1f)
            },
        ) {
            // An episode's backdrop is its 300 px still; full width needs the larger copy.
            Artwork(api?.imageUrl(item.backdrop ?: item.poster), api?.frameUrl(item.id), title, Modifier.fillMaxSize(), poster = false, sharp = api?.sharperImageUrl(item.backdrop ?: item.poster, 780))
            Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0f to c.surface.copy(alpha = 0.4f), 0.3f to Color.Transparent, 1f to c.surface)))
        }
        Row(
            Modifier.padding(top = 180.dp).padding(horizontal = 20.dp).enterStaggered(0),
            verticalAlignment = Alignment.Bottom,
            horizontalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            if (item.kind != "episode" && item.kind != "video") {
                Surface(shape = MaterialTheme.shapes.medium, shadowElevation = 8.dp, modifier = Modifier.width(120.dp).aspectRatio(2f / 3f)) {
                    Artwork(api?.imageUrl(item.poster), api?.frameUrl(item.id), title, Modifier.fillMaxSize())
                }
            }
            Column(Modifier.weight(1f).padding(bottom = 4.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                if (item.kind == "season" || item.kind == "episode") {
                    Text(title, style = MaterialTheme.typography.titleMedium, color = c.primary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                Text(
                    if (item.kind == "episode") item.title else if (item.kind == "season") item.title else title,
                    style = MaterialTheme.typography.headlineMedium, maxLines = 3, overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.semantics { heading() },
                )
                val meta = listOfNotNull(
                    subtitleOf(item).takeIf { item.kind != "movie" },
                    item.year?.toString(),
                    runtime(item.durationS ?: detail.files.firstOrNull()?.durationS),
                ).distinct().joinToString(" · ")
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    item.rating?.takeIf { it > 0 }?.let {
                        Icon(Icons.Rounded.Star, null, Modifier.size(16.dp), tint = c.tertiary)
                        Text("%.1f".format(it), style = MaterialTheme.typography.labelLarge, color = c.tertiary)
                    }
                    if (meta.isNotEmpty()) Text(meta, style = MaterialTheme.typography.labelLarge, color = c.onSurfaceVariant)
                }
            }
        }
    }
    Spacer(Modifier.height(20.dp))
}

/** The story, three lines until tapped. */
@Composable
private fun Summary(text: String) {
    var open by rememberSaveable { mutableStateOf(false) }
    Text(
        text,
        style = MaterialTheme.typography.bodyLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = if (open) Int.MAX_VALUE else 4,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.animateContentSize(MaterialTheme.motionScheme.defaultSpatialSpec()).clip(MaterialTheme.shapes.small).clickable { open = !open },
    )
}

@Composable
private fun EpisodeRow(ep: ItemCard, onPlay: () -> Unit, onOpen: () -> Unit) {
    val api = LocalApi.current
    val c = MaterialTheme.colorScheme
    val source = remember { MutableInteractionSource() }
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 4.dp).pressScale(source).clip(MaterialTheme.shapes.large)
            .clickable(source, null, onClick = onOpen).padding(8.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.width(150.dp).aspectRatio(16f / 9f).clip(MaterialTheme.shapes.medium).clickable(onClick = onPlay)) {
            Artwork(api?.imageUrl(ep.backdrop ?: ep.poster), api?.frameUrl(ep.id), ep.title, Modifier.fillMaxSize(), poster = false)
            Box(Modifier.align(Alignment.Center).size(40.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.5f)), contentAlignment = Alignment.Center) {
                Icon(Icons.Rounded.PlayArrow, stringResource(R.string.action_play), tint = Color.White)
            }
            ep.progress?.let { p ->
                Box(Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(4.dp).background(Color.Black.copy(alpha = 0.5f))) {
                    Box(Modifier.fillMaxHeight().fillMaxWidth(p).background(c.primary))
                }
            }
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                listOfNotNull(ep.episodeNo?.let { stringResource(R.string.item_episode_number, it) }, runtime(ep.durationS)).joinToString(" · "),
                style = MaterialTheme.typography.labelMedium, color = c.primary,
            )
            Text(ep.title, style = MaterialTheme.typography.titleMedium, maxLines = 2, overflow = TextOverflow.Ellipsis)
            ep.summary?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = c.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis) }
        }
        if (ep.watched) Icon(Icons.Rounded.CheckCircle, stringResource(R.string.item_watched), tint = c.primary)
    }
}

@Composable
private fun CastCard(member: CastMember, image: String?) {
    val c = MaterialTheme.colorScheme
    Column(Modifier.width(92.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(84.dp).clip(CircleShape).background(c.surfaceContainerHigh), contentAlignment = Alignment.Center) {
            Icon(Icons.Rounded.Person, null, tint = c.onSurfaceVariant, modifier = Modifier.size(40.dp))
            if (image != null) AsyncImage(image, null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        }
        Spacer(Modifier.height(6.dp))
        Text(member.name, style = MaterialTheme.typography.labelLarge, maxLines = 2, overflow = TextOverflow.Ellipsis, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
        if (member.role.isNotBlank()) Text(member.role, style = MaterialTheme.typography.labelSmall, color = c.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** Container, picture and sound of the file, for the curious. */
@Composable
private fun FileInfo(file: MediaFile) {
    val c = MaterialTheme.colorScheme
    val video = file.streams.firstOrNull { it.type == "video" }
    val audio = file.streams.count { it.type == "audio" }
    val subs = file.streams.count { it.type == "subtitle" }
    val parts = listOfNotNull(
        file.container?.substringBefore(',')?.uppercase(),
        video?.let { listOfNotNull(it.codec?.uppercase(), it.height?.let { h -> "${h}p" }).joinToString(" ") }?.ifBlank { null },
        if (audio > 0) pluralStringResource(R.plurals.item_audio_tracks, audio, audio) else null,
        if (subs > 0) pluralStringResource(R.plurals.item_subtitle_tracks, subs, subs) else null,
        "%.1f GB".format(file.size / 1e9).takeIf { file.size > 0 },
    )
    Surface(color = c.surfaceContainerLow, shape = MaterialTheme.shapes.large, modifier = Modifier.padding(horizontal = 16.dp, vertical = 28.dp).fillMaxWidth().widthIn(max = 840.dp)) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(stringResource(R.string.item_file), style = MaterialTheme.typography.titleSmall)
            Text(parts.joinToString(" · "), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
        }
    }
}
