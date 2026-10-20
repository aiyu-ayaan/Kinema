package com.aiyu.kinema.ui

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.InteractionSource
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.toPath
import androidx.compose.material3.toShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.graphics.shapes.Morph
import androidx.graphics.shapes.RoundedPolygon
import coil3.compose.AsyncImage
import com.aiyu.kinema.R
import com.aiyu.kinema.core.ItemCard
import com.aiyu.kinema.ui.theme.rememberReducedMotion
import kotlinx.coroutines.delay

private val Emphasized = CubicBezierEasing(0.2f, 0f, 0f, 1f)

/**
 * The brand mark: morphs through Material shapes while slowly turning. Each segment morphs for 60% of its time,
 * then rests, so it reads as breathing rather than spinning. Still when system animations are off.
 */
@Composable
fun MorphingShape(modifier: Modifier = Modifier) {
    val c = MaterialTheme.colorScheme
    val brush = Brush.linearGradient(listOf(c.primary, c.tertiary))
    val shapes: List<RoundedPolygon> = remember {
        listOf(MaterialShapes.Cookie9Sided, MaterialShapes.Clover4Leaf, MaterialShapes.Sunny, MaterialShapes.Gem, MaterialShapes.Puffy, MaterialShapes.SoftBurst)
    }
    val morphs = remember { shapes.indices.map { Morph(shapes[it], shapes[(it + 1) % shapes.size]) } }
    val live = !rememberReducedMotion()
    val transition = rememberInfiniteTransition(label = "morph")
    val t = if (live) transition.animateFloat(0f, shapes.size.toFloat(), infiniteRepeatable(tween(2400 * shapes.size, easing = LinearEasing)), label = "t") else null
    val rotation = if (live) transition.animateFloat(0f, 360f, infiniteRepeatable(tween(18_000, easing = LinearEasing)), label = "rot") else null
    val path = remember { Path() }
    val matrix = remember { Matrix() }
    Canvas(modifier) {
        val value = t?.value ?: 0f
        val index = value.toInt() % morphs.size
        val progress = Emphasized.transform(((value - value.toInt()) / 0.6f).coerceAtMost(1f))
        path.rewind()
        morphs[index].toPath(progress, path)
        // Polygons are normalized around (0.5,0.5) in unit space; scale to the canvas.
        matrix.reset()
        matrix.scale(size.width, size.height)
        path.transform(matrix)
        rotate(rotation?.value ?: 0f, pivot = Offset(size.width / 2, size.height / 2)) { drawPath(path, brush) }
    }
}

/** The brand: the morphing shape with the play mark on it. */
@Composable
fun BrandMark(modifier: Modifier = Modifier, size: Dp = 120.dp) {
    Box(modifier.size(size), contentAlignment = Alignment.Center) {
        MorphingShape(Modifier.fillMaxSize())
        Icon(Icons.Rounded.PlayArrow, null, tint = MaterialTheme.colorScheme.onPrimary, modifier = Modifier.size(size * 0.42f))
    }
}

/** An icon on one of the expressive Material shapes (cookie, clover, ...). */
@Composable
fun ShapedIcon(icon: ImageVector, shape: RoundedPolygon, container: Color, content: Color, modifier: Modifier = Modifier, size: Dp = 44.dp) {
    Box(modifier.size(size).clip(shape.toShape()).background(container), contentAlignment = Alignment.Center) {
        Icon(icon, contentDescription = null, tint = content, modifier = Modifier.size(size * 0.5f))
    }
}

/** Springs down a little while pressed. Pass the component's own interaction source. */
@Composable
fun Modifier.pressScale(source: InteractionSource): Modifier {
    val pressed by source.collectIsPressedAsState()
    val scale by animateFloatAsState(if (pressed) 0.94f else 1f, MaterialTheme.motionScheme.fastSpatialSpec(), label = "press")
    return graphicsLayer { scaleX = scale; scaleY = scale }
}

