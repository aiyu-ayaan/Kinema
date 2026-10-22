package com.aiyu.kinema.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class StreamSeekTest {
    // A stream that began at 600 s, played 20 s and holds 60 s: the player sits at 20 s, buffered to 80 s.
    private fun seek(target: Double, seekable: Boolean = true, back: Long = 0) =
        StreamSeek.bufferedTargetMs(target, 600.0, 20_000, 80_000, seekable, back)

    @Test
    fun insideTheBufferJumpsLocally() {
        assertEquals(50_000L, seek(650.0))
        assertEquals(20_000L, seek(620.0))
    }

    @Test
    fun beyondTheBufferOrBeforeTheStreamRestarts() {
        assertNull(seek(700.0))
        assertNull(seek(679.5))
        assertNull(seek(590.0))
    }

    @Test
    fun behindThePlayheadNeedsAKeptBackBuffer() {
        assertNull(seek(610.0))
        assertEquals(10_000L, seek(610.0, back = 15_000))
        assertNull(seek(600.0, back = 15_000))
    }

    @Test
    fun anUnseekableStreamAlwaysRestarts() {
        assertNull(seek(650.0, seekable = false))
    }
}
