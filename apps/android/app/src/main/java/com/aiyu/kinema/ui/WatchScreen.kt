package com.aiyu.kinema.ui

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
internal fun BoxScope.NextEpisodeCard(nextId: Long?, remaining: Double?, lifted: Boolean, onPlay: () -> Unit) {
    val api = LocalApi.current
    val repository = LocalRepository.current
    var hidden by remember(nextId) { mutableStateOf(false) }
    val visible = nextId != null && remaining != null && remaining in 0.0..UP_NEXT_S && !hidden
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
