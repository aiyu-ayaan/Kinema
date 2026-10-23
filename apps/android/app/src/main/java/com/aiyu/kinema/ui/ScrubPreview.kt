package com.aiyu.kinema.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage

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
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (tile != null) {
            val scale = 0.9f
            val displayW = (tile.w * scale).dp
            val displayH = (tile.h * scale).dp
            val sheetW = (tile.w * SHEET_GRID * scale).dp
            val sheetH = (tile.h * SHEET_GRID * scale).dp
            Box(
                Modifier
                    .size(displayW, displayH)
                    .clip(RoundedCornerShape(8.dp))
                    .border(1.dp, Color.White.copy(alpha = 0.35f), RoundedCornerShape(8.dp))
                    .background(Color.Black),
            ) {
                AsyncImage(
                    model = tile.url,
                    contentDescription = null,
                    modifier = Modifier
                        .size(sheetW, sheetH)
                        .offset(x = (-tile.x * scale).dp, y = (-tile.y * scale).dp),
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
