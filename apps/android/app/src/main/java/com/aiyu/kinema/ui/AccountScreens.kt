package com.aiyu.kinema.ui

import android.text.format.DateUtils
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Inbox
import androidx.compose.material.icons.rounded.QueryStats
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.aiyu.kinema.R
import com.aiyu.kinema.core.HistoryEntry
import com.aiyu.kinema.core.MediaRequest
import com.aiyu.kinema.core.RequestCandidate
import com.aiyu.kinema.core.RequestSearch
import com.aiyu.kinema.core.Session
import com.aiyu.kinema.core.WatchStats
import com.aiyu.kinema.core.fillDaily
import com.aiyu.kinema.core.hoursMinutes
import com.aiyu.kinema.core.tmdbPosterKey
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** A pushed page: back arrow, title and a body that manages its own scrolling. */
@Composable
private fun AccountPage(title: String, content: @Composable ColumnScope.() -> Unit) {
    val navigator = LocalNavigator.current
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
        Column(Modifier.widthIn(max = 840.dp).fillMaxSize().statusBarsPadding()) {
            Row(Modifier.padding(horizontal = 4.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                IconButton(navigator.back) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, stringResource(R.string.action_back)) }
                Text(title, style = MaterialTheme.typography.headlineSmall, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f).semantics { heading() })
            }
            content()
        }
    }
}

private val ListPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 96.dp)

@Composable
private fun duration(seconds: Long): String {
    val (h, m) = hoursMinutes(seconds)
    return when {
        h > 0 && m > 0 -> stringResource(R.string.duration_hm, h, m)
        h > 0 -> stringResource(R.string.duration_h, h)
        else -> stringResource(R.string.duration_m, m)
    }
}

@Composable
private fun ago(unixSeconds: Long): String {
    val now = remember { System.currentTimeMillis() }
    return DateUtils.getRelativeTimeSpanString(unixSeconds * 1000, now, DateUtils.MINUTE_IN_MILLIS).toString()
}

// ---- Title requests -------------------------------------------------------------------------------------------

@Composable
private fun statusLabel(status: String) = stringResource(
    when (status) {
        "approved" -> R.string.requests_status_approved
        "declined" -> R.string.requests_status_declined
        "available" -> R.string.requests_status_available
        else -> R.string.requests_status_pending
    },
)

@Composable
private fun kindLabel(kind: String) = stringResource(if (kind == "show") R.string.requests_kind_show else R.string.requests_kind_movie)

/** Settings › Requests: the titles this profile asked for, and a sheet to ask for another. */
@Composable
fun RequestsScreen() {
    val repository = LocalRepository.current
    val api = LocalApi.current
    val scope = rememberCoroutineScope()
    val session by repository.session.collectAsStateWithLifecycle()
    val me = (session as? Session.SignedIn)?.user?.id
    val loader = rememberLoad { it.requests() }
    var composing by rememberSaveable { mutableStateOf(false) }
    var error by remember { mutableStateOf<Throwable?>(null) }
    val c = MaterialTheme.colorScheme

    AccountPage(stringResource(R.string.requests_title)) {
        Box(Modifier.weight(1f)) {
            Loaded(loader) { all ->
                // An administrator's list holds everyone's requests; this page is about one's own.
                val mine = all.filter { me == null || it.userId == me }
                LazyColumn(contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    item(key = "new") {
                        Button({ composing = true }, Modifier.fillMaxWidth().height(56.dp)) {
                            Icon(Icons.Rounded.Add, null)
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(R.string.requests_new))
                        }
                    }
                    error?.let { e -> item(key = "error") { Text(errorText(e), color = c.error) } }
                    if (mine.isEmpty()) {
                        item(key = "empty") {
                            MessageState(Icons.Rounded.Inbox, stringResource(R.string.requests_empty), stringResource(R.string.requests_empty_body), Modifier.height(320.dp))
                        }
                    }
                    items(mine, key = { it.id }) { r ->
                        RequestRow(r, api?.imageUrl(r.poster), onWithdraw = {
                            scope.launch {
                                error = repository.request { it.withdrawRequest(r.id) }.exceptionOrNull()
                                loader.reload()
                            }
                        })
                    }
                }
            }
        }
    }

    if (composing) RequestSheet(onDismiss = { composing = false }, onSent = { composing = false; loader.reload() })
}

