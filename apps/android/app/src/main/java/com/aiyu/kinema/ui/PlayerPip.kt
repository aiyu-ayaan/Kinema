package com.aiyu.kinema.ui

import android.app.PendingIntent
import android.app.PictureInPictureParams
import android.app.RemoteAction
import android.content.BroadcastReceiver
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.graphics.Rect
import android.graphics.drawable.Icon
import android.os.Build
import android.util.Rational
import androidx.activity.ComponentActivity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.core.app.PictureInPictureModeChangedInfo
import androidx.core.content.ContextCompat
import androidx.core.util.Consumer
import androidx.lifecycle.Lifecycle
import androidx.media3.common.Player
import androidx.media3.common.VideoSize
import com.aiyu.kinema.R
import com.aiyu.kinema.playback.Playback
import kotlin.math.roundToInt

fun Context.findActivity(): ComponentActivity? {
    var c: Context? = this
    while (c is ContextWrapper) {
        if (c is ComponentActivity) return c
        c = c.baseContext
    }
    return null
}

private const val ACTION_TOGGLE = "com.aiyu.kinema.PIP_TOGGLE"
private const val ACTION_BACK = "com.aiyu.kinema.PIP_BACK"
private const val ACTION_FORWARD = "com.aiyu.kinema.PIP_FORWARD"
private const val SKIP_S = 10.0

/** Android refuses picture-in-picture shapes wider than 2.39:1 or taller than 1:2.39. */
private fun pipAspect(width: Float, height: Int): Rational {
    val ratio = (width / height).coerceIn(1f / 2.38f, 2.38f)
    return Rational((ratio * 10_000).roundToInt(), 10_000)
}

private fun pipAction(activity: ComponentActivity, code: Int, action: String, icon: Int, label: Int): RemoteAction {
    val text = activity.getString(label)
    val intent = PendingIntent.getBroadcast(
        activity, code, Intent(action).setPackage(activity.packageName),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
    return RemoteAction(Icon.createWithResource(activity, icon), text, text, intent)
}

/** Back 10 s, play/pause, forward 10 s; only play/pause when the system allows a single button. */
private fun pipParams(activity: ComponentActivity, playing: Boolean, aspect: Rational, autoEnter: Boolean?, sourceRect: Rect? = null): PictureInPictureParams {
    val toggle = pipAction(
        activity, 1, ACTION_TOGGLE,
        if (playing) R.drawable.ic_pip_pause else R.drawable.ic_pip_play,
        if (playing) R.string.player_pause else R.string.action_play,
    )
    val actions = if (activity.maxNumPictureInPictureActions >= 3) {
        listOf(
            pipAction(activity, 0, ACTION_BACK, R.drawable.ic_pip_back, R.string.player_back10),
            toggle,
            pipAction(activity, 2, ACTION_FORWARD, R.drawable.ic_pip_forward, R.string.player_forward10),
        )
    } else listOf(toggle)
    return PictureInPictureParams.Builder()
        .setAspectRatio(aspect)
        .setActions(actions)
        .apply {
            if (sourceRect != null && sourceRect.width() > 0 && sourceRect.height() > 0) {
                setSourceRectHint(sourceRect)
            }
            if (autoEnter != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) setAutoEnterEnabled(autoEnter)
        }
        .build()
}

fun enterPip(activity: ComponentActivity, playing: Boolean, aspect: Rational = Rational(16, 9), sourceRect: Rect? = null) {
    if (activity.packageManager.hasSystemFeature(PackageManager.FEATURE_PICTURE_IN_PICTURE)) {
        runCatching { activity.enterPictureInPictureMode(pipParams(activity, playing, aspect, null, sourceRect)) }
    }
}

/**
 * Picture-in-picture for the player. With [enabled], leaving the app (home, recents) shrinks the video into a
 * window that keeps playing; closing that window pauses it. Returns whether the activity is in that window now.
 */
@Composable
fun rememberPictureInPicture(player: Player, enabled: Boolean, playing: Boolean, sourceRect: Rect? = null): Boolean {
    val activity = LocalContext.current.findActivity() ?: return false
    val supported = remember(activity) { activity.packageManager.hasSystemFeature(PackageManager.FEATURE_PICTURE_IN_PICTURE) }
    var inPip by remember { mutableStateOf(activity.isInPictureInPictureMode) }
    var aspect by remember { mutableStateOf(Rational(16, 9)) }
    val canEnter by rememberUpdatedState(supported && enabled)
    val isPlaying by rememberUpdatedState(playing)

    fun params() = pipParams(activity, isPlaying, aspect, canEnter && isPlaying, sourceRect)

    // Keeps the window's shape and its play/pause button current, and arms auto-enter on Android 12+.
    LaunchedEffect(supported, enabled, playing, aspect, sourceRect) {
        if (supported) runCatching { activity.setPictureInPictureParams(params()) }
    }

    DisposableEffect(player, activity) {
        val sizes = object : Player.Listener {
            override fun onVideoSizeChanged(videoSize: VideoSize) {
                if (videoSize.width > 0 && videoSize.height > 0) aspect = pipAspect(videoSize.width * videoSize.pixelWidthHeightRatio, videoSize.height)
            }
        }
        val modes = Consumer<PictureInPictureModeChangedInfo> { info ->
            inPip = info.isInPictureInPictureMode
            // Closing the window stops the activity before it leaves picture-in-picture; expanding it does not.
            if (!info.isInPictureInPictureMode && !activity.lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) player.pause()
        }
        val leaving = Runnable {
            if (canEnter && isPlaying && !activity.isInPictureInPictureMode) {
                runCatching { activity.enterPictureInPictureMode(params()) }
            }
        }
        val toggle = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                // Seeks go through the screen's logic: a remux or transcode stream may have to restart.
                val controls = Playback.controls
                when (intent.action) {
                    ACTION_BACK -> controls?.seekBy?.invoke(-SKIP_S)
                    ACTION_FORWARD -> controls?.seekBy?.invoke(SKIP_S)
                    else -> if (player.isPlaying) player.pause() else player.play()
                }
            }
        }
        player.addListener(sizes)
        activity.addOnPictureInPictureModeChangedListener(modes)
        activity.addOnUserLeaveHintListener(leaving)
        val filter = IntentFilter().apply { addAction(ACTION_TOGGLE); addAction(ACTION_BACK); addAction(ACTION_FORWARD) }
        ContextCompat.registerReceiver(activity, toggle, filter, ContextCompat.RECEIVER_NOT_EXPORTED)
        onDispose {
            player.removeListener(sizes)
            activity.removeOnPictureInPictureModeChangedListener(modes)
            activity.removeOnUserLeaveHintListener(leaving)
            runCatching { activity.unregisterReceiver(toggle) }
            // The rest of the app must not shrink into a window when the viewer leaves it.
            if (supported && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                runCatching { activity.setPictureInPictureParams(PictureInPictureParams.Builder().setAutoEnterEnabled(false).build()) }
            }
        }
    }
    return inPip
}