/** Slides up and fades in on first show; [index] staggers siblings by 40ms. */
@Composable
fun Modifier.enterStaggered(index: Int): Modifier {
    if (rememberReducedMotion()) return this
    val progress = remember { Animatable(0f) }
    val spec = MaterialTheme.motionScheme.slowSpatialSpec<Float>()
    LaunchedEffect(Unit) {
        delay(index.coerceAtMost(10) * 40L)
        progress.animateTo(1f, spec)
    }
    return graphicsLayer {
        alpha = progress.value.coerceIn(0f, 1f)
        translationY = (1f - progress.value) * 32.dp.toPx()
    }
}

/** Deterministic warm gradient so items without art still look distinct (same hash as the web). */
private fun placeholderBrush(seed: String): Brush {
    var h = 0
    for (c in seed) h = (h * 31 + c.code) % 360
    val a = Color.hsl(h.toFloat(), 0.35f, 0.26f)
    val b = Color.hsl(((h + 40) % 360).toFloat(), 0.30f, 0.14f)
    return Brush.linearGradient(listOf(a, b))
}

/**
 * Server art first, then [frame] (a video frame from the server's thumbnail cache), then a gradient with the title.
 * A frame carries no title, so a poster built from one gets the title printed over it.
 * [sharp] is a larger copy of [image] for art shown big; when it fails the stored size is used.
 */
@Composable
fun Artwork(image: String?, frame: String?, title: String, modifier: Modifier = Modifier, poster: Boolean = true, sharp: String? = null) {
    val sources = remember(sharp, image, frame) { listOfNotNull(sharp, image, frame) }
    var failed by rememberSaveable(sources) { mutableIntStateOf(0) }
    val src = sources.getOrNull(failed)
    Box(modifier.background(MaterialTheme.colorScheme.surfaceContainerHigh)) {
        if (src != null) {
            AsyncImage(
                model = src,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                onError = { failed++ },
                modifier = Modifier.fillMaxSize(),
            )
            if (src == frame && poster) {
                Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.5f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.85f))))
                Text(title, color = Color.White, style = MaterialTheme.typography.labelLarge, maxLines = 3, overflow = TextOverflow.Ellipsis, modifier = Modifier.align(Alignment.BottomStart).padding(10.dp))
            }
        } else {
            Column(
                Modifier.fillMaxSize().background(placeholderBrush(title)).padding(10.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Icon(Icons.Rounded.Movie, null, tint = Color.White.copy(alpha = 0.8f), modifier = Modifier.size(if (poster) 28.dp else 22.dp))
                Text(title, color = Color.White.copy(alpha = 0.85f), style = MaterialTheme.typography.labelMedium, textAlign = TextAlign.Center, maxLines = 3, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

@Composable
private fun BoxScope.ProgressBar(value: Float) {
    Box(Modifier.align(Alignment.BottomCenter).padding(8.dp).fillMaxWidth().height(4.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.5f))) {
        Box(Modifier.fillMaxHeight().fillMaxWidth(value).clip(CircleShape).background(MaterialTheme.colorScheme.primary))
    }
}

/** "S1 · E5" for episodes, season or episode counts for shows and seasons, else the year. */
@Composable
fun subtitleOf(item: ItemCard): String? = when {
    item.kind == "episode" && item.episodeNo != null -> stringResource(R.string.item_episode_short, item.seasonNo ?: 0, item.episodeNo!!)
    item.kind == "show" && item.childCount > 0 -> androidx.compose.ui.res.pluralStringResource(R.plurals.item_seasons, item.childCount, item.childCount)
    item.kind == "season" -> androidx.compose.ui.res.pluralStringResource(R.plurals.item_episodes, item.childCount, item.childCount)
    else -> item.year?.toString()
}

/**
 * A title on a 2:3 poster, or a 16:9 still when [wide] (episodes, continue watching). Springs on press and lifts its
 * artwork; the watched check and progress bar sit on the art.
 */
@Composable
fun PosterCard(item: ItemCard, onClick: () -> Unit, modifier: Modifier = Modifier, wide: Boolean = false) {
    val api = LocalApi.current
    val source = remember { MutableInteractionSource() }
    val title = if (item.kind == "episode" && wide) item.showTitle ?: item.title else item.title
    val sub = if (item.kind == "episode" && wide) listOfNotNull(subtitleOf(item), item.title).joinToString(" · ") else subtitleOf(item)
    Column(modifier.pressScale(source).clip(MaterialTheme.shapes.medium).clickable(source, null, onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(if (wide) 16f / 9f else 2f / 3f).clip(MaterialTheme.shapes.medium)) {
            val key = if (wide) item.backdrop ?: item.poster else item.poster
            Artwork(
                image = api?.imageUrl(key),
                // Episode stills are stored at 300 px; a wide card on a phone is about 800 px.
                sharp = if (wide) api?.sharperImageUrl(key, 780) else null,
                frame = api?.frameUrl(item.id),
                title = if (item.showTitle != null && item.kind != "movie") item.showTitle!! else item.title,
                poster = !wide,
                modifier = Modifier.fillMaxSize(),
            )
            if (item.watched) {
                Icon(
                    Icons.Rounded.CheckCircle, stringResource(R.string.item_watched), tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.align(Alignment.TopEnd).padding(8.dp).background(Color.Black.copy(alpha = 0.45f), CircleShape),
                )
            }
            item.progress?.let { ProgressBar(it) }
        }
        Spacer(Modifier.height(8.dp))
        Text(title, style = MaterialTheme.typography.titleSmall, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(horizontal = 2.dp))
        if (sub != null) {
            Text(sub, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(horizontal = 2.dp))
        }
        Spacer(Modifier.height(4.dp))
    }
}

@Composable
fun SectionTitle(text: String, modifier: Modifier = Modifier, action: (@Composable () -> Unit)? = null) {
    Row(modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(text, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f).semantics { heading() })
        action?.invoke()
    }
}

/** A titled horizontal row of cards. Empty rows are not shown. */
@Composable
fun ShelfRow(title: String, items: List<ItemCard>, onOpen: (ItemCard) -> Unit, wide: Boolean = false, action: (@Composable () -> Unit)? = null) {
    if (items.isEmpty()) return
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        SectionTitle(title, action = action)
        LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            items(items, key = { it.id }) { item ->
                PosterCard(item, { onOpen(item) }, Modifier.width(if (wide) 260.dp else 128.dp).animateItem(), wide = wide)
            }
        }
    }
}

