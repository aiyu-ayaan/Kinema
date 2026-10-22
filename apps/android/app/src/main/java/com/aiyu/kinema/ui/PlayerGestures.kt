package com.aiyu.kinema.ui

import android.app.Activity
import android.media.AudioManager
import android.media.audiofx.LoudnessEnhancer
import android.provider.Settings
import android.view.WindowManager
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.VolumeOff
import androidx.compose.material.icons.automirrored.rounded.VolumeUp
import androidx.compose.material.icons.rounded.BrightnessMedium
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.media3.exoplayer.ExoPlayer
import com.aiyu.kinema.R
import kotlin.math.roundToInt
import kotlinx.coroutines.delay

enum class Swipe { BRIGHTNESS, VOLUME }

/** Loudest volume: 200 %, the upper 100 % being boost, as in MX Player. */
const val MAX_VOLUME = 2f

/** Gain at full boost, in millibels: +12 dB is clearly louder without tearing most speakers apart. */
private const val BOOST_MB = 1200

/**
 * Screen brightness (0..1, for this window only) and media volume (0..[MAX_VOLUME]). Up to 1 the phone's own media
 * volume moves; above it a [LoudnessEnhancer] on the player's audio session boosts quiet files.
 */
class SwipeLevels(private val activity: Activity, private val player: ExoPlayer) {
    private val audio = activity.getSystemService(AudioManager::class.java)
    private val steps = audio?.getStreamMaxVolume(AudioManager.STREAM_MUSIC)?.coerceAtLeast(1) ?: 1
    private var enhancer: LoudnessEnhancer? = null

    var volume by mutableFloatStateOf(streamLevel())
        private set
    var brightness by mutableFloatStateOf(initialBrightness())
        private set

    private fun streamLevel() = (audio?.getStreamVolume(AudioManager.STREAM_MUSIC) ?: steps).toFloat() / steps

    private fun initialBrightness(): Float {
        val own = activity.window.attributes.screenBrightness
        if (own >= 0) return own
        return runCatching { Settings.System.getInt(activity.contentResolver, Settings.System.SCREEN_BRIGHTNESS) / 255f }.getOrDefault(0.5f)
    }

    /** The volume keys may have moved the media volume since the last swipe; boost holds only while it is at the top. */
    fun syncVolume() {
        val stream = streamLevel()
        if (volume <= 1f || stream < 1f) {
            volume = stream
            boost(0)
        }
    }

    fun changeVolume(to: Float) {
        val level = to.coerceIn(0f, MAX_VOLUME)
        volume = level
        runCatching { audio?.setStreamVolume(AudioManager.STREAM_MUSIC, (level.coerceAtMost(1f) * steps).roundToInt(), 0) }
        boost(((level - 1f).coerceAtLeast(0f) * BOOST_MB).roundToInt())
    }

    fun changeBrightness(to: Float) {
        brightness = to.coerceIn(0f, 1f)
        // 0 turns some screens fully off; keep a sliver of light.
        activity.window.attributes = activity.window.attributes.apply { screenBrightness = brightness.coerceAtLeast(0.01f) }
    }

    private fun boost(millibels: Int) {
        if (millibels == 0 && enhancer == null) return
        val e = enhancer ?: runCatching { LoudnessEnhancer(player.audioSessionId) }.getOrNull()?.also { enhancer = it } ?: return
        runCatching {
            e.setTargetGain(millibels)
            e.enabled = millibels > 0
        }
    }

    /** The window goes back to the system brightness; the media volume stays where the viewer left it. */
    fun release() {
        activity.window.attributes = activity.window.attributes.apply { screenBrightness = WindowManager.LayoutParams.BRIGHTNESS_OVERRIDE_NONE }
        runCatching { enhancer?.release() }
        enhancer = null
    }
}

@Composable
fun rememberSwipeLevels(player: ExoPlayer): SwipeLevels? {
    val activity = LocalContext.current.findActivity() ?: return null
    val levels = remember(player) { SwipeLevels(activity, player) }
    DisposableEffect(levels) { onDispose { levels.release() } }
    return levels
}

