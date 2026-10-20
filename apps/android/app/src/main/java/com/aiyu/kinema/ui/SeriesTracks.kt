package com.aiyu.kinema.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.R
import com.aiyu.kinema.core.SeriesTracks
import kotlinx.coroutines.launch

/** Admin choice of audio and subtitle track for every episode of a show; each pick is saved at once. */
@Composable
fun SeriesTracksButton(showId: Long, tracks: SeriesTracks, onError: () -> Unit) {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    var open by remember { mutableStateOf(false) }
    var audio by remember(tracks) { mutableStateOf(tracks.audio) }
    var subtitle by remember(tracks) { mutableStateOf(tracks.subtitle) }
    val audios = remember(tracks) { tracks.streams.filter { it.type == "audio" } }
    val subs = remember(tracks) { tracks.streams.filter { it.type == "subtitle" } }

    fun save(a: Int?, s: Int?) {
        val (oldA, oldS) = audio to subtitle
        audio = a
        subtitle = s
        scope.launch { repository.request { it.setSeriesTracks(showId, a, s) }.onFailure { audio = oldA; subtitle = oldS; onError() } }
    }

    FilledTonalIconButton({ open = true }, Modifier.size(64.dp)) { Icon(Icons.Rounded.Translate, stringResource(R.string.series_tracks)) }
    if (!open) return
    ModalBottomSheet({ open = false }, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)) {
        Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp).padding(bottom = 24.dp)) {
            val default = stringResource(R.string.series_default)
            Text(
                stringResource(if (tracks.streams.isEmpty()) R.string.series_no_streams else R.string.series_hint),
                style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(8.dp),
            )
            if (tracks.streams.isEmpty()) return@Column
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(Modifier.weight(1f)) {
                    Text(stringResource(R.string.player_audio), style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(8.dp))
                    TrackRow(default, audio == null) { save(null, subtitle) }
                    audios.forEachIndexed { i, a -> TrackRow(trackLabel(a, i + 1), audio == i) { save(i, subtitle) } }
                }
                Column(Modifier.weight(1f)) {
                    Text(stringResource(R.string.player_subtitles), style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(8.dp))
                    TrackRow(default, subtitle == null) { save(audio, null) }
                    TrackRow(stringResource(R.string.player_subtitles_off), subtitle == -1) { save(audio, -1) }
                    subs.forEachIndexed { i, s ->
                        TrackRow(trackLabel(s, i + 1), subtitle == i, enabled = s.isTextSubtitle, note = if (s.isTextSubtitle) null else stringResource(R.string.player_subtitle_image)) { save(audio, i) }
                    }
                }
            }
        }
    }
}
