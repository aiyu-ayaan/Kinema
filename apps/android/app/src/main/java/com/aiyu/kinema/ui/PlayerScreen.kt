package com.aiyu.kinema.ui

import android.app.Activity
import android.content.pm.ActivityInfo
import android.media.AudioManager
import android.media.MediaCodecList
import android.util.Rational
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.displayCutoutPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.PlaylistPlay
import androidx.compose.material.icons.rounded.Fullscreen
import androidx.compose.material.icons.rounded.FullscreenExit
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material.icons.rounded.Forward10
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PictureInPictureAlt
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Replay10
import androidx.compose.material.icons.rounded.Stop
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material.icons.rounded.SubtitlesOff
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.material3.toPath
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableDoubleStateOf
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.graphics.shapes.Morph
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.compose.ContentFrame
import androidx.media3.ui.compose.SURFACE_TYPE_TEXTURE_VIEW
import kotlin.math.roundToInt
import com.aiyu.kinema.R
import com.aiyu.kinema.core.Cue
import com.aiyu.kinema.core.ItemDetail
import com.aiyu.kinema.core.PlayCaps
import com.aiyu.kinema.core.PlaybackPlan
import com.aiyu.kinema.core.Stream
import com.aiyu.kinema.core.StreamSeek
import com.aiyu.kinema.core.parseVtt
import com.aiyu.kinema.playback.Playback
import com.aiyu.kinema.ui.theme.PlayerTheme
import java.util.Locale
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

/** What this phone decodes in hardware or software beyond H.264; the server remuxes or transcodes the rest. */
private val deviceCaps: PlayCaps by lazy {
    val types = MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos.filter { !it.isEncoder }.flatMap { it.supportedTypes.asList() }.toSet()
    PlayCaps(hevc = "video/hevc" in types, av1 = "video/av01" in types)
}

private fun lerp(start: Float, stop: Float, fraction: Float): Float = start + (stop - start) * fraction

/** 3725 -> "1:02:05"; 65 -> "1:05". */
fun clock(seconds: Double): String {
    val s = maxOf(0, seconds.toLong())
    val h = s / 3600
    val m = (s % 3600) / 60
    return if (h > 0) "%d:%02d:%02d".format(Locale.ROOT, h, m, s % 60) else "%d:%02d".format(Locale.ROOT, m, s % 60)
}

/** "English · AC3 5.1 · Commentary" */
@Composable
internal fun trackLabel(s: Stream, n: Int): String {
    val language = s.language?.let { code -> Locale.forLanguageTag(code).displayLanguage.takeIf { it.isNotBlank() && it != code } ?: code.uppercase() }
    val channels = when (val ch = s.channels) {
        null, 0 -> null
        1 -> stringResource(R.string.track_mono)
        2 -> stringResource(R.string.track_stereo)
        else -> "${ch - 1}.1"
    }
    val codec = listOfNotNull(s.codec?.uppercase(), channels.takeIf { s.type == "audio" }).joinToString(" ").ifBlank { null }
    return listOfNotNull(language ?: stringResource(R.string.track_numbered, n), codec, s.title).joinToString(" · ")
}

/** Kept across "next episode", which opens a new player: autoplay stays in full screen like YouTube. */
private var lastFullscreen = false

/**
 * Always-dark playback of one title. Both phones and tablets open in the YouTube-style watch page
 * (WatchScreen.kt) with the up-next list, swipe up or toggle to go full screen, and swipe down to minimize to in-app PiP.
 */
@Composable
fun PlayerScreen(
    id: Long,
    start: Double?,
    isMinimized: Boolean = false,
    onMinimize: () -> Unit = {},
    onExpand: () -> Unit = {},
    onClose: () -> Unit = {},
) {
    // Read once: a reload while playing (watch state changed) must not restart the video.
    val loader = rememberLoad(id) { it.item(id) }
    val large = LocalConfiguration.current.smallestScreenWidthDp >= 600
    var fullscreen by remember { mutableStateOf(lastFullscreen) }
    if (!isMinimized) {
        Immersive(lockLandscape = fullscreen, hideBars = fullscreen)
    }
    BackHandler(fullscreen) { fullscreen = false; lastFullscreen = false }
    BackHandler(!fullscreen && !isMinimized) { onMinimize() }

    PlayerTheme {
        Box(Modifier.fillMaxSize()) {
            when (val s = loader.state.value) {
                Load.Loading -> if (!isMinimized) Box(Modifier.fillMaxSize().background(Color.Black)) { LoadingIndicator(Modifier.align(Alignment.Center).size(72.dp)) }
                is Load.Failed -> if (!isMinimized) Box(Modifier.fillMaxSize().background(Color.Black)) { ErrorState(errorText(s.error), loader.reload) }
                is Load.Ok -> {
                    val detail = s.value
                    if (detail.files.isEmpty()) {
                        if (!isMinimized) Box(Modifier.fillMaxSize().background(Color.Black)) {
                            MessageState(Icons.Rounded.SubtitlesOff, stringResource(R.string.player_no_file), null, action = stringResource(R.string.action_back), onAction = onClose)
                        }
                    } else {
                        val at = remember(detail.item.id) { start ?: if (detail.item.watched) 0.0 else detail.item.positionS ?: 0.0 }
                        key(detail.item.id) {
                            Player(
                                detail = detail,
                                startAt = at,
                                large = large,
                                fullscreen = fullscreen,
                                isMinimized = isMinimized,
                                onMinimize = onMinimize,
                                onExpand = onExpand,
                                onClose = onClose,
                                onFullscreen = { fullscreen = !fullscreen; lastFullscreen = fullscreen },
                            )
                        }
                    }
                }
            }
        }
    }
}

/**
 * What the window looked like before the first player opened. "Next episode" opens the new player before the old one
 * leaves, so only the last player to close puts it back.
 */
private object WindowBefore {
    var players = 0
    var orientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
    var lightStatus = true
    var lightNav = true
}