@Composable
fun LoadingBox(modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) { LoadingIndicator(Modifier.size(64.dp)) }
}

/** A friendly full-area message with an optional action. */
@Composable
fun MessageState(icon: ImageVector, title: String, message: String?, modifier: Modifier = Modifier, action: String? = null, onAction: (() -> Unit)? = null) {
    val c = MaterialTheme.colorScheme
    Column(
        modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        ShapedIcon(icon, MaterialShapes.Cookie9Sided, c.secondaryContainer, c.onSecondaryContainer, Modifier.enterStaggered(0), size = 88.dp)
        Text(title, style = MaterialTheme.typography.titleLarge, textAlign = TextAlign.Center, modifier = Modifier.enterStaggered(1))
        if (message != null) Text(message, style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant, textAlign = TextAlign.Center, modifier = Modifier.enterStaggered(2))
        if (action != null && onAction != null) FilledTonalButton(onAction, Modifier.enterStaggered(3)) { Text(action) }
    }
}

@Composable
fun ErrorState(message: String?, onRetry: () -> Unit, modifier: Modifier = Modifier) =
    MessageState(Icons.Rounded.CloudOff, stringResource(R.string.error_title), message, modifier, stringResource(R.string.action_retry), onRetry)

/** A small rounded tag (play mode, rating, genre). */
@Composable
fun Tag(text: String, modifier: Modifier = Modifier, container: Color = MaterialTheme.colorScheme.secondaryContainer, content: Color = MaterialTheme.colorScheme.onSecondaryContainer) {
    Surface(color = container, contentColor = content, shape = CircleShape, modifier = modifier) {
        Text(text, style = MaterialTheme.typography.labelMedium, modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp))
    }
}