/** A vertical swipe on the left half sets brightness, on the right half volume. [onSwipe] gets null when it ends. */
fun Modifier.swipeLevels(levels: SwipeLevels, onSwipe: (Swipe?) -> Unit) = pointerInput(levels) {
    var kind: Swipe? = null
    detectVerticalDragGestures(
        onDragStart = { start ->
            // The top 20% is reserved for swipe-down-to-PiP, bottom 10% for system navigation
            kind = when {
                start.y < size.height * 0.20f || start.y > size.height * 0.90f -> null
                start.x < size.width / 2f -> Swipe.BRIGHTNESS
                else -> Swipe.VOLUME
            }
            if (kind == Swipe.VOLUME) levels.syncVolume()
            onSwipe(kind)
        },
        onDragEnd = {
            kind = null
            onSwipe(null)
        },
        onDragCancel = {
            kind = null
            onSwipe(null)
        },
        onVerticalDrag = { change, dy ->
            val k = kind ?: return@detectVerticalDragGestures
            // A swipe the full height of the screen moves a level by 100 %.
            val delta = -dy / size.height
            when (k) {
                Swipe.BRIGHTNESS -> levels.changeBrightness(levels.brightness + delta)
                Swipe.VOLUME -> levels.changeVolume(levels.volume + delta)
            }
            change.consume()
        },
    )
}

/** Continuous downward drag on the video or player that pulls it down and minimizes when released past threshold. */
fun Modifier.swipeDownToMinimize(
    enabled: Boolean = true,
    onDrag: (Float) -> Unit,
    onRelease: (Boolean) -> Unit,
) = if (!enabled) this else pointerInput(Unit) {
    var totalDy = 0f
    detectVerticalDragGestures(
        onDragStart = {
            totalDy = 0f
        },
        onDragEnd = {
            val passed = totalDy > 100.dp.toPx()
            onRelease(passed)
            totalDy = 0f
        },
        onDragCancel = {
            onRelease(false)
            totalDy = 0f
        },
        onVerticalDrag = { change, dy ->
            if (dy > 0 || totalDy > 0f) {
                totalDy = maxOf(0f, totalDy + dy)
                onDrag(totalDy)
                change.consume()
            }
        },
    )
}

/**
 * YouTube-style video gesture detector:
 * - When [isFullscreen] is true (landscape fullscreen):
 *   - Pulling down (> 50dp or starting in top 35% with > 30dp) exits fullscreen back to watch screen
 *   - Vertical drag on left half adjusts brightness; on right half adjusts volume
 * - When [isFullscreen] is false (portrait watch screen):
 *   - Swiping up (< -35dp) triggers [onEnterFullscreen]
 *   - Swiping down calls [onPullDownDrag] continuously and on release triggers [onPullDownRelease] with true
 *     if dragged past 75dp (to minimize to in-app PiP)
 */
fun Modifier.playerVideoGestures(
    isFullscreen: Boolean,
    levels: SwipeLevels?,
    onSwipeLevel: (Swipe?) -> Unit,
    onEnterFullscreen: () -> Unit,
    onExitFullscreen: () -> Unit,
    onPullDownDrag: (Float) -> Unit,
    onPullDownRelease: (Boolean) -> Unit,
) = pointerInput(isFullscreen, levels) {
    var kind: Swipe? = null
    var totalDy = 0f
    var isFullscreenExit = false

    detectVerticalDragGestures(
        onDragStart = { start ->
            totalDy = 0f
            isFullscreenExit = false
            if (isFullscreen) {
                if (levels != null && start.x < size.width * 0.20f && start.y in (size.height * 0.15f)..(size.height * 0.85f)) {
                    kind = Swipe.BRIGHTNESS
                    onSwipeLevel(kind)
                } else if (levels != null && start.x > size.width * 0.80f && start.y in (size.height * 0.15f)..(size.height * 0.85f)) {
                    kind = Swipe.VOLUME
                    levels.syncVolume()
                    onSwipeLevel(kind)
                } else {
                    isFullscreenExit = true
                    kind = null
                }
            }
        },
        onDragEnd = {
            if (isFullscreen) {
                if (isFullscreenExit && totalDy > 35.dp.toPx()) {
                    onExitFullscreen()
                }
                kind = null
                onSwipeLevel(null)
            } else {
                if (totalDy < -35.dp.toPx()) {
                    onPullDownRelease(false)
                    onEnterFullscreen()
                } else {
                    val passed = totalDy > 65.dp.toPx()
                    onPullDownRelease(passed)
                }
            }
            totalDy = 0f
        },
        onDragCancel = {
            if (!isFullscreen) {
                onPullDownRelease(false)
            }
            kind = null
            onSwipeLevel(null)
            totalDy = 0f
        },
        onVerticalDrag = { change, dy ->
            totalDy += dy
            if (isFullscreen) {
                val k = kind
                if (k != null && levels != null) {
                    val delta = -dy / size.height
                    when (k) {
                        Swipe.BRIGHTNESS -> levels.changeBrightness(levels.brightness + delta)
                        Swipe.VOLUME -> levels.changeVolume(levels.volume + delta)
                    }
                }
                change.consume()
            } else {
                if (dy > 0 || totalDy > 0f) {
                    onPullDownDrag(maxOf(0f, totalDy))
                }
                change.consume()
            }
        },
    )
}