/** Screen kept on and light bar icons over the dark page; optionally landscape and no system bars. All restored on close. */
@Composable
private fun Immersive(lockLandscape: Boolean, hideBars: Boolean) {
    val activity = LocalContext.current as? Activity ?: return
    val view = LocalView.current
    DisposableEffect(lockLandscape, hideBars) {
        val controller = WindowCompat.getInsetsController(activity.window, view)
        if (WindowBefore.players++ == 0) {
            WindowBefore.orientation = activity.requestedOrientation
            WindowBefore.lightStatus = controller.isAppearanceLightStatusBars
            WindowBefore.lightNav = controller.isAppearanceLightNavigationBars
        }
        activity.requestedOrientation = if (lockLandscape) ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE else WindowBefore.orientation
        controller.isAppearanceLightStatusBars = false
        controller.isAppearanceLightNavigationBars = false
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        if (hideBars) controller.hide(WindowInsetsCompat.Type.systemBars()) else controller.show(WindowInsetsCompat.Type.systemBars())
        view.keepScreenOn = true
        onDispose {
            if (--WindowBefore.players == 0) {
                activity.requestedOrientation = WindowBefore.orientation
                controller.isAppearanceLightStatusBars = WindowBefore.lightStatus
                controller.isAppearanceLightNavigationBars = WindowBefore.lightNav
                controller.show(WindowInsetsCompat.Type.systemBars())
                view.keepScreenOn = false
            }
        }
    }
}

