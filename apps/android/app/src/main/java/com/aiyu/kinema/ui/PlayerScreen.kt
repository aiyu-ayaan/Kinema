package com.aiyu.kinema.ui

import android.app.Activity
import android.content.pm.ActivityInfo
import android.media.MediaCodecList
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.displayCutoutPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Forward10
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Replay10
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material.icons.rounded.SubtitlesOff
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.material3.toPath
import androidx.compose.material3.toShape
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableDoubleStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.graphics.shapes.Morph
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.compose.ContentFrame
import com.aiyu.kinema.R
import com.aiyu.kinema.core.Cue
import com.aiyu.kinema.core.ItemDetail
import com.aiyu.kinema.core.PlayCaps
import com.aiyu.kinema.core.PlaybackPlan
import com.aiyu.kinema.core.Stream
import com.aiyu.kinema.core.parseVtt
import com.aiyu.kinema.ui.theme.PlayerTheme
import java.util.Locale
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** What this phone decodes in hardware or software beyond H.264; the server remuxes or transcodes the rest. */
private val deviceCaps: PlayCaps by lazy {
    val types = MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos.filter { !it.isEncoder }.flatMap { it.supportedTypes.asList() }.toSet()
    PlayCaps(hevc = "video/hevc" in types, av1 = "video/av01" in types)
}

/** 3725 -> "1:02:05"; 65 -> "1:05". */
fun clock(seconds: Double): String {
    val s = maxOf(0, seconds.toLong())
    val h = s / 3600
    val m = (s % 3600) / 60
    return if (h > 0) "%d:%02d:%02d".format(Locale.ROOT, h, m, s % 60) else "%d:%02d".format(Locale.ROOT, m, s % 60)
}