@Composable
private fun RequestRow(r: MediaRequest, image: String?, onWithdraw: () -> Unit) {
    val c = MaterialTheme.colorScheme
    Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth()) {
        Row(Modifier.padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Artwork(image, null, r.title, Modifier.width(64.dp).aspectRatio(2f / 3f).clip(MaterialTheme.shapes.medium))
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(r.title, style = MaterialTheme.typography.titleMedium, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Text(listOfNotNull(kindLabel(r.kind), r.year?.toString(), ago(r.createdAt)).joinToString(" · "), style = MaterialTheme.typography.bodySmall, color = c.onSurfaceVariant)
                val good = r.status == "approved" || r.status == "available"
                when {
                    r.status == "declined" -> Tag(statusLabel(r.status), container = c.errorContainer, content = c.onErrorContainer)
                    good -> Tag(statusLabel(r.status), container = c.primaryContainer, content = c.onPrimaryContainer)
                    else -> Tag(statusLabel(r.status))
                }
                r.note?.let { Text(it, style = MaterialTheme.typography.bodyMedium, maxLines = 3, overflow = TextOverflow.Ellipsis) }
                r.adminNote?.let { Text(stringResource(R.string.requests_admin_note, it), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant) }
                if (r.pending) TextButton(onWithdraw) { Text(stringResource(R.string.requests_withdraw), color = c.error) }
            }
        }
    }
}