@Composable
private fun Player(
    detail: ItemDetail,
    startAt: Double,
    large: Boolean,
    fullscreen: Boolean,
    isMinimized: Boolean,
    onMinimize: () -> Unit,
    onExpand: () -> Unit,
    onClose: () -> Unit,
    onFullscreen: () -> Unit,
) {
    val context = LocalContext.current
    val repository = LocalRepository.current
    val nav = LocalNavigator.current
    val api = LocalApi.current ?: return
    val scope = rememberCoroutineScope()
    val item = detail.item
    val file = detail.files.first()
    val audios = remember(file) { file.streams.filter { it.type == "audio" } }
    val subs = remember(file) { file.streams.filter { it.type == "subtitle" } }

    val player = remember(api) {
        // Streams go through the server client: its certificate pin and the session header.
        ExoPlayer.Builder(context).setMediaSourceFactory(DefaultMediaSourceFactory(OkHttpDataSource.Factory(api.client)))
            // Pauses for calls and other apps' audio, and when headphones are unplugged, like any media app.
            .setAudioAttributes(AudioAttributes.Builder().setUsage(C.USAGE_MEDIA).setContentType(C.AUDIO_CONTENT_TYPE_MOVIE).build(), true)
            .setHandleAudioBecomingNoisy(true)
            .build()
            .apply {
                // A fixed audio session from the start, so the volume boost can attach to it without a glitch.
                context.getSystemService(AudioManager::class.java)?.generateAudioSessionId()?.takeIf { it > 0 }?.let { audioSessionId = it }
            }
    }
    val pipEnabled by repository.pictureInPicture.collectAsStateWithLifecycle()
    // The notification and lock screen show the title and poster; the poster is fetched once, off the critical path.
    val artwork = remember(item.id) {
        scope.async { withTimeoutOrNull(1_500) { item.poster?.let { key -> repository.request { it.bytes(it.url("/images/$key")) }.getOrNull() } } }
    }

    var plan by remember { mutableStateOf<PlaybackPlan?>(null) }
    // A decoder that exists can still fail a stream (HEVC Main 10 on a Main-only decoder): then ask for H.264.
    var caps by remember { mutableStateOf(deviceCaps) }
    // null until the first plan: the server picks the track from its language settings.
    var audio by remember { mutableStateOf<Int?>(null) }
    var subId by remember { mutableStateOf<Long?>(null) }
    var cues by remember { mutableStateOf<List<Cue>>(emptyList()) }
    // Remux and transcode streams start at `offset`, so the title's time is offset + the player's position.
    var offset by remember { mutableDoubleStateOf(0.0) }
    var time by remember { mutableDoubleStateOf(startAt) }
    var scrub by remember { mutableStateOf<Float?>(null) }
    var scrubTiles by remember { mutableStateOf<List<ScrubTile>>(emptyList()) }
    LaunchedEffect(item.id) {
        var vtt: String? = null
        var tries = 0
        while (vtt.isNullOrBlank() && tries < 30) {
            vtt = repository.request { it.timelineVtt(item.id) }.getOrNull()
            if (vtt == null) break
            if (vtt.isNotBlank()) {
                scrubTiles = parseTimelineVtt(vtt) { relative -> api.spriteUrl(item.id, relative) }
                break
            }
            delay(10_000)
            tries++
        }
    }
    var playing by remember { mutableStateOf(false) }
    var buffering by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var chrome by remember { mutableStateOf(true) }
    var poke by remember { mutableIntStateOf(0) }
    var tracks by remember { mutableStateOf(false) }
    var upNext by remember { mutableStateOf(false) }
    var fill by remember { mutableStateOf(false) }
    var watchedSent by remember { mutableStateOf(false) }
    var loadSeq by remember { mutableIntStateOf(0) }
    val playbackError = stringResource(R.string.player_error)
    val duration = plan?.durationS ?: file.durationS

    fun absTime() = offset + player.currentPosition / 1000.0

    // While a new stream loads, the player still reports the old one: skips build on the pending target instead.
    fun now() = if (buffering) time else absTime()

    fun report(watched: Boolean = false, notify: Boolean = false) {
        val pos = absTime()
        if (pos > 0) repository.saveProgress(item.id, pos, watched, notify)
    }

    fun load(audioIdx: Int?, at: Double) {
        error = null
        buffering = true
        time = at
        val seq = ++loadSeq
        scope.launch {
            repository.request { it.plan(file.id, audioIdx, caps, at) }
                .onSuccess { p ->
                    // A newer seek or track change started meanwhile: its answer wins.
                    if (seq != loadSeq) return@onSuccess
                    // Language settings pick the starting subtitle; later choices are the viewer's.
                    if (plan == null && p.subtitle != null) subId = p.subtitle
                    plan = p
                    audio = p.audio
                    val art = artwork.await()
                    if (seq != loadSeq) return@onSuccess
                    fun media(url: String) = MediaItem.Builder().setUri(url).setMediaMetadata(
                        MediaMetadata.Builder()
                            .setTitle(if (item.kind == "episode") item.showTitle ?: item.title else item.title)
                            .setArtist(if (item.kind == "episode") item.title else item.year?.toString())
                            .apply { if (art != null) setArtworkData(art, MediaMetadata.PICTURE_TYPE_FRONT_COVER) }
                            .build(),
                    ).build()
                    if (p.mode == "direct") {
                        offset = 0.0
                        player.setMediaItem(media(api.streamUrl(file.id, audioIdx, caps, 0.0)), (at * 1000).toLong())
                    } else {
                        // ffmpeg gets the requested time; p.start is where its output really begins.
                        offset = p.start
                        player.setMediaItem(media(api.streamUrl(file.id, audioIdx, caps, at)))
                    }
                    player.prepare()
                    player.play()
                }
                .onFailure { if (seq == loadSeq) { buffering = false; error = it.message ?: playbackError } }
        }
    }

    fun seek(to: Double) {
        val target = to.coerceIn(0.0, duration?.minus(1)?.coerceAtLeast(0.0) ?: Double.MAX_VALUE)
        if (plan?.mode == "direct") {
            time = target
            player.seekTo((target * 1000).toLong())
        } else {
            // Inside what this stream already holds: jump there and leave the server's ffmpeg running. Not while a
            // stream is loading: the player still reports the old one.
            val local = if (buffering) null else StreamSeek.bufferedTargetMs(
                target, offset, player.currentPosition, player.bufferedPosition, player.isCurrentMediaItemSeekable,
            )
            if (local != null) {
                time = target
                player.seekTo(local)
            } else load(audio, target)
        }
    }

    DisposableEffect(player) {
        val listener = object : Player.Listener {
            override fun onIsPlayingChanged(isPlaying: Boolean) { playing = isPlaying }
            override fun onPlaybackStateChanged(state: Int) {
                buffering = state == Player.STATE_BUFFERING
                if (state == Player.STATE_ENDED) {
                    if (!watchedSent) { watchedSent = true; report(true) }
                    detail.nextId?.let { nav.play(it, 0.0) } ?: onClose()
                }
            }
            override fun onPlayerError(e: PlaybackException) {
                val decoder = e.errorCode in PlaybackException.ERROR_CODE_DECODER_INIT_FAILED..PlaybackException.ERROR_CODE_DECODING_FORMAT_UNSUPPORTED
                if (decoder && (caps.hevc || caps.av1)) {
                    caps = PlayCaps(hevc = false, av1 = false)
                    load(audio, now())
                    return
                }
                buffering = false
                error = playbackError
            }
        }
        player.addListener(listener)
        load(null, startAt)
        // The media service keeps this playing with the app in the background, with notification and lock screen controls.
        Playback.attach(
            context, player,
            Playback.Controls(
                stop = { player.pause(); onClose() },
                seekBy = { seek(now() + it) },
                seekTo = { seek(it) },
                durationMs = { ((duration ?: 0.0) * 1000).toLong() },
                currentPositionMs = { (now() * 1000).toLong() },
                next = detail.nextId?.let { next -> { nav.play(next, 0.0) } },
            ),
        )
        onDispose {
            if (!watchedSent) report(notify = true) else repository.notifyChanged()
            player.removeListener(listener)
            Playback.detach(context, player)
            player.release()
        }
    }

    // The clock, and progress every 10s while playing; watched once 92% is seen.
    LaunchedEffect(player) {
        var lastReport = 0L
        while (true) {
            delay(250)
            if (!buffering) time = absTime()
            val d = duration
            if (!watchedSent && d != null && d > 0 && time / d > 0.92) { watchedSent = true; report(true) }
            if (playing && System.currentTimeMillis() - lastReport > 10_000) { lastReport = System.currentTimeMillis(); report() }
        }
    }

    LaunchedEffect(subId) {
        cues = emptyList()
        val id = subId ?: return@LaunchedEffect
        repository.request { it.subtitle(id) }.onSuccess { cues = parseVtt(it) }
    }

    // Controls fade after 3s without a touch while playing.
    LaunchedEffect(poke, playing, tracks, upNext) {
        if (playing && !tracks && !upNext) { delay(3_000); chrome = false }
    }
    val density = LocalDensity.current
    var watchVideoBounds by remember { mutableStateOf<Rect?>(null) }
    val defaultTop = with(density) { WindowInsets.systemBars.asPaddingValues().calculateTopPadding().toPx() }
    val screenWidthPx = with(density) { LocalConfiguration.current.screenWidthDp.dp.toPx() }
    val screenHeightPx = with(density) { LocalConfiguration.current.screenHeightDp.dp.toPx() }
    val startBounds = if (fullscreen) {
        Rect(0f, 0f, screenWidthPx, screenHeightPx)
    } else {
        watchVideoBounds ?: Rect(0f, defaultTop, screenWidthPx, defaultTop + screenWidthPx * 9f / 16f)
    }
    val pipSourceRect = remember(startBounds) {
        android.graphics.Rect(
            startBounds.left.roundToInt(),
            startBounds.top.roundToInt(),
            startBounds.right.roundToInt(),
            startBounds.bottom.roundToInt(),
        )
    }
    val inPip = rememberPictureInPicture(player, enabled = pipEnabled, playing = playing, sourceRect = pipSourceRect)
    val levels = if (!large || fullscreen) rememberSwipeLevels(player) else null
    var swipe by remember { mutableStateOf<Swipe?>(null) }
    val showChrome = !inPip && (chrome || !playing || error != null)

    val onTracks: (() -> Unit)? = if (audios.size > 1 || subs.isNotEmpty()) ({ tracks = true; poke++ }) else null
    var seekHint by remember { mutableStateOf<Pair<Boolean, Int>?>(null) }
    val dockAnim = remember { Animatable(if (isMinimized) 1f else 0f) }
    val motion = MaterialTheme.motionScheme

    LaunchedEffect(isMinimized) {
        dockAnim.animateTo(if (isMinimized) 1f else 0f, motion.defaultSpatialSpec())
    }

    val maxDragPx = with(LocalDensity.current) { 250.dp.toPx() }
    val videoGestures = Modifier.playerVideoGestures(
        isFullscreen = fullscreen,
        levels = levels,
        onSwipeLevel = { swipe = it },
        onEnterFullscreen = { onFullscreen(); poke++ },
        onExitFullscreen = { onFullscreen(); poke++ },
        onPullDownDrag = { dy ->
            scope.launch {
                dockAnim.snapTo((dy / maxDragPx).coerceIn(0f, 1f))
            }
        },
        onPullDownRelease = { passed ->
            scope.launch {
                if (passed) {
                    onMinimize()
                    dockAnim.animateTo(1f, motion.defaultSpatialSpec())
                } else {
                    dockAnim.animateTo(0f, motion.defaultSpatialSpec())
                }
            }
        },
    )

    val keys = remember { FocusRequester() }
    LaunchedEffect(tracks, isMinimized) { if (!tracks && !isMinimized) runCatching { keys.requestFocus() } }
    fun onKey(e: KeyEvent): Boolean {
        if (isMinimized || e.type != KeyEventType.KeyDown || e.isCtrlPressed || e.isAltPressed || e.isMetaPressed || tracks || error != null) return false
        val repeat = e.nativeKeyEvent.repeatCount > 0
        when (e.key) {
            Key.DirectionLeft -> seek(now() - 10)
            Key.DirectionRight -> seek(now() + 10)
            Key.Spacebar, Key.K -> if (!repeat) { if (player.isPlaying) player.pause() else player.play() }
            Key.F, Key.T -> if (!large) return false else if (!repeat) onFullscreen()
            else -> return false
        }
        chrome = true
        poke++
        return true
    }

    val focusModifier = if (!isMinimized) {
        Modifier
            .onPreviewKeyEvent(::onKey)
            .focusRequester(keys)
            .focusable()
    } else Modifier

    BoxWithConstraints(
        Modifier
            .fillMaxSize()
            .then(focusModifier)
    ) {
        val dockProgress = (if (fullscreen) 0f else dockAnim.value).coerceIn(0f, 1f)

        // 1. WatchLayout in background (fades out as video docks into miniplayer)
        if (!fullscreen && !inPip && dockProgress < 1f) {
            Box(
                Modifier
                    .fillMaxSize()
                    .graphicsLayer {
                        alpha = (1f - dockProgress * 2.2f).coerceIn(0f, 1f)
                        translationY = dockProgress.coerceAtLeast(0f) * 80.dp.toPx()
                    }
            ) {
                WatchLayout(detail, onTracks) { modifier ->
                    Spacer(
                        modifier.onGloballyPositioned { coords ->
                            val b = coords.boundsInRoot()
                            if (b.width > 0f && b.height > 0f) {
                                watchVideoBounds = b
                            }
                        }
                    )
                }
            }
        }

        // 2. Single Persistent Video Frame
        val videoModifier = if (fullscreen || inPip) {
            Modifier.fillMaxSize()
        } else {
            val startW = startBounds.width.coerceAtLeast(1f)
            val startH = startBounds.height.coerceAtLeast(1f)
            val startX = startBounds.left
            val startY = startBounds.top

            val miniWidthPx = with(density) { (if (large) 260.dp else 210.dp).toPx() }
            val miniHeightPx = miniWidthPx * 9f / 16f
            val marginEndPx = with(density) { (if (large) 20.dp else 12.dp).toPx() }
            val marginBottomPx = with(density) { (if (large) 20.dp else 76.dp).toPx() }
            val endX = screenWidthPx - miniWidthPx - marginEndPx
            val endY = screenHeightPx - miniHeightPx - marginBottomPx

            val progress = dockProgress.coerceIn(0f, 1f)
            val curW = lerp(startW, miniWidthPx, progress)
            val curH = lerp(startH, miniHeightPx, progress)
            val curX = lerp(startX, endX, progress)
            val curY = lerp(startY, endY, progress)
            val cornerRadius = lerp(0f, with(density) { 16.dp.toPx() }, progress).coerceAtLeast(0f)
            val elevation = lerp(0f, with(density) { 10.dp.toPx() }, progress).coerceAtLeast(0f)

            Modifier
                .offset { IntOffset(curX.roundToInt(), curY.roundToInt()) }
                .size(with(density) { curW.toDp() }, with(density) { curH.toDp() })
                .graphicsLayer {
                    this.shadowElevation = elevation
                    if (progress > 0.01f) {
                        this.shape = RoundedCornerShape(cornerRadius.toDp().coerceAtLeast(0.dp))
                        this.clip = true
                    } else {
                        this.clip = false
                    }
                }
        }

        VideoContainer(
            modifier = videoModifier,
            player = player,
            fill = fill,
            cues = cues,
            time = scrub?.toDouble() ?: time,
            duration = duration,
            scrub = scrub,
            scrubTiles = scrubTiles,
            showChrome = showChrome,
            playing = playing,
            buffering = buffering,
            error = error,
            detail = detail,
            plan = plan,
            subId = subId,
            seekHint = seekHint,
            swipe = swipe,
            levels = levels,
            isFull = fullscreen,
            inPip = inPip,
            dockProgress = dockProgress,
            videoGestures = videoGestures,
            onTap = { chrome = !chrome; poke++ },
            onDoubleTap = { forward ->
                if (forward == null) { if (player.isPlaying) player.pause() else player.play() }
                else {
                    seek(now() + if (forward) 10 else -10)
                    seekHint = forward to ((seekHint?.second ?: 0) + 1)
                }
                poke++
            },
            onBack = {
                if (fullscreen) onFullscreen()
                else onMinimize()
            },
            onTracks = onTracks,
            onFill = { fill = !fill; poke++ },
            onPip = if (pipEnabled && context.findActivity() != null) { { enterPip(context.findActivity()!!, playing, aspect = Rational(16, 9), sourceRect = pipSourceRect) } } else null,
            onList = if (fullscreen) ({ upNext = true; poke++ }) else null,
            onPlay = { if (player.isPlaying) player.pause() else player.play(); poke++ },
            onBack10 = { seek(now() - 10); poke++ },
            onForward10 = { seek(now() + 10); poke++ },
            onScrub = { scrub = it; poke++ },
            onScrubEnd = { scrub?.let { seek(it.toDouble()) }; scrub = null },
            onStop = { player.pause(); onClose() },
            onFullscreen = { onFullscreen(); poke++ },
            onSeek = ::seek,
            onRetry = { load(audio, time) },
            onExpand = onExpand,
            onClose = { player.pause(); onClose() },
            large = large,
            nav = nav,
        )

        if (fullscreen) {
            UpNextPanel(upNext, detail) { upNext = false; poke++ }
        }
    }

    // A pick is saved for this profile, so the web app and other phones start the title the same way.
    fun saveTracks(audioIdx: Int?, subtitle: Long?) {
        val a = audios.indexOfFirst { it.idx == audioIdx }.takeIf { it >= 0 }
        val s = if (subtitle == null) -1 else subs.indexOfFirst { it.id == subtitle }.takeIf { it >= 0 }
        scope.launch { repository.request { it.setTrackChoice(item.id, a, s) } }
    }

    if (tracks) {
        TrackSheet(audios, subs, audio, subId, onDismiss = { tracks = false; poke++ }, onAudio = { idx ->
            tracks = false
            if (idx != audio) { audio = idx; saveTracks(idx, subId); load(idx, now()) }
        }, onSubtitle = { subId = it; tracks = false; saveTracks(audio, it) })
    }
}

