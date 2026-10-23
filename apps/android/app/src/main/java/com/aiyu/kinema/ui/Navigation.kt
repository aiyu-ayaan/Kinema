package com.aiyu.kinema.ui

import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterExitState
import androidx.compose.animation.core.Easing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.MotionScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.State
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import androidx.navigation3.ui.NavDisplay
import androidx.navigationevent.NavigationEvent
import com.aiyu.kinema.core.KinemaApi
import com.aiyu.kinema.core.KinemaRepository
import kotlin.coroutines.cancellation.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable

val LocalRepository = staticCompositionLocalOf<KinemaRepository> { error("No repository") }

/** The client of the address in use, for image and stream URLs. Null before a server is chosen. */
val LocalApi = staticCompositionLocalOf<KinemaApi?> { null }

/** Opens a title's page, plays it or opens another page ([go]); screens get it instead of the back stack. */
class Navigator(val open: (Long) -> Unit, val play: (Long, Double?) -> Unit, val back: () -> Unit, val go: (NavKey) -> Unit)

val LocalNavigator = staticCompositionLocalOf<Navigator> { error("No navigator") }

@Serializable
data object TabsKey : NavKey

@Serializable
data class ItemKey(val id: Long) : NavKey

@Serializable
data class PlaylistKey(val id: Long) : NavKey

@Serializable
data class CollectionKey(val id: Long) : NavKey

/** [start] null resumes where the viewer stopped. */
@Serializable
data class PlayerKey(val id: Long, val start: Double? = null) : NavKey

/** Settings › Updates, About Kinema and its Acknowledgements (AboutScreens.kt). */
@Serializable
data object UpdatesKey : NavKey

@Serializable
data object AboutKey : NavKey

@Serializable
data object AcknowledgementsKey : NavKey

@Serializable
data class ActivePlayback(val id: Long, val start: Double? = null) : java.io.Serializable

/**
 * Signed-in navigation. Pages push with a shared-axis slide; the system back gesture previews the page beneath.
 * An active player is hosted as an overlay: minimizes to a YouTube-style in-app PiP miniplayer at bottom right.
 */
@Composable
fun AppNavigation() {
    val stack = rememberNavBackStack(TabsKey)
    val motion = MaterialTheme.motionScheme
    val large = LocalConfiguration.current.smallestScreenWidthDp >= 600
    var activePlayback by rememberSaveable { mutableStateOf<ActivePlayback?>(null) }
    var isMinimized by rememberSaveable { mutableStateOf(false) }

    val navigator = remember(stack) {
        Navigator(
            open = { stack.add(ItemKey(it)) },
            play = { id, start ->
                activePlayback = ActivePlayback(id, start)
                isMinimized = false
            },
            back = { if (stack.size > 1) stack.removeAt(stack.lastIndex) },
            go = { stack.add(it) },
        )
    }
    androidx.compose.runtime.CompositionLocalProvider(LocalNavigator provides navigator) {
        Box(Modifier.fillMaxSize()) {
            NavDisplay(
                backStack = stack,
                onBack = { if (stack.size > 1) stack.removeAt(stack.lastIndex) },
                transitionSpec = { sharedAxisX(forward = true, motion) },
                popTransitionSpec = { sharedAxisX(forward = false, motion).apply { targetContentZIndex = -1f } },
                predictivePopTransitionSpec = { edge -> predictivePop(edge) },
                // Each page is opaque, so the page beneath only shows around a shrinking one.
                entryProvider = entryProvider {
                    entry<TabsKey> { Page { MainScreen() } }
                    entry<ItemKey> { Page { ItemScreen(it.id) } }
                    entry<PlaylistKey> { Page { PlaylistScreen(it.id) } }
                    entry<CollectionKey> { Page { CollectionScreen(it.id) } }
                    entry<PlayerKey> { PlayerScreen(it.id, it.start, isMinimized = false, onMinimize = {}, onExpand = {}, onClose = { if (stack.size > 1) stack.removeAt(stack.lastIndex) }) }
                    entry<UpdatesKey> { Page { UpdatesScreen() } }
                    entry<AboutKey> { Page { AboutScreen() } }
                    entry<AcknowledgementsKey> { Page { AcknowledgementsScreen() } }
                },
            )
            com.aiyu.kinema.update.UpdatePrompt()

            activePlayback?.let { playback ->
                PlayerScreen(
                    id = playback.id,
                    start = playback.start,
                    isMinimized = isMinimized,
                    onMinimize = { isMinimized = true },
                    onExpand = { isMinimized = false },
                    onClose = {
                        activePlayback = null
                        isMinimized = false
                    },
                )
            }
        }
    }
}

