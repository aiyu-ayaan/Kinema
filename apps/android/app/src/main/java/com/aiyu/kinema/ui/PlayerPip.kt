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

/** Android refuses picture-in-picture shapes wider than 2.39:1 or taller than 1:2.39. */
private fun pipAspect(width: Float, height: Int): Rational {
    val ratio = (width / height).coerceIn(1f / 2.38f, 2.38f)
    return Rational((ratio * 10_000).roundToInt(), 10_000)
}

/**
 * Picture-in-picture for the player. With [enabled], leaving the app (home, recents) shrinks the video into a
 * window that keeps playing; closing that window pauses it. Returns whether the activity is in that window now.
 */
@Composable
fun rememberPictureInPicture(player: Player, enabled: Boolean, playing: Boolean): Boolean {
    val activity = LocalContext.current.findActivity() ?: return false
    val supported = remember(activity) { activity.packageManager.hasSystemFeature(PackageManager.FEATURE_PICTURE_IN_PICTURE) }
    var inPip by remember { mutableStateOf(activity.isInPictureInPictureMode) }
    var aspect by remember { mutableStateOf(Rational(16, 9)) }
    val canEnter by rememberUpdatedState(supported && enabled)
    val isPlaying by rememberUpdatedState(playing)

    fun params(): PictureInPictureParams {
        val toggleLabel = activity.getString(if (isPlaying) R.string.player_pause else R.string.action_play)
        val toggle = PendingIntent.getBroadcast(
            activity, 0, Intent(ACTION_TOGGLE).setPackage(activity.packageName),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val icon = Icon.createWithResource(activity, if (isPlaying) R.drawable.ic_pip_pause else R.drawable.ic_pip_play)
        return PictureInPictureParams.Builder()
            .setAspectRatio(aspect)
            .setActions(listOf(RemoteAction(icon, toggleLabel, toggleLabel, toggle)))
            .apply { if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) setAutoEnterEnabled(canEnter && isPlaying) }
            .build()
    }

    // Keeps the window's shape and its play/pause button current, and arms auto-enter on Android 12+.
    LaunchedEffect(supported, enabled, playing, aspect) {
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
        // Before Android 12 there is no auto-enter: going home only gives this hint.
        val leaving = Runnable {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S && canEnter && isPlaying) runCatching { activity.enterPictureInPictureMode(params()) }
        }
        val toggle = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                if (player.isPlaying) player.pause() else player.play()
            }
        }
        player.addListener(sizes)
        activity.addOnPictureInPictureModeChangedListener(modes)
        activity.addOnUserLeaveHintListener(leaving)
        ContextCompat.registerReceiver(activity, toggle, IntentFilter(ACTION_TOGGLE), ContextCompat.RECEIVER_NOT_EXPORTED)
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