/** "English · AC3 5.1 · Commentary" */
@Composable
private fun trackLabel(s: Stream, n: Int): String {
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

/** Full-screen, always-dark playback of one title. Turns the phone sideways and hides the system bars while open. */
@Composable
fun PlayerScreen(id: Long, start: Double?) {
    // Read once: a reload while playing (watch state changed) must not restart the video.
    val loader = rememberLoad(id) { it.item(id) }
    val nav = LocalNavigator.current
    ImmersiveLandscape()
    PlayerTheme {
        Box(Modifier.fillMaxSize().background(Color.Black)) {
            when (val s = loader.state.value) {
                Load.Loading -> LoadingIndicator(Modifier.align(Alignment.Center).size(72.dp))
                is Load.Failed -> ErrorState(errorText(s.error), loader.reload)
                is Load.Ok -> {
                    val detail = s.value
                    if (detail.files.isEmpty()) MessageState(Icons.Rounded.SubtitlesOff, stringResource(R.string.player_no_file), null, action = stringResource(R.string.action_back), onAction = nav.back)
                    else {
                        val at = remember(detail.item.id) { start ?: if (detail.item.watched) 0.0 else detail.item.positionS ?: 0.0 }
                        Player(detail, at)
                    }
                }
            }
        }
    }
}

/** Landscape, no system bars, screen kept on; all restored when the player closes. */
@Composable
private fun ImmersiveLandscape() {
    val activity = LocalContext.current as? Activity ?: return
    val view = LocalView.current
    DisposableEffect(Unit) {
        val orientation = activity.requestedOrientation
        activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
        val controller = WindowCompat.getInsetsController(activity.window, view)
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        controller.hide(WindowInsetsCompat.Type.systemBars())
        view.keepScreenOn = true
        onDispose {
            activity.requestedOrientation = orientation
            controller.show(WindowInsetsCompat.Type.systemBars())
            view.keepScreenOn = false
        }
    }
}

@Composable
private fun Player(detail: ItemDetail, startAt: Double) {
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
        ExoPlayer.Builder(context).setMediaSourceFactory(DefaultMediaSourceFactory(OkHttpDataSource.Factory(api.client))).build()
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
    var playing by remember { mutableStateOf(false) }
    var buffering by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var chrome by remember { mutableStateOf(true) }
    var poke by remember { mutableIntStateOf(0) }
    var tracks by remember { mutableStateOf(false) }
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
                    if (p.mode == "direct") {
                        offset = 0.0
                        player.setMediaItem(MediaItem.fromUri(api.streamUrl(file.id, audioIdx, caps, 0.0)), (at * 1000).toLong())
                    } else {
                        // ffmpeg gets the requested time; p.start is where its output really begins.
                        offset = p.start
                        player.setMediaItem(MediaItem.fromUri(api.streamUrl(file.id, audioIdx, caps, at)))
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
        } else load(audio, target)
    }

    DisposableEffect(player) {
        val listener = object : Player.Listener {
            override fun onIsPlayingChanged(isPlaying: Boolean) { playing = isPlaying }
            override fun onPlaybackStateChanged(state: Int) {
                buffering = state == Player.STATE_BUFFERING
                if (state == Player.STATE_ENDED) {
                    if (!watchedSent) { watchedSent = true; report(true) }
                    detail.nextId?.let { nav.play(it, 0.0) } ?: nav.back()
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
        onDispose {
            if (!watchedSent) report(notify = true) else repository.notifyChanged()
            player.removeListener(listener)
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
    LaunchedEffect(poke, playing, tracks) {
        if (playing && !tracks) { delay(3_000); chrome = false }
    }
    val showChrome = chrome || !playing || error != null

    var seekHint by remember { mutableStateOf<Pair<Boolean, Int>?>(null) }
    Box(
        Modifier.fillMaxSize().background(Color.Black).pointerTaps(
            onTap = { chrome = !chrome; poke++ },
            onDoubleTap = { forward ->
                if (forward == null) { if (player.isPlaying) player.pause() else player.play() }
                else {
                    seek(now() + if (forward) 10 else -10)
                    seekHint = forward to ((seekHint?.second ?: 0) + 1)
                }
                poke++
            },
        ),
    ) {
        ContentFrame(player, Modifier.fillMaxSize(), contentScale = if (fill) ContentScale.Crop else ContentScale.Fit)

        Subtitles(cues, time, lifted = showChrome)
        SeekHint(seekHint)

        if (buffering && error == null) LoadingIndicator(Modifier.align(Alignment.Center).size(80.dp))

        error?.let {
            Surface(color = Color.Black.copy(alpha = 0.8f), shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.align(Alignment.Center).widthIn(max = 420.dp).padding(24.dp)) {
                Column(Modifier.padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(stringResource(R.string.player_error_title), style = MaterialTheme.typography.titleLarge)
                    Text(it, style = MaterialTheme.typography.bodyMedium, color = Color.White.copy(alpha = 0.7f), textAlign = TextAlign.Center)
                    Button({ load(audio, time) }) { Text(stringResource(R.string.action_retry)) }
                }
            }
        }

        val effects = MaterialTheme.motionScheme
        AnimatedVisibility(showChrome, Modifier.fillMaxSize(), enter = fadeIn(effects.defaultEffectsSpec()), exit = fadeOut(effects.slowEffectsSpec())) {
            Box(Modifier.fillMaxSize()) {
                Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0f to Color.Black.copy(alpha = 0.75f), 0.3f to Color.Transparent, 0.6f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.85f))))
                TopBar(detail, plan, onBack = nav.back, onTracks = if (audios.size > 1 || subs.isNotEmpty()) ({ tracks = true }) else null, subtitlesOn = subId != null, fill = fill, onFill = { fill = !fill; poke++ })
                if (error == null) CenterControls(
                    playing = playing,
                    onPlay = { if (player.isPlaying) player.pause() else player.play(); poke++ },
                    onBack10 = { seek(now() - 10); poke++ },
                    onForward10 = { seek(now() + 10); poke++ },
                    modifier = Modifier.align(Alignment.Center),
                )
                BottomBar(
                    time = scrub?.toDouble() ?: time,
                    duration = duration,
                    onScrub = { scrub = it; poke++ },
                    onScrubEnd = { scrub?.let { seek(it.toDouble()) }; scrub = null },
                    next = detail.nextId?.let { next -> { nav.play(next, 0.0) } },
                    modifier = Modifier.align(Alignment.BottomCenter),
                )
            }
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
private fun androidx.compose.foundation.layout.BoxScope.Subtitles(cues: List<Cue>, time: Double, lifted: Boolean) {
    val active = cues.filter { it.start <= time && time < it.end }
    if (active.isEmpty()) return
    val lift by animateFloatAsState(if (lifted) 1f else 0f, MaterialTheme.motionScheme.defaultSpatialSpec(), label = "subLift")
    Column(
        Modifier.align(Alignment.BottomCenter).padding(horizontal = 48.dp).padding(bottom = (28 + 84 * lift).dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        active.forEach {
            Text(
                it.text,
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.titleLarge.copy(shadow = Shadow(Color.Black, blurRadius = 8f)),
                modifier = Modifier.background(Color.Black.copy(alpha = 0.4f), MaterialTheme.shapes.extraSmall).padding(horizontal = 8.dp, vertical = 2.dp),
            )
        }
    }
}

@Composable
private fun TopBar(detail: ItemDetail, plan: PlaybackPlan?, onBack: () -> Unit, onTracks: (() -> Unit)?, subtitlesOn: Boolean, fill: Boolean, onFill: () -> Unit) {
    val item = detail.item
    val title = if (item.kind == "episode") item.showTitle ?: item.title else item.title
    val subtitle = if (item.kind == "episode") listOfNotNull(subtitleOf(item), item.title).joinToString(" · ") else item.year?.toString()
    Row(Modifier.fillMaxWidth().displayCutoutPadding().padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onBack) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, stringResource(R.string.action_back), tint = Color.White) }
        Column(Modifier.weight(1f).padding(horizontal = 8.dp)) {
            Text(title, style = MaterialTheme.typography.titleLarge, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (subtitle != null) Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = Color.White.copy(alpha = 0.7f), maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        plan?.let {
            val mode = when (it.mode) {
                "direct" -> R.string.player_mode_direct
                "remux" -> R.string.player_mode_remux
                else -> R.string.player_mode_transcode
            }
            Tag(stringResource(mode), container = Color.White.copy(alpha = 0.15f), content = Color.White)
        }
        IconButton(onFill) { Icon(Icons.Rounded.AspectRatio, stringResource(if (fill) R.string.player_fit else R.string.player_fill), tint = if (fill) MaterialTheme.colorScheme.primary else Color.White) }
        onTracks?.let { IconButton(it) { Icon(if (subtitlesOn) Icons.Rounded.Subtitles else Icons.Rounded.SubtitlesOff, stringResource(R.string.player_tracks), tint = Color.White) } }
    }
}

/**
 * Play / pause on a shape that morphs with a spring: a soft cookie while paused (inviting a tap), a circle while
 * playing. Skip buttons beside it.
 */
@Composable
private fun CenterControls(playing: Boolean, onPlay: () -> Unit, onBack10: () -> Unit, onForward10: () -> Unit, modifier: Modifier = Modifier) {
    val morph = remember { Morph(MaterialShapes.Cookie9Sided, MaterialShapes.Circle) }
    val progress = remember { Animatable(if (playing) 1f else 0f) }
    val spring = MaterialTheme.motionScheme.defaultSpatialSpec<Float>()
    LaunchedEffect(playing) { progress.animateTo(if (playing) 1f else 0f, spring) }
    val source = remember { MutableInteractionSource() }
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(40.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onBack10, Modifier.size(64.dp)) { Icon(Icons.Rounded.Replay10, stringResource(R.string.player_back10), tint = Color.White, modifier = Modifier.size(40.dp)) }
        Box(
            Modifier.size(88.dp).pressScale(source).clip(MorphShape(morph) { progress.value }).background(MaterialTheme.colorScheme.primary)
                .clickable(source, null, onClick = onPlay)
                .graphicsLayer { rotationZ = (1f - progress.value) * 30f },
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                if (playing) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                stringResource(if (playing) R.string.player_pause else R.string.action_play),
                tint = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.size(44.dp).graphicsLayer { rotationZ = -(1f - progress.value) * 30f },
            )
        }
        IconButton(onForward10, Modifier.size(64.dp)) { Icon(Icons.Rounded.Forward10, stringResource(R.string.player_forward10), tint = Color.White, modifier = Modifier.size(40.dp)) }
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
private fun BottomBar(time: Double, duration: Double?, onScrub: (Float) -> Unit, onScrubEnd: () -> Unit, next: (() -> Unit)?, modifier: Modifier = Modifier) {
    val motion = MaterialTheme.motionScheme
    Column(modifier.fillMaxWidth().navigationBarsPadding().displayCutoutPadding().padding(horizontal = 24.dp, vertical = 12.dp)) {
        AnimatedVisibility(next != null, enter = slideInVertically(motion.defaultSpatialSpec()) { it } + fadeIn(), exit = slideOutVertically { it } + fadeOut()) {
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                TextButton({ next?.invoke() }) {
                    Text(stringResource(R.string.player_next), color = Color.White)
                    Spacer(Modifier.width(6.dp))
                    Icon(Icons.Rounded.SkipNext, null, tint = Color.White)
                }
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(clock(time), style = MaterialTheme.typography.labelLarge, color = Color.White)
            if (duration != null && duration > 0) {
                Slider(
                    value = time.toFloat().coerceIn(0f, duration.toFloat()),
                    onValueChange = onScrub,
                    onValueChangeFinished = onScrubEnd,
                    valueRange = 0f..duration.toFloat(),
                    colors = SliderDefaults.colors(inactiveTrackColor = Color.White.copy(alpha = 0.25f)),
                    modifier = Modifier.weight(1f),
                )
                Text(clock(duration), style = MaterialTheme.typography.labelLarge, color = Color.White.copy(alpha = 0.7f))
            } else Spacer(Modifier.weight(1f))
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
private fun TrackRow(label: String, selected: Boolean, enabled: Boolean = true, note: String? = null, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    ListItem(
        headlineContent = { Text(label, maxLines = 2, overflow = TextOverflow.Ellipsis) },
        supportingContent = note?.let { { Text(it) } },
        leadingContent = { Icon(Icons.Rounded.Check, null, tint = if (selected) c.primary else Color.Transparent) },
        colors = ListItemDefaults.colors(containerColor = if (selected) c.secondaryContainer else Color.Transparent),
        modifier = Modifier.clip(MaterialTheme.shapes.large).clickable(enabled = enabled, onClick = onClick).graphicsLayer { alpha = if (enabled) 1f else 0.4f },
    )
}
