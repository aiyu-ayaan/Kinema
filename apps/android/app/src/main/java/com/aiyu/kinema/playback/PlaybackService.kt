package com.aiyu.kinema.playback

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Bundle
import androidx.media3.common.Player
import androidx.media3.session.CommandButton
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionCommand
import androidx.media3.session.SessionResult
import com.aiyu.kinema.R
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture

/**
 * The player of the open player screen, handed to [PlaybackService] for the notification, lock screen and
 * headset controls. One at a time: a newer screen (the next episode) takes over before the old one lets go.
 */
object Playback {
    /** What the notification buttons do; the screen owns the logic (remux streams restart on a seek). */
    class Controls(val stop: () -> Unit, val seekBy: (Double) -> Unit, val next: (() -> Unit)?)

    internal var player: Player? = null
        private set
    internal var controls: Controls? = null
        private set

    /** Set by the running service; tells it the player or its buttons changed. */
    internal var changed: (() -> Unit)? = null

    fun attach(context: Context, player: Player, controls: Controls) {
        this.player = player
        this.controls = controls
        changed?.invoke()
        // Also when the service runs: it may be on its way down after the previous screen's detach.
        runCatching { context.startService(Intent(context, PlaybackService::class.java)) }
    }

    fun detach(context: Context, player: Player) {
        if (this.player !== player) return
        this.player = null
        controls = null
        context.stopService(Intent(context, PlaybackService::class.java))
    }
}

/**
 * Keeps playback alive in the background as a foreground media service; Media3 posts the media notification while
 * the session's player has something to play. The player itself belongs to the player screen, which releases it.
 */
class PlaybackService : MediaSessionService() {
    private var session: MediaSession? = null
    private val listener: () -> Unit = { sync() }

    override fun onCreate() {
        super.onCreate()
        val player = Playback.player ?: return stopSelf()
        val open = packageManager.getLaunchIntentForPackage(packageName)?.let {
            PendingIntent.getActivity(this, 0, it, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        }
        session = MediaSession.Builder(this, player)
            .setCallback(Callback())
            .setMediaButtonPreferences(buttons())
            .apply { if (open != null) setSessionActivity(open) }
            .build()
            .also(::addSession)
        Playback.changed = listener
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        sync()
        return super.onStartCommand(intent, flags, startId)
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

    /** Swiped away from recents: nothing to come back to, so stop instead of playing on. */
    override fun onTaskRemoved(rootIntent: Intent?) {
        session?.player?.pause()
        stopSelf()
    }

    override fun onDestroy() {
        if (Playback.changed === listener) Playback.changed = null
        session?.release()
        session = null
        super.onDestroy()
    }

    /** A newer player screen took over (next episode), or its buttons changed. */
    private fun sync() {
        val s = session ?: return
        val player = Playback.player ?: return
        if (s.player !== player) s.player = player
        s.setMediaButtonPreferences(buttons())
    }

    /** Back 10 s, play/pause (added by Media3), forward 10 s in the compact view; next episode and stop when expanded. */
    private fun buttons(): List<CommandButton> = listOfNotNull(
        CommandButton.Builder(CommandButton.ICON_SKIP_BACK_10)
            .setSessionCommand(BACK)
            .setDisplayName(getString(R.string.player_back10))
            .setSlots(CommandButton.SLOT_BACK)
            .build(),
        CommandButton.Builder(CommandButton.ICON_SKIP_FORWARD_10)
            .setSessionCommand(FORWARD)
            .setDisplayName(getString(R.string.player_forward10))
            .setSlots(CommandButton.SLOT_FORWARD)
            .build(),
        Playback.controls?.next?.let {
            CommandButton.Builder(CommandButton.ICON_NEXT)
                .setSessionCommand(NEXT)
                .setDisplayName(getString(R.string.player_next))
                .setSlots(CommandButton.SLOT_OVERFLOW)
                .build()
        },
        CommandButton.Builder(CommandButton.ICON_STOP)
            .setSessionCommand(STOP)
            .setDisplayName(getString(R.string.player_stop))
            .setSlots(CommandButton.SLOT_OVERFLOW)
            .build(),
    )

    private class Callback : MediaSession.Callback {
        override fun onConnect(session: MediaSession, controller: MediaSession.ControllerInfo): MediaSession.ConnectionResult =
            MediaSession.ConnectionResult.AcceptedResultBuilder(session)
                .setAvailableSessionCommands(
                    MediaSession.ConnectionResult.DEFAULT_SESSION_COMMANDS.buildUpon().add(BACK).add(FORWARD).add(NEXT).add(STOP).build(),
                )
                .build()

        override fun onCustomCommand(
            session: MediaSession,
            controller: MediaSession.ControllerInfo,
            customCommand: SessionCommand,
            args: Bundle,
        ): ListenableFuture<SessionResult> {
            val controls = Playback.controls
            when (customCommand.customAction) {
                BACK.customAction -> controls?.seekBy?.invoke(-10.0)
                FORWARD.customAction -> controls?.seekBy?.invoke(10.0)
                NEXT.customAction -> controls?.next?.invoke()
                STOP.customAction -> controls?.stop?.invoke()
            }
            return Futures.immediateFuture(SessionResult(SessionResult.RESULT_SUCCESS))
        }
    }

    private companion object {
        val BACK = SessionCommand("kinema.back10", Bundle.EMPTY)
        val FORWARD = SessionCommand("kinema.forward10", Bundle.EMPTY)
        val NEXT = SessionCommand("kinema.next", Bundle.EMPTY)
        val STOP = SessionCommand("kinema.stop", Bundle.EMPTY)
    }
}
