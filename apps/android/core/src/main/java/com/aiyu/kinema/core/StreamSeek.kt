package com.aiyu.kinema.core

/**
 * Seeking inside a remux or transcode stream. The stream starts at `offsetS` of the title, so the title time
 * `targetS` sits at `targetS - offsetS` in the player. When that position is already buffered the player can jump
 * there and the server keeps its ffmpeg running; otherwise the caller restarts the stream at the target.
 */
object StreamSeek {
    /**
     * The position to seek to in the player (ms), or null when the stream has to restart.
     *
     * [seekable] is what the player reports for the loaded stream: ExoPlayer maps every seek in a stream it cannot
     * seek to t=0, so an unseekable one must always restart. [backBufferMs] is how much already-played media the
     * player keeps behind [positionMs]; [marginMs] stays clear of the buffer's end so a seek never lands on data
     * that has not arrived yet.
     */
    fun bufferedTargetMs(
        targetS: Double,
        offsetS: Double,
        positionMs: Long,
        bufferedMs: Long,
        seekable: Boolean,
        backBufferMs: Long = 0,
        marginMs: Long = 1_000,
    ): Long? {
        if (!seekable) return null
        val local = ((targetS - offsetS) * 1000).toLong()
        val from = (positionMs - backBufferMs).coerceAtLeast(0)
        val to = bufferedMs - marginMs
        return if (local in from..to) local else null
    }
}
