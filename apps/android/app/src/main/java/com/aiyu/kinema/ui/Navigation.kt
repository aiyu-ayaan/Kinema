package com.aiyu.kinema.ui

import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.core.animate
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MotionScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberNavBackStack
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

/** Opens a title's page or plays it; screens get it instead of the back stack. */
class Navigator(val open: (Long) -> Unit, val play: (Long, Double?) -> Unit, val back: () -> Unit)

val LocalNavigator = staticCompositionLocalOf<Navigator> { error("No navigator") }

@Serializable
data object TabsKey : NavKey

@Serializable
data class ItemKey(val id: Long) : NavKey

/** [start] null resumes where the viewer stopped. */
@Serializable
data class PlayerKey(val id: Long, val start: Double? = null) : NavKey

/**
 * Signed-in navigation. Pages push with a shared-axis slide; the system back gesture previews the page below:
 * the top page shrinks toward the edge being swiped from while the one beneath fades up, and lets go or springs
 * back with the gesture.
 */
@Composable
fun AppNavigation() {
    val stack = rememberNavBackStack(TabsKey)
    val motion = MaterialTheme.motionScheme
    val navigator = remember(stack) {
        Navigator(
            open = { stack.add(ItemKey(it)) },
            play = { id, start ->
                // "Next episode" replaces the player instead of stacking players.
                if (stack.lastOrNull() is PlayerKey) stack.removeAt(stack.lastIndex)
                stack.add(PlayerKey(id, start))
            },
            back = { if (stack.size > 1) stack.removeAt(stack.lastIndex) },
        )
    }
    androidx.compose.runtime.CompositionLocalProvider(LocalNavigator provides navigator) {
        NavDisplay(
            backStack = stack,
            onBack = { if (stack.size > 1) stack.removeAt(stack.lastIndex) },
            transitionSpec = { sharedAxisX(forward = true, motion) },
            popTransitionSpec = { sharedAxisX(forward = false, motion) },
            predictivePopTransitionSpec = { edge -> predictivePop(edge, motion) },
            entryProvider = entryProvider {
                entry<TabsKey> { MainScreen() }
                entry<ItemKey> { ItemScreen(it.id) }
                entry<PlayerKey> { PlayerScreen(it.id, it.start) }
            },
        )
    }
}

/** Shared axis X: the new page slides in from the end (or back from the start) while the old one fades out. */
fun <T> AnimatedContentTransitionScope<T>.sharedAxisX(forward: Boolean, motion: MotionScheme): ContentTransform =
    (slideInHorizontally(motion.defaultSpatialSpec()) { if (forward) it / 4 else -it / 4 } + fadeIn(motion.defaultEffectsSpec()))
        .togetherWith(slideOutHorizontally(motion.fastSpatialSpec()) { if (forward) -it / 4 else it / 4 } + fadeOut(motion.fastEffectsSpec()))

/** Material predictive back: the page scales to 90 % toward the swiped edge; the page beneath rises from 95 %. */
private fun <T> AnimatedContentTransitionScope<T>.predictivePop(edge: Int, motion: MotionScheme): ContentTransform {
    val pivot = if (edge == NavigationEvent.EDGE_LEFT) TransformOrigin(1f, 0.5f) else TransformOrigin(0f, 0.5f)
    return (fadeIn(motion.defaultEffectsSpec()) + scaleIn(motion.defaultSpatialSpec(), initialScale = 0.95f))
        .togetherWith(scaleOut(motion.defaultSpatialSpec(), targetScale = 0.9f, transformOrigin = pivot) + fadeOut(motion.slowEffectsSpec()))
}

/**
 * Back gesture with a live preview inside one screen (tabs, sheets). While the user drags, the returned progress runs
 * 0..1 for [predictiveBackEffect]; [onBack] runs when the gesture is committed, and a cancelled gesture springs back.
 */
@Composable
fun rememberPredictiveBack(enabled: Boolean, onBack: () -> Unit): State<Float> {
    val progress = remember { mutableFloatStateOf(0f) }
    PredictiveBackHandler(enabled) { events ->
        try {
            events.collect { progress.floatValue = it.progress }
            onBack()
            progress.floatValue = 0f
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
