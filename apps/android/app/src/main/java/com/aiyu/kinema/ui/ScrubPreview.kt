package com.aiyu.kinema.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.toBitmap

data class ScrubTile(
    val start: Double,
    val end: Double,
    val url: String,
    val x: Int,
    val y: Int,
    val w: Int,
    val h: Int,
)

const val SHEET_GRID = 10

fun parseTimelineVtt(vtt: String, resolveUrl: (String) -> String): List<ScrubTile> {
    val tiles = ArrayList<ScrubTile>()
    val lines = vtt.lines()
    var i = 0
    val cueRegex = Regex("""^(.+?)#xywh=(\d+),(\d+),(\d+),(\d+)$""")

    while (i < lines.size) {
        val line = lines[i].trim()
        if (line.contains("-->")) {
            val times = line.split("-->")
            if (times.size == 2) {
                val start = parseVttTime(times[0].trim())
                val end = parseVttTime(times[1].trim())
                if (i + 1 < lines.size) {
                    val cueLine = lines[i + 1].trim()
                    val match = cueRegex.find(cueLine)
                    if (match != null) {
                        val (sheet, x, y, w, h) = match.destructured
                        tiles.add(
                            ScrubTile(
                                start = start,
                                end = end,
                                url = resolveUrl(sheet),
                                x = x.toIntOrNull() ?: 0,
                                y = y.toIntOrNull() ?: 0,
                                w = w.toIntOrNull() ?: 160,
                                h = h.toIntOrNull() ?: 90,
                            ),
                        )
                        i++
                    }
                }
            }
        }
        i++
    }
    return tiles
}

private fun parseVttTime(ts: String): Double {
    val clean = ts.split(" ")[0].trim()
    val parts = clean.split(":")
    return when (parts.size) {
        3 -> (parts[0].toDoubleOrNull() ?: 0.0) * 3600 +
            (parts[1].toDoubleOrNull() ?: 0.0) * 60 +
            (parts[2].toDoubleOrNull() ?: 0.0)
        2 -> (parts[0].toDoubleOrNull() ?: 0.0) * 60 +
            (parts[1].toDoubleOrNull() ?: 0.0)
        else -> 0.0
    }
}

fun tileAt(tiles: List<ScrubTile>, t: Double): ScrubTile? {
    if (tiles.isEmpty()) return null
    var lo = 0
    var hi = tiles.size - 1
    while (lo < hi) {
        val mid = (lo + hi + 1) shr 1
        if (tiles[mid].start <= t) lo = mid
        else hi = mid - 1
    }
    return tiles[lo]
}

@Composable
fun ScrubThumbnail(
    tiles: List<ScrubTile>,
    time: Double,
    modifier: Modifier = Modifier,
) {
    val tile = remember(tiles, time) { tileAt(tiles, time) }
    val context = LocalContext.current
    var bitmap by remember { mutableStateOf<ImageBitmap?>(null) }
    var loadedUrl by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(tile?.url) {
        val url = tile?.url
        if (url == null) {
            bitmap = null
            loadedUrl = null
            return@LaunchedEffect
        }
        if (url == loadedUrl && bitmap != null) return@LaunchedEffect

        val loader = SingletonImageLoader.get(context)
        val request = ImageRequest.Builder(context)
            .data(url)
            .size(coil3.size.Size.ORIGINAL)
            .build()
        val result = loader.execute(request)
        if (result is SuccessResult) {
            bitmap = result.image.toBitmap().asImageBitmap()
            loadedUrl = url
        } else {
            bitmap = null
        }
    }

    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (tile != null && bitmap != null) {
            val scale = 0.9f
            val displayW = (tile.w * scale).dp
            val displayH = (tile.h * scale).dp
            Canvas(
                modifier = Modifier
                    .size(displayW, displayH)
                    .clip(RoundedCornerShape(8.dp))
                    .border(1.dp, Color.White.copy(alpha = 0.35f), RoundedCornerShape(8.dp)),
            ) {
                drawImage(
                    image = bitmap!!,
                    srcOffset = IntOffset(tile.x, tile.y),
                    srcSize = IntSize(tile.w, tile.h),
                    dstOffset = IntOffset.Zero,
                    dstSize = IntSize(size.width.toInt(), size.height.toInt()),
                )
            }
        }
        Box(
            Modifier
                .padding(top = 4.dp)
                .background(Color.Black.copy(alpha = 0.85f), CircleShape)
                .padding(horizontal = 8.dp, vertical = 2.dp),
        ) {
            Text(
                text = clock(time),
                style = MaterialTheme.typography.labelSmall,
                color = Color.White,
            )
        }
    }
}