/** A leaving page rounds its corners, so one shrinking under the back gesture reads as a card lifting off. */
@Composable
private fun Page(content: @Composable () -> Unit) {
    val corner = LocalNavAnimatedContentScope.current.transition.animateDp(
        transitionSpec = { tween(PREDICTIVE_MS, easing = LinearEasing) },
        label = "corner",
    ) { if (it == EnterExitState.PostExit) 28.dp else 0.dp }
    Surface(
        Modifier.fillMaxSize().graphicsLayer {
            shape = RoundedCornerShape(corner.value)
            clip = corner.value > 0.dp
        },
        content = content,
    )
}

/** Shared axis X: the new page slides in from the end (or back from the start) while the old one fades out. */
fun <T> AnimatedContentTransitionScope<T>.sharedAxisX(forward: Boolean, motion: MotionScheme): ContentTransform =
    (slideInHorizontally(motion.defaultSpatialSpec()) { if (forward) it / 4 else -it / 4 } + fadeIn(motion.defaultEffectsSpec()))
        .togetherWith(slideOutHorizontally(motion.fastSpatialSpec()) { if (forward) -it / 4 else it / 4 } + fadeOut(motion.fastEffectsSpec()))

/**
 * Android's own back animation (the one between activities): while the finger drags, the page shrinks to 90 % and
 * drifts a little toward the far edge, and the page beneath slides in from the near side, dimmed. Let go and the page
 * slides off the screen; it never fades, so no ghost of it lingers over the page beneath.
 *
 * NavDisplay seeks one transition with the gesture progress (already eased by the system), then plays the rest on
 * release. So both phases live on one timeline: the drag is the first [DRAG_END] of it, the slide-off the rest. The
 * easings below are piecewise linear for the same reason the specs are tweens: a curve of their own would make the
 * page lag the finger, then lurch.
 */
private fun <T> AnimatedContentTransitionScope<T>.predictivePop(edge: Int): ContentTransform {
    val away = if (edge == NavigationEvent.EDGE_LEFT) 1 else -1
    val drift = Easing { f -> if (f < DRAG_END) f / DRAG_END * DRIFT else DRIFT + (1 - DRIFT) * (f - DRAG_END) / (1 - DRAG_END) }
    val shrink = Easing { f -> (f / DRAG_END).coerceAtMost(1f) }
    val linear = tween<Float>(PREDICTIVE_MS, easing = LinearEasing)
    val exit = slideOutHorizontally(tween(PREDICTIVE_MS, easing = drift)) { away * it } +
        scaleOut(tween(PREDICTIVE_MS, easing = shrink), targetScale = 0.9f)
    val enter = slideInHorizontally(tween(PREDICTIVE_MS, easing = LinearEasing)) { -away * it / 10 } +
        fadeIn(linear, initialAlpha = 0.6f)
    return (enter togetherWith exit).apply { targetContentZIndex = -1f }
}

private const val PREDICTIVE_MS = 320

/** The share of the timeline the finger drives; past it the page leaves on its own. */
private const val DRAG_END = 0.5f

/** How far the page drifts during the drag, as a share of its width (the system's is about 8 %). */
private const val DRIFT = 0.08f

/**
 * Back gesture with a live preview inside one screen (tabs, sheets). While the user drags, the returned progress runs
 * 0..1 for [predictiveBackEffect]; [onBack] runs when the gesture is committed, and either way the content springs
 * back to full size instead of snapping.
 */
@Composable
fun rememberPredictiveBack(enabled: Boolean, onBack: () -> Unit): State<Float> {
    val progress = remember { mutableFloatStateOf(0f) }
    PredictiveBackHandler(enabled) { events ->
        try {
            events.collect { progress.floatValue = it.progress }
            onBack()
            animate(progress.floatValue, 0f) { value, _ -> progress.floatValue = value }
        } catch (e: CancellationException) {
            withContext(NonCancellable) { animate(progress.floatValue, 0f) { value, _ -> progress.floatValue = value } }
            throw e
        }
    }
    return progress
}

/** The Material back preview: the content scales down to 90 % and rounds its corners as [progress] grows. */
fun Modifier.predictiveBackEffect(progress: () -> Float): Modifier = graphicsLayer {
    val p = progress()
    if (p > 0f) {
        scaleX = 1f - 0.1f * p
        scaleY = 1f - 0.1f * p
        shape = RoundedCornerShape((32 * p).dp)
        clip = true
    }
}
