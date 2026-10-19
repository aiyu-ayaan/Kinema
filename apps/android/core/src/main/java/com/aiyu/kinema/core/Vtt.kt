package com.aiyu.kinema.core

/** One subtitle line, times in seconds of the whole video. */
data class Cue(val start: Double, val end: Double, val text: String)

/** WebVTT from the server (`/subtitles/{id}`): cue timings and text, styling tags stripped. */
fun parseVtt(text: String): List<Cue> {
    val cues = ArrayList<Cue>()
    val blocks = text.replace("\r\n", "\n").split(Regex("\n{2,}"))
    for (block in blocks) {
        val lines = block.lines()
        val timing = lines.indexOfFirst { "-->" in it }
        if (timing < 0) continue
        val (a, b) = lines[timing].split("-->").map { it.trim().substringBefore(' ') }
        val start = vttTime(a) ?: continue
        val end = vttTime(b) ?: continue
        val body = lines.drop(timing + 1).joinToString("\n").replace(Regex("<[^>]+>"), "").trim()
        if (body.isNotEmpty()) cues += Cue(start, end, body)
    }
    return cues.sortedBy { it.start }
}

/** `01:02:03.500` or `02:03.500` to seconds. */
private fun vttTime(s: String): Double? {
    val parts = s.split(':')
    if (parts.size !in 2..3) return null
    val sec = parts.last().replace(',', '.').toDoubleOrNull() ?: return null
    val min = parts[parts.size - 2].toIntOrNull() ?: return null
    val hours = if (parts.size == 3) parts[0].toIntOrNull() ?: return null else 0
    return hours * 3600 + min * 60 + sec
}