/** Tap toggles the controls; a double tap on the outer thirds skips 10s back or forward, in the middle it pauses. */
private fun Modifier.pointerTaps(onTap: () -> Unit, onDoubleTap: (Boolean?) -> Unit) = pointerInput(Unit) {
    detectTapGestures(
        onTap = { onTap() },
        onDoubleTap = { p ->
            val third = size.width / 3f
            onDoubleTap(if (p.x < third) false else if (p.x > 2 * third) true else null)
        },
    )
}

/** "-10s" / "+10s" bubble on the side that was double tapped; repeated taps count up. */
@Composable
private fun androidx.compose.foundation.layout.BoxScope.SeekHint(hint: Pair<Boolean, Int>?) {
    var shown by remember { mutableStateOf<Pair<Boolean, Int>?>(null) }
    LaunchedEffect(hint) {
        if (hint == null) return@LaunchedEffect
        shown = hint
        delay(700)
        shown = null
    }
    val motion = MaterialTheme.motionScheme
    listOf(false, true).forEach { forward ->
        AnimatedVisibility(
            shown?.first == forward,
            Modifier.align(if (forward) Alignment.CenterEnd else Alignment.CenterStart).padding(horizontal = 64.dp),
            enter = scaleIn(motion.fastSpatialSpec(), initialScale = 0.6f) + fadeIn(motion.fastEffectsSpec()),
            exit = scaleOut(motion.defaultSpatialSpec(), targetScale = 1.2f) + fadeOut(motion.defaultEffectsSpec()),
        ) {
            Box(Modifier.size(112.dp).clip(MaterialShapes.Cookie9Sided.toShape()).background(Color.White.copy(alpha = 0.18f)), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(if (forward) Icons.Rounded.Forward10 else Icons.Rounded.Replay10, null, tint = Color.White, modifier = Modifier.size(36.dp))
                    Text(stringResource(R.string.player_seconds, (shown?.second ?: 1) * 10), color = Color.White, style = MaterialTheme.typography.labelLarge)
                }
            }
        }
    }
}