/** Swiping up on the miniplayer expands it back to the full player; swiping down dismisses it. */
fun Modifier.miniPlayerGestures(
    onExpand: () -> Unit,
    onClose: () -> Unit,
) = pointerInput(Unit) {
    var totalDy = 0f
    detectVerticalDragGestures(
        onDragStart = {
            totalDy = 0f
        },
        onDragEnd = {
            if (totalDy < -36.dp.toPx()) {
                onExpand()
            } else if (totalDy > 50.dp.toPx()) {
                onClose()
            }
            totalDy = 0f
        },
        onDragCancel = {
            totalDy = 0f
        },
        onVerticalDrag = { change, dy ->
            totalDy += dy
            change.consume()
        },
    )
}

/** Swiping down from the upper portion of the video smoothly triggers Picture-in-Picture. */
fun Modifier.swipeDownToPip(onSwipeDown: () -> Unit) = pointerInput(onSwipeDown) {
    var eligible = false
    var totalDy = 0f
    detectVerticalDragGestures(
        onDragStart = { start ->
            // Initiated from the top 35% of the video
            eligible = start.y < size.height * 0.35f
            totalDy = 0f
        },
        onDragEnd = {
            if (eligible && totalDy > size.height * 0.15f) {
                onSwipeDown()
            }
            eligible = false
            totalDy = 0f
        },
        onDragCancel = {
            eligible = false
            totalDy = 0f
        },
        onVerticalDrag = { change, dy ->
            if (eligible && dy > 0) {
                totalDy += dy
                change.consume()
            }
        },
    )
}

/** The level being swiped, as an icon, a bar and a percentage; lingers briefly after the finger lifts. */
@Composable
fun BoxScope.SwipeIndicator(swipe: Swipe?, levels: SwipeLevels) {
    var shown by remember { mutableStateOf<Swipe?>(null) }
    var last by remember { mutableStateOf(Swipe.VOLUME) }
    LaunchedEffect(swipe) {
        if (swipe != null) {
            shown = swipe
            last = swipe
        } else {
            delay(700)
            shown = null
        }
    }
    val motion = MaterialTheme.motionScheme
    AnimatedVisibility(
        shown != null,
        Modifier.align(Alignment.TopCenter).padding(top = 56.dp),
        enter = scaleIn(motion.fastSpatialSpec(), initialScale = 0.85f) + fadeIn(motion.fastEffectsSpec()),
        exit = fadeOut(motion.defaultEffectsSpec()),
    ) {
        val volume = last == Swipe.VOLUME
        val level = if (volume) levels.volume else levels.brightness
        val boosted = volume && level > 1f
        val accent = if (boosted) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.primary
        Surface(color = Color.Black.copy(alpha = 0.65f), contentColor = Color.White, shape = MaterialTheme.shapes.extraLarge) {
            Row(Modifier.padding(horizontal = 20.dp, vertical = 14.dp), horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    when {
                        !volume -> Icons.Rounded.BrightnessMedium
                        level == 0f -> Icons.AutoMirrored.Rounded.VolumeOff
                        else -> Icons.AutoMirrored.Rounded.VolumeUp
                    },
                    stringResource(if (volume) R.string.player_volume else R.string.player_brightness),
                    tint = accent,
                )
                LinearProgressIndicator(
                    progress = { if (volume) level / MAX_VOLUME else level },
                    color = accent,
                    trackColor = Color.White.copy(alpha = 0.25f),
                    modifier = Modifier.width(160.dp),
                )
                Text(
                    if (boosted) stringResource(R.string.player_boost, (level * 100).roundToInt()) else "${(level * 100).roundToInt()}%",
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.SemiBold,
                    color = if (boosted) accent else Color.White,
                    modifier = Modifier.widthIn(min = 48.dp),
                )
            }
        }
    }
}