/** Movie or show, a debounced TMDB search, and one tap to pick. Without a TMDB key the typed title is sent as it is. */
@Composable
private fun RequestSheet(onDismiss: () -> Unit, onSent: () -> Unit) {
    val repository = LocalRepository.current
    val api = LocalApi.current
    val scope = rememberCoroutineScope()
    val sheet = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var kind by rememberSaveable { mutableStateOf("movie") }
    var query by rememberSaveable { mutableStateOf("") }
    var note by rememberSaveable { mutableStateOf("") }
    var search by remember { mutableStateOf<RequestSearch?>(null) }
    var searching by remember { mutableStateOf(false) }
    var picked by remember { mutableStateOf<RequestCandidate?>(null) }
    var error by remember { mutableStateOf<Throwable?>(null) }
    var busy by remember { mutableStateOf(false) }
    val c = MaterialTheme.colorScheme

    LaunchedEffect(query, kind) {
        picked = null
        if (query.isBlank()) { search = null; return@LaunchedEffect }
        delay(400)
        searching = true
        try {
            repository.request { it.requestSearch(query.trim(), kind) }.fold({ search = it; error = null }, { error = it })
        } finally {
            searching = false
        }
    }

    val typedOnly = search?.tmdb == false
    val canSend = !busy && (picked != null || (typedOnly && query.isNotBlank()))

    ModalBottomSheet(onDismiss, sheetState = sheet) {
        Column(
            Modifier.fillMaxWidth().padding(horizontal = 24.dp).padding(bottom = 24.dp).imePadding().navigationBarsPadding().verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(stringResource(R.string.requests_new), style = MaterialTheme.typography.headlineSmall, modifier = Modifier.semantics { heading() })
            val kinds = listOf("movie" to R.string.requests_kind_movie, "show" to R.string.requests_kind_show)
            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                kinds.forEachIndexed { i, (key, label) ->
                    SegmentedButton(kind == key, { kind = key }, SegmentedButtonDefaults.itemShape(i, kinds.size)) { Text(stringResource(label)) }
                }
            }
            OutlinedTextField(
                query, { query = it; error = null },
                label = { Text(stringResource(R.string.requests_search)) },
                leadingIcon = { Icon(Icons.Rounded.Search, null) },
                trailingIcon = { if (searching) LoadingIndicator(Modifier.size(32.dp)) },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                shape = MaterialTheme.shapes.large,
                modifier = Modifier.fillMaxWidth(),
            )
            val found = search
            when {
                found == null -> Unit
                typedOnly -> Text(stringResource(R.string.requests_no_tmdb), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
                found.results.isEmpty() && !searching -> Text(stringResource(R.string.requests_no_results), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
                else -> found.results.take(8).forEach { cand ->
                    CandidateRow(cand, api?.imageUrl(tmdbPosterKey(cand.poster)), picked?.tmdbId == cand.tmdbId) { picked = cand }
                }
            }
            if (picked != null || typedOnly) {
                OutlinedTextField(
                    note, { note = it },
                    label = { Text(stringResource(R.string.requests_note)) },
                    minLines = 2,
                    shape = MaterialTheme.shapes.large,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            error?.let { Text(errorText(it), color = c.error) }
            Button(
                {
                    busy = true
                    error = null
                    scope.launch {
                        val p = picked
                        repository.request {
                            it.createRequest(kind, p?.title ?: query.trim(), p?.year, p?.tmdbId, p?.poster, p?.overview, note.trim().ifBlank { null })
                        }.fold({ onSent() }, { error = it })
                        busy = false
                    }
                },
                enabled = canSend,
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                if (busy) LoadingIndicator(Modifier.size(24.dp), color = c.onPrimary) else Text(stringResource(R.string.requests_submit))
            }
        }
    }
}

@Composable
private fun CandidateRow(cand: RequestCandidate, image: String?, selected: Boolean, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    Surface(
        color = if (selected) c.primaryContainer else c.surfaceContainerHigh,
        contentColor = if (selected) c.onPrimaryContainer else c.onSurface,
        shape = MaterialTheme.shapes.large,
        modifier = Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).clickable(onClick = onClick),
    ) {
        Row(Modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Artwork(image, null, cand.title, Modifier.width(48.dp).aspectRatio(2f / 3f).clip(MaterialTheme.shapes.small), poster = false)
            Column(Modifier.weight(1f)) {
                Text(cand.title, style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
                cand.year?.let { Text(it.toString(), style = MaterialTheme.typography.bodySmall) }
                cand.overview?.takeIf { it.isNotBlank() }?.let { Text(it, style = MaterialTheme.typography.bodySmall, maxLines = 2, overflow = TextOverflow.Ellipsis) }
            }
        }
    }
}

// ---- Watch activity -------------------------------------------------------------------------------------------

/** Settings › Watch activity: this profile's watch time, a 30-day chart, most watched and recent history. */
@Composable
fun ActivityScreen() {
    val loader = rememberLoad { it.myStats() }
    AccountPage(stringResource(R.string.activity_title)) {
        Box(Modifier.weight(1f)) { Loaded(loader) { ActivityBody(it) } }
    }
}

@Composable
private fun ActivityBody(stats: WatchStats) {
    if (stats.totals.titles == 0L && stats.history.isEmpty()) {
        MessageState(Icons.Rounded.QueryStats, stringResource(R.string.activity_empty), stringResource(R.string.activity_empty_body))
        return
    }
    val navigator = LocalNavigator.current
    val open = rememberOpen()
    val api = LocalApi.current
    val t = stats.totals
    val tiles = listOf(
        stringResource(R.string.activity_total) to duration(t.totalS),
        stringResource(R.string.activity_week) to duration(t.last7dS),
        stringResource(R.string.activity_month) to duration(t.last30dS),
        stringResource(R.string.activity_titles) to t.titles.toString(),
        stringResource(R.string.activity_completed) to t.completed.toString(),
    )
    LazyColumn(contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        items(tiles.chunked(2)) { pair ->
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                pair.forEach { (label, value) -> StatTile(label, value, Modifier.weight(1f)) }
                if (pair.size == 1) Spacer(Modifier.weight(1f))
            }
        }
        item(key = "daily") { DailyChart(stats) }
        if (stats.topTitles.isNotEmpty()) {
            item(key = "top-h") { SectionTitle(stringResource(R.string.activity_top_titles)) }
            items(stats.topTitles, key = { "top-${it.itemId}" }) { top ->
                ActivityRow(api?.imageUrl(top.poster), top.title, kindLabel(if (top.kind == "show") "show" else "movie"), duration(top.seconds)) { navigator.open(top.itemId) }
            }
        }
        if (stats.topGenres.isNotEmpty()) {
            item(key = "genre-h") { SectionTitle(stringResource(R.string.activity_top_genres)) }
            item(key = "genres") {
                val peak = stats.topGenres.maxOf { it.seconds }.coerceAtLeast(1)
                Surface(color = MaterialTheme.colorScheme.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        stats.topGenres.forEach { g ->
                            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                Row {
                                    Text(g.genre, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
                                    Text(duration(g.seconds), style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                Box(Modifier.fillMaxWidth().height(8.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surfaceContainerHighest)) {
                                    Box(Modifier.fillMaxWidth(g.seconds.toFloat() / peak).fillMaxHeight().background(MaterialTheme.colorScheme.primary, CircleShape))
                                }
                            }
                        }
                    }
                }
            }
        }
        if (stats.history.isNotEmpty()) {
            item(key = "history-h") { SectionTitle(stringResource(R.string.activity_history)) }
            items(stats.history, key = { "h-${it.item.id}" }) { h -> HistoryRow(h, api?.imageUrl(h.item.poster ?: h.item.backdrop)) { open(h.item) } }
        }
    }
}

@Composable
private fun StatTile(label: String, value: String, modifier: Modifier = Modifier) {
    val c = MaterialTheme.colorScheme
    Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(value, style = MaterialTheme.typography.headlineSmall, maxLines = 1)
            Text(label, style = MaterialTheme.typography.labelMedium, color = c.onSurfaceVariant)
        }
    }
}

/** One bar per day of the last 30; a day without play is a dot. No chart library: a Canvas is enough. */
@Composable
private fun DailyChart(stats: WatchStats) {
    val c = MaterialTheme.colorScheme
    val today = remember { System.currentTimeMillis() / 1000 / 86400 }
    val values = remember(stats.daily, today) { fillDaily(stats.daily, today) }
    val peak = values.max()
    val summary = stringResource(R.string.activity_daily_peak, duration(peak))
    Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(stringResource(R.string.activity_daily), style = MaterialTheme.typography.titleMedium, modifier = Modifier.weight(1f).semantics { heading() })
                Text(summary, style = MaterialTheme.typography.labelMedium, color = c.onSurfaceVariant)
            }
            val bar = c.primary
            val idle = c.surfaceContainerHighest
            Canvas(Modifier.fillMaxWidth().height(120.dp).semantics { contentDescription = summary }) {
                val gap = 3.dp.toPx()
                val w = (size.width - gap * (values.size - 1)) / values.size
                val max = peak.coerceAtLeast(1).toFloat()
                values.forEachIndexed { i, v ->
                    val h = if (v == 0L) w else (v / max * size.height).coerceAtLeast(w)
                    drawRoundRect(if (v == 0L) idle else bar, Offset(i * (w + gap), size.height - h), Size(w, h), CornerRadius(w / 2))
                }
            }
        }
    }
}

@Composable
private fun ActivityRow(image: String?, title: String, subtitle: String?, trailing: String, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth().clip(MaterialTheme.shapes.extraLarge).clickable(onClick = onClick)) {
        Row(Modifier.padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Artwork(image, null, title, Modifier.width(48.dp).aspectRatio(2f / 3f).clip(MaterialTheme.shapes.small), poster = false)
            Column(Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.titleSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (!subtitle.isNullOrBlank()) Text(subtitle, style = MaterialTheme.typography.bodySmall, color = c.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(trailing, style = MaterialTheme.typography.labelMedium, color = c.onSurfaceVariant)
        }
    }
}

@Composable
private fun HistoryRow(h: HistoryEntry, image: String?, onClick: () -> Unit) {
    val item = h.item
    val sub = listOfNotNull(if (item.showTitle != null) item.title else null, subtitleOf(item)).joinToString(" · ")
    ActivityRow(image, item.showTitle ?: item.title, sub, ago(h.lastPlayedAt), onClick)
}