@Composable
private fun androidx.compose.foundation.layout.BoxScope.Subtitles(cues: List<Cue>, time: Double, lifted: Boolean, small: Boolean = false) {
    val active = cues.filter { it.start <= time && time < it.end }
    if (active.isEmpty()) return
    val lift by animateFloatAsState(if (lifted) 1f else 0f, MaterialTheme.motionScheme.defaultSpatialSpec(), label = "subLift")
    Column(
        Modifier.align(Alignment.BottomCenter).padding(horizontal = if (small) 8.dp else 48.dp).padding(bottom = if (small) 6.dp else (28 + 84 * lift).dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        active.forEach {
            Text(
                it.text,
                color = Color.White,
                fontSize = if (small) 11.sp else 20.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.titleLarge.copy(shadow = Shadow(Color.Black, blurRadius = 8f)),
                modifier = Modifier.background(Color.Black.copy(alpha = 0.4f), MaterialTheme.shapes.extraSmall).padding(horizontal = 8.dp, vertical = 2.dp),
            )
        }
    }
}

@Composable
private fun TopBar(
    detail: ItemDetail,
    plan: PlaybackPlan?,
    showTitle: Boolean,
    onBack: () -> Unit,
    onOpen: (Long) -> Unit,
    onTracks: (() -> Unit)?,
    subtitlesOn: Boolean,
    fill: Boolean,
    onFill: () -> Unit,
    onPip: (() -> Unit)?,
    onList: (() -> Unit)?,
    isTablet: Boolean = false,
    isFullscreen: Boolean = false,
) {
    val item = detail.item
    val title = if (item.kind == "episode") item.showTitle ?: item.title else item.title
    val subtitle = if (item.kind == "episode") listOfNotNull(subtitleOf(item), item.title).joinToString(" · ") else item.year?.toString()
    Row(
        Modifier
            .fillMaxWidth()
            .displayCutoutPadding()
            .padding(horizontal = 8.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        val backIcon = if (!isFullscreen) Icons.Rounded.ExpandMore else Icons.AutoMirrored.Rounded.ArrowBack
        val backLabel = if (!isFullscreen) stringResource(R.string.player_minimize) else stringResource(R.string.action_back)
        IconButton(onBack, modifier = Modifier.size(40.dp)) { Icon(backIcon, backLabel, tint = Color.White) }

        if (showTitle) {
            Column(Modifier.weight(1f).padding(horizontal = 8.dp)) {
                Text(title, style = MaterialTheme.typography.titleLarge, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (item.kind == "episode" && item.episodeNo != null) {
                    EpisodeLinks(item, onOpen, Color.White, " · ${item.title}", MaterialTheme.typography.bodyMedium, Modifier, Color.White.copy(alpha = 0.7f))
                } else if (subtitle != null) {
                    Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = Color.White.copy(alpha = 0.7f), maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        } else {
            Spacer(Modifier.weight(1f))
        }

        plan?.let {
            val mode = when (it.mode) {
                "direct" -> R.string.player_mode_direct
                "remux" -> R.string.player_mode_remux
                else -> R.string.player_mode_transcode
            }
            Surface(
                color = Color.White.copy(alpha = 0.15f),
                contentColor = Color.White,
                shape = CircleShape,
                modifier = Modifier.padding(end = 4.dp),
            ) {
                Text(
                    stringResource(mode),
                    style = MaterialTheme.typography.labelSmall,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                )
            }
        }
        onPip?.let { IconButton(it, modifier = Modifier.size(40.dp)) { Icon(Icons.Rounded.PictureInPictureAlt, stringResource(R.string.player_pip), tint = Color.White) } }
        IconButton(onFill, modifier = Modifier.size(40.dp)) { Icon(Icons.Rounded.AspectRatio, stringResource(if (fill) R.string.player_fit else R.string.player_fill), tint = if (fill) MaterialTheme.colorScheme.primary else Color.White) }
        onTracks?.let { IconButton(it, modifier = Modifier.size(40.dp)) { Icon(if (subtitlesOn) Icons.Rounded.Subtitles else Icons.Rounded.SubtitlesOff, stringResource(R.string.player_tracks), tint = Color.White) } }
        onList?.let { IconButton(it, modifier = Modifier.size(40.dp)) { Icon(Icons.AutoMirrored.Rounded.PlaylistPlay, stringResource(R.string.player_up_next_list), tint = Color.White) } }
    }
}

/**
 * Play / pause on a shape that morphs with a spring: a soft cookie while paused (inviting a tap), a circle while
 * playing. Skip buttons beside it.
 */
@Composable
private fun CenterControls(playing: Boolean, onPlay: () -> Unit, onBack10: () -> Unit, onForward10: () -> Unit, small: Boolean, modifier: Modifier = Modifier) {
    val morph = remember { Morph(MaterialShapes.Cookie9Sided, MaterialShapes.Circle) }
    val progress = remember { Animatable(if (playing) 1f else 0f) }
    val spring = MaterialTheme.motionScheme.defaultSpatialSpec<Float>()
    LaunchedEffect(playing) { progress.animateTo(if (playing) 1f else 0f, spring) }
    val source = remember { MutableInteractionSource() }
    val skip = if (small) 44.dp else 52.dp
    val skipIcon = if (small) 28.dp else 34.dp
    val playSize = if (small) 56.dp else 76.dp
    val playIcon = if (small) 28.dp else 38.dp
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(if (small) 24.dp else 48.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onBack10, Modifier.size(skip)) { Icon(Icons.Rounded.Replay10, stringResource(R.string.player_back10), tint = Color.White, modifier = Modifier.size(skipIcon)) }
        Box(
            Modifier.size(playSize).pressScale(source).clip(MorphShape(morph) { progress.value }).background(MaterialTheme.colorScheme.primary)
                .clickable(source, null, onClick = onPlay)
                .graphicsLayer { rotationZ = (1f - progress.value) * 30f },
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                if (playing) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                stringResource(if (playing) R.string.player_pause else R.string.action_play),
                tint = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.size(playIcon).graphicsLayer { rotationZ = -(1f - progress.value) * 30f },
            )
        }
        IconButton(onForward10, Modifier.size(skip)) { Icon(Icons.Rounded.Forward10, stringResource(R.string.player_forward10), tint = Color.White, modifier = Modifier.size(skipIcon)) }
    }
}

/** A Material shape morph as a clip shape; [progress] is read at draw time so the spring never recomposes. */
private class MorphShape(private val morph: Morph, private val progress: () -> Float) : Shape {
    private val matrix = Matrix()
    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline {
        val path = morph.toPath(progress(), Path())
        matrix.reset()
        matrix.scale(size.width, size.height)
        path.transform(matrix)
        return Outline.Generic(path)
    }
}

@Composable
private fun BottomBar(
    time: Double,
    duration: Double?,
    scrub: Float?,
    scrubTiles: List<ScrubTile>,
    onScrub: (Float) -> Unit,
    onScrubEnd: () -> Unit,
    onStop: () -> Unit,
    fullscreen: Boolean?,
    onFullscreen: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier
            .fillMaxWidth()
            .navigationBarsPadding()
            .displayCutoutPadding()
            .padding(horizontal = 16.dp, vertical = 4.dp),
    ) {
        if (scrub != null && duration != null && duration > 0) {
            val frac = (scrub / duration.toFloat()).coerceIn(0.05f, 0.95f)
            Box(
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 48.dp),
            ) {
                Box(
                    Modifier
                        .fillMaxWidth(frac)
                        .wrapContentWidth(Alignment.End),
                ) {
                    ScrubThumbnail(
                        tiles = scrubTiles,
                        time = scrub.toDouble(),
                        modifier = Modifier.padding(bottom = 6.dp),
                    )
                }
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            // Stop: saves where you are and leaves the player (the web player's stop button).
            IconButton(onStop, modifier = Modifier.size(36.dp)) { Icon(Icons.Rounded.Stop, stringResource(R.string.player_stop), tint = Color.White) }
            Text(clock(time), style = MaterialTheme.typography.labelMedium, color = Color.White)
            if (duration != null && duration > 0) {
                Slider(
                    value = time.toFloat().coerceIn(0f, duration.toFloat()),
                    onValueChange = onScrub,
                    onValueChangeFinished = onScrubEnd,
                    valueRange = 0f..duration.toFloat(),
                    colors = SliderDefaults.colors(
                        thumbColor = MaterialTheme.colorScheme.primary,
                        activeTrackColor = MaterialTheme.colorScheme.primary,
                        inactiveTrackColor = Color.White.copy(alpha = 0.28f),
                    ),
                    track = { state ->
                        SliderDefaults.Track(
                            sliderState = state,
                            modifier = Modifier.height(4.dp),
                            thumbTrackGapSize = 0.dp,
                            trackInsideCornerSize = 0.dp,
                            drawStopIndicator = null,
                            colors = SliderDefaults.colors(
                                activeTrackColor = MaterialTheme.colorScheme.primary,
                                inactiveTrackColor = Color.White.copy(alpha = 0.28f),
                            ),
                        )
                    },
                    thumb = {
                        Box(
                            Modifier
                                .size(14.dp)
                                .clip(CircleShape)
                                .background(MaterialTheme.colorScheme.primary),
                        )
                    },
                    modifier = Modifier.weight(1f),
                )
                Text(clock(duration), style = MaterialTheme.typography.labelMedium, color = Color.White.copy(alpha = 0.7f))
            } else Spacer(Modifier.weight(1f))
            // Tablets only: a phone is always full screen.
            fullscreen?.let {
                IconButton(onFullscreen, modifier = Modifier.size(36.dp)) {
                    Icon(if (it) Icons.Rounded.FullscreenExit else Icons.Rounded.Fullscreen, stringResource(if (it) R.string.player_exit_fullscreen else R.string.player_fullscreen), tint = Color.White)
                }
            }
        }
    }
}

@Composable
private fun TrackSheet(audios: List<Stream>, subs: List<Stream>, audio: Int?, subId: Long?, onDismiss: () -> Unit, onAudio: (Int) -> Unit, onSubtitle: (Long?) -> Unit) {
    val sheet = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    ModalBottomSheet(onDismiss, sheetState = sheet) {
        Row(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp).padding(bottom = 24.dp), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            if (audios.size > 1) {
                Column(Modifier.weight(1f)) {
                    Text(stringResource(R.string.player_audio), style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(8.dp))
                    audios.forEachIndexed { i, a -> TrackRow(trackLabel(a, i + 1), a.idx == audio) { onAudio(a.idx) } }
                }
            }
            Column(Modifier.weight(1f)) {
                Text(stringResource(R.string.player_subtitles), style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(8.dp))
                TrackRow(stringResource(R.string.player_subtitles_off), subId == null) { onSubtitle(null) }
                subs.forEachIndexed { i, s ->
                    TrackRow(trackLabel(s, i + 1), s.id == subId, enabled = s.isTextSubtitle, note = if (s.isTextSubtitle) null else stringResource(R.string.player_subtitle_image)) { onSubtitle(s.id) }
                }
            }
        }
    }
}

@Composable
internal fun TrackRow(label: String, selected: Boolean, enabled: Boolean = true, note: String? = null, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    ListItem(
        headlineContent = { Text(label, maxLines = 4, overflow = TextOverflow.Ellipsis) },
        supportingContent = note?.let { { Text(it) } },
        leadingContent = { Icon(Icons.Rounded.Check, null, tint = if (selected) c.primary else Color.Transparent) },
        colors = ListItemDefaults.colors(containerColor = if (selected) c.secondaryContainer else Color.Transparent),
        modifier = Modifier.clip(MaterialTheme.shapes.large).clickable(enabled = enabled, onClick = onClick).graphicsLayer { alpha = if (enabled) 1f else 0.4f },
    )
}

@Composable
private fun VideoContainer(
    modifier: Modifier,
    player: ExoPlayer,
    fill: Boolean,
    cues: List<Cue>,
    time: Double,
    duration: Double?,
    scrub: Float?,
    scrubTiles: List<ScrubTile>,
    showChrome: Boolean,
    playing: Boolean,
    buffering: Boolean,
    error: String?,
    detail: ItemDetail,
    plan: PlaybackPlan?,
    subId: Long?,
    seekHint: Pair<Boolean, Int>?,
    swipe: Swipe?,
    levels: SwipeLevels?,
    isFull: Boolean,
    inPip: Boolean,
    dockProgress: Float,
    videoGestures: Modifier,
    onTap: () -> Unit,
    onDoubleTap: (Boolean?) -> Unit,
    onBack: () -> Unit,
    onTracks: (() -> Unit)?,
    onFill: () -> Unit,
    onPip: (() -> Unit)?,
    onList: (() -> Unit)?,
    onPlay: () -> Unit,
    onBack10: () -> Unit,
    onForward10: () -> Unit,
    onScrub: (Float) -> Unit,
    onScrubEnd: () -> Unit,
    onStop: () -> Unit,
    onFullscreen: () -> Unit,
    onSeek: (Double) -> Unit,
    onRetry: () -> Unit,
    onExpand: () -> Unit,
    onClose: () -> Unit,
    large: Boolean,
    nav: Navigator,
) {
    val file = detail.files.first()
    val progress = if (inPip) 0f else dockProgress.coerceIn(0f, 1f)
    val isMini = !inPip && progress >= 0.95f
    val fullAlpha = if (inPip) 0f else (1f - progress * 2.5f).coerceIn(0f, 1f)
    val miniAlpha = if (inPip) 0f else ((progress - 0.4f) / 0.6f).coerceIn(0f, 1f)

    Box(
        modifier = modifier
            .background(Color.Black)
            .then(
                if (inPip) {
                    Modifier
                } else if (isMini) {
                    Modifier
                        .miniPlayerGestures(onExpand = onExpand, onClose = onClose)
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            onClick = onExpand,
                        )
                } else {
                    Modifier
                        .then(videoGestures)
                        .then(if (progress < 0.05f) Modifier.pointerTaps(onTap = onTap, onDoubleTap = onDoubleTap) else Modifier)
                }
            )
    ) {
        ContentFrame(
            player = player,
            modifier = Modifier.fillMaxSize(),
            surfaceType = SURFACE_TYPE_TEXTURE_VIEW,
            contentScale = if (fill && !inPip) ContentScale.Crop else ContentScale.Fit,
        )

        Subtitles(cues, time, lifted = showChrome && progress < 0.2f && !inPip, small = inPip || progress > 0.5f)

        // Full player overlays
        if (fullAlpha > 0f) {
            Box(Modifier.fillMaxSize().graphicsLayer { alpha = fullAlpha }) {
                SeekHint(seekHint)
                if (levels != null) SwipeIndicator(swipe, levels)

                if (buffering && error == null) LoadingIndicator(Modifier.align(Alignment.Center).size(80.dp))

                error?.let {
                    Surface(
                        color = Color.Black.copy(alpha = 0.8f),
                        shape = MaterialTheme.shapes.extraLarge,
                        modifier = Modifier.align(Alignment.Center).widthIn(max = 420.dp).padding(24.dp),
                    ) {
                        Column(
                            Modifier.padding(24.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            Text(stringResource(R.string.player_error_title), style = MaterialTheme.typography.titleLarge)
                            Text(it, style = MaterialTheme.typography.bodyMedium, color = Color.White.copy(alpha = 0.7f), textAlign = TextAlign.Center)
                            Button(onRetry) { Text(stringResource(R.string.action_retry)) }
                        }
                    }
                }

                val effects = MaterialTheme.motionScheme
                AnimatedVisibility(
                    showChrome,
                    Modifier.fillMaxSize(),
                    enter = fadeIn(effects.defaultEffectsSpec()),
                    exit = fadeOut(effects.slowEffectsSpec()),
                ) {
                    Box(Modifier.fillMaxSize()) {
                        Box(
                            Modifier.fillMaxSize().background(
                                Brush.verticalGradient(
                                    0f to Color.Black.copy(alpha = 0.75f),
                                    0.3f to Color.Transparent,
                                    0.6f to Color.Transparent,
                                    1f to Color.Black.copy(alpha = 0.85f),
                                ),
                            ),
                        )
                        TopBar(
                            detail = detail,
                            plan = plan,
                            showTitle = isFull,
                            onBack = onBack,
                            onOpen = nav.open,
                            onTracks = onTracks,
                            subtitlesOn = subId != null,
                            fill = fill,
                            onFill = onFill,
                            onPip = onPip,
                            onList = onList,
                            isTablet = large,
                            isFullscreen = isFull,
                        )
                        if (error == null) CenterControls(
                            playing = playing,
                            onPlay = onPlay,
                            onBack10 = onBack10,
                            onForward10 = onForward10,
                            small = !isFull,
                            modifier = Modifier.align(Alignment.Center),
                        )
                        BottomBar(
                            time = time,
                            duration = duration,
                            scrub = scrub,
                            scrubTiles = scrubTiles,
                            onScrub = onScrub,
                            onScrubEnd = onScrubEnd,
                            onStop = onStop,
                            fullscreen = isFull,
                            onFullscreen = onFullscreen,
                            modifier = Modifier.align(Alignment.BottomCenter),
                        )
                    }
                }

                if (error == null) {
                    val intro = file.intro
                    val credits = file.credits
                    val remaining = duration?.let { it - time }
                    val inCredits = credits != null && time >= credits.startS && (remaining ?: 0.0) > 2
                    val due = remaining != null && remaining >= 0 && if (credits != null) time >= credits.startS else remaining <= UP_NEXT_S
                    val span = if (credits != null && duration != null) duration - credits.startS else UP_NEXT_S
                    SkipPill(stringResource(R.string.player_skip_intro), intro != null && time >= intro.startS && time < intro.endS - 2, lifted = showChrome) { intro?.let { onSeek(it.endS) } }
                    SkipPill(stringResource(R.string.player_skip_credits), inCredits && detail.nextId == null, lifted = showChrome, onClick = onStop)
                    NextEpisodeCard(detail.nextId, due, remaining, span, lifted = showChrome, compact = !isFull) { detail.nextId?.let { nav.play(it, 0.0) } }
                }
            }
        }

        // Mini player overlays
        if (miniAlpha > 0f) {
            var miniControlsVisible by remember { mutableStateOf(false) }
            var miniControlsKey by remember { mutableIntStateOf(0) }

            LaunchedEffect(isMini, miniControlsKey) {
                if (isMini) {
                    miniControlsVisible = true
                    delay(2500)
                    miniControlsVisible = false
                }
            }

            Box(
                Modifier
                    .fillMaxSize()
                    .graphicsLayer { alpha = miniAlpha }
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                        onClick = onExpand,
                    ),
            ) {
                Box(
                    Modifier
                        .fillMaxSize()
                        .clip(RoundedCornerShape(16.dp))
                        .border(BorderStroke(1.dp, Color.White.copy(alpha = 0.15f * miniAlpha)), RoundedCornerShape(16.dp)),
                )
                AnimatedVisibility(
                    visible = miniControlsVisible,
                    enter = fadeIn(tween(200)),
                    exit = fadeOut(tween(300)),
                ) {
                    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
                        if (buffering) {
                            LoadingIndicator(
                                modifier = Modifier.align(Alignment.Center).size(36.dp),
                                color = Color.White,
                            )
                        } else {
                            Box(
                                modifier = Modifier
                                    .align(Alignment.Center)
                                    .size(44.dp)
                                    .clip(CircleShape)
                                    .background(Color.Black.copy(alpha = 0.55f))
                                    .clickable {
                                        onPlay()
                                        miniControlsKey++
                                    },
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    if (playing) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                                    contentDescription = stringResource(if (playing) R.string.player_pause else R.string.action_play),
                                    tint = Color.White,
                                    modifier = Modifier.size(24.dp),
                                )
                            }
                        }
                        Box(
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .padding(6.dp)
                                .size(32.dp)
                                .clip(CircleShape)
                                .background(Color.Black.copy(alpha = 0.55f))
                                .clickable(onClick = onClose),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(
                                Icons.Rounded.Close,
                                contentDescription = stringResource(R.string.player_dismiss),
                                tint = Color.White,
                                modifier = Modifier.size(18.dp),
                            )
                        }
                    }
                }
                val progressFraction = if (duration != null && duration > 0) (time / duration).toFloat().coerceIn(0f, 1f) else 0f
                LinearProgressIndicator(
                    progress = { progressFraction },
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(3.dp),
                    color = MaterialTheme.colorScheme.primary,
                    trackColor = Color.White.copy(alpha = 0.2f),
                )
            }
        }
    }
}

