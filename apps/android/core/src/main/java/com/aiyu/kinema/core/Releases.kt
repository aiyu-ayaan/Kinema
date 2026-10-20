package com.aiyu.kinema.core

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull

/** Release streams. A channel takes its own builds and every steadier one; [stage] orders them: alpha < beta < stable. */
enum class UpdateChannel(val stage: Long) { STABLE(2), BETA(1), ALPHA(0) }

/** The app's own releases on GitHub, the same rules as the desktop app (apps/desktop/src/updates.js). */
object Releases {
    const val API = "https://api.github.com/repos/aiyu-ayaan/Kinema/releases?per_page=30"

    // The release workflow names versions 0.0.2-alpha.1, 0.0.2-beta.1 and 0.0.2 (scripts/release-version.mjs).
    private val VERSION = Regex("""^v?(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta)\.(\d+))?$""")
    private val STAGE = mapOf("alpha" to 0L, "beta" to 1L, "stable" to 2L)

    /** `1.2.3-beta.4` as [major, minor, patch, stage, number]; null for any other shape (a `-dev` build never updates). */
    fun versionKey(text: String): List<Long>? {
        val m = VERSION.matchEntire(text.trim()) ?: return null
        val (major, minor, patch, label, number) = m.destructured
        // A stable release has no pre-release number and still sorts above every one of its pre-releases.
        return listOf(major.toLong(), minor.toLong(), patch.toLong(), STAGE.getValue(label.ifEmpty { "stable" }), number.toLongOrNull() ?: Long.MAX_VALUE)
    }

    private fun newer(a: List<Long>, b: List<Long>): Boolean {
        val i = a.indices.firstOrNull { a[it] != b[it] } ?: return false
        return a[i] > b[i]
    }

    /** [notes]: the release's description in Markdown, empty when it has none. */
    data class Apk(val version: String, val url: String, val page: String, val size: Long, val notes: String = "")

    /** The channel a build belongs to, the default: whoever installed an alpha wants alphas. */
    fun channelOf(version: String): UpdateChannel =
        UpdateChannel.entries.firstOrNull { it.stage == versionKey(version)?.get(3) } ?: UpdateChannel.STABLE

    private fun parse(body: String): List<JsonObject> =
        (runCatching { Json.parseToJsonElement(body) as? JsonArray }.getOrNull() ?: JsonArray(emptyList())).mapNotNull { it as? JsonObject }

    /** The Markdown notes of the release of [version]; a dev build shows those of the release it was built from. */
    fun notesOf(body: String, version: String): String? {
        val tag = "v" + version.trim().removeSuffix("-dev")
        return parse(body).firstOrNull { it["tag_name"]?.jsonPrimitive?.contentOrNull == tag }
            ?.get("body")?.jsonPrimitive?.contentOrNull?.takeIf { it.isNotBlank() }
    }

    /**
     * The newest release with an APK that is newer than [installed], on [channel] (by default the installed one's):
     * alpha takes alphas, betas and stable; beta takes betas and stable; stable only stable.
     */
    fun pickApk(body: String, installed: String, channel: UpdateChannel = channelOf(installed)): Apk? {
        val current = versionKey(installed) ?: return null
        var best: Pair<List<Long>, Apk>? = null
        for (release in parse(body)) {
            fun field(name: String) = release[name]?.jsonPrimitive?.contentOrNull.orEmpty()
            val tag = field("tag_name")
            val key = versionKey(tag) ?: continue
            if (release["draft"]?.jsonPrimitive?.booleanOrNull == true || key[3] < channel.stage || !newer(key, best?.first ?: current)) continue
            val asset = release["assets"]?.jsonArray?.map { it.jsonObject }
                ?.firstOrNull { it["name"]?.jsonPrimitive?.contentOrNull.orEmpty().endsWith(".apk") } ?: continue
            val url = asset["browser_download_url"]?.jsonPrimitive?.contentOrNull ?: continue
            best = key to Apk(tag.removePrefix("v"), url, field("html_url"), asset["size"]?.jsonPrimitive?.longOrNull ?: 0, field("body"))
        }
        return best?.second
    }
}
