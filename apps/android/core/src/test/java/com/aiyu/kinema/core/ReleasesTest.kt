package com.aiyu.kinema.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ReleasesTest {
    private fun release(tag: String, vararg assets: String, draft: Boolean = false) =
        """{"tag_name":"v$tag","html_url":"p/$tag","body":"n$tag","draft":$draft,"assets":[${assets.joinToString(",") { """{"name":"$it","browser_download_url":"u/$it","size":5}""" }}]}"""

    private val body = listOf(
        release("0.0.2-alpha.1", "Kinema-0.0.2-alpha.1.apk"),
        release("0.0.1", "Kinema-0.0.1.apk"),
        release("0.0.1-beta.2", "Kinema-0.0.1-beta.2.apk"),
        release("0.0.3", "kinema-server-0.0.3-linux-x86_64.tar.gz"),
        release("0.0.4", "Kinema-0.0.4.apk", draft = true),
    ).joinToString(",", "[", "]")

    @Test
    fun picksNewestApkOnChannel() {
        assertEquals(Releases.Apk("0.0.2-alpha.1", "u/Kinema-0.0.2-alpha.1.apk", "p/0.0.2-alpha.1", 5, "n0.0.2-alpha.1"), Releases.pickApk(body, "0.0.1-alpha.1"))
        // Beta takes stable, never an alpha; 0.0.3 has no APK and 0.0.4 is a draft.
        assertEquals("0.0.1", Releases.pickApk(body, "0.0.1-beta.2")?.version)
        assertNull(Releases.pickApk(body, "0.0.1"))
        assertNull(Releases.pickApk(body, "0.0.1-alpha.1-dev"))
        assertNull(Releases.pickApk("not json", "0.0.1"))
        // A chosen channel overrides the installed one's.
        assertEquals("0.0.1", Releases.pickApk(body, "0.0.1-alpha.1", UpdateChannel.STABLE)?.version)
        assertEquals("n0.0.1", Releases.notesOf(body, "0.0.1-dev"))
        assertEquals(UpdateChannel.BETA, Releases.channelOf("0.0.1-beta.2"))
    }
}
