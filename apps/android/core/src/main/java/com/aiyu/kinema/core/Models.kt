package com.aiyu.kinema.core

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

// Mirrors of the server's JSON (backend routes, web `lib/types.ts`). Unknown fields are ignored.

@Serializable
data class ServerInfo(
    val name: String,
    val version: String,
    @SerialName("public_url") val publicUrl: String = "",
    /** SHA-256 of the server's own certificate; lets an app that came in by the public address trust its home one. */
    @SerialName("tls_fingerprint") val tlsFingerprint: String = "",
)

@Serializable
data class AuthStatus(@SerialName("needs_setup") val needsSetup: Boolean)

@Serializable
data class User(
    val id: Long,
    val name: String,
    @SerialName("is_admin") val isAdmin: Boolean = false,
    val avatar: String? = null,
    @SerialName("has_password") val hasPassword: Boolean = false,
    @SerialName("has_pin") val hasPin: Boolean = false,
)

@Serializable
data class SessionResponse(val token: String, val user: User)

@Serializable
data class Library(
    val id: Long,
    val name: String,
    val kind: String,
    @SerialName("item_count") val itemCount: Int = 0,
    @SerialName("show_count") val showCount: Int = 0,
)

@Serializable
data class ItemCard(
    val id: Long,
    @SerialName("library_id") val libraryId: Long = 0,
    @SerialName("parent_id") val parentId: Long? = null,
    /** Season or episode: the show it belongs to. */
    @SerialName("show_id") val showId: Long? = null,
    val kind: String,
    val title: String,
    val year: Int? = null,
    @SerialName("season_no") val seasonNo: Int? = null,
    @SerialName("episode_no") val episodeNo: Int? = null,
    val summary: String? = null,
    val rating: Double? = null,
    val poster: String? = null,
    val backdrop: String? = null,
    @SerialName("show_title") val showTitle: String? = null,
    @SerialName("position_s") val positionS: Double? = null,
    val watched: Boolean = false,
    @SerialName("duration_s") val durationS: Double? = null,
    @SerialName("child_count") val childCount: Int = 0,
    @SerialName("added_at") val addedAt: Long = 0,
) {
    /** Movies, episodes and loose videos play; shows and seasons open their page. */
    val playable get() = kind == "movie" || kind == "episode" || kind == "video"

    /** 0..1 for a progress bar, or null when there is nothing to show. */
    val progress: Float?
        get() {
            val p = positionS ?: return null
            val d = durationS ?: return null
            if (watched || p <= 0 || d <= 0) return null
            return (p / d).toFloat().coerceIn(0f, 1f)
        }
}

@Serializable
data class Stream(
    val id: Long,
    @SerialName("file_id") val fileId: Long = 0,
    val idx: Int,
    val type: String,
    val codec: String? = null,
    val language: String? = null,
    val title: String? = null,
    val channels: Int? = null,
    val width: Int? = null,
    val height: Int? = null,
    @SerialName("is_default") val isDefault: Boolean = false,
) {
    /** Text subtitles are drawn by the app; image ones (PGS, VobSub) cannot be. */
    val isTextSubtitle get() = codec in TEXT_SUBS

    private companion object {
        val TEXT_SUBS = setOf("subrip", "srt", "ass", "ssa", "webvtt", "vtt", "mov_text", "text")
    }
}

@Serializable
data class MediaFile(
    val id: Long,
    val size: Long = 0,
    val container: String? = null,
    @SerialName("duration_s") val durationS: Double? = null,
    val streams: List<Stream> = emptyList(),
    /** Intro and end credits, found by the server or set by an admin. */
    val markers: List<Marker> = emptyList(),
) {
    val intro get() = markers.firstOrNull { it.kind == "intro" }
    val credits get() = markers.firstOrNull { it.kind == "credits" }
}

@Serializable
data class Marker(
    val kind: String,
    @SerialName("start_s") val startS: Double,
    @SerialName("end_s") val endS: Double,
    val source: String = "",
)

@Serializable
data class CastMember(val id: Long, val name: String, val role: String = "", val image: String? = null)

@Serializable
data class ItemDetail(
    val item: ItemCard,
    val genres: List<String> = emptyList(),
    val files: List<MediaFile> = emptyList(),
    val children: List<ItemCard> = emptyList(),
    val cast: List<CastMember> = emptyList(),
    @SerialName("prev_id") val prevId: Long? = null,
    @SerialName("next_id") val nextId: Long? = null,
    @SerialName("next_up") val nextUp: ItemCard? = null,
    /** Show only, for admins: the audio and subtitle pick every episode starts with. */
    val tracks: SeriesTracks? = null,
)

/** Positions among the first episode's audio / subtitle [streams]; null is the server's language settings, subtitle -1 is off. */
@Serializable
data class SeriesTracks(val audio: Int? = null, val subtitle: Int? = null, val streams: List<Stream> = emptyList())

@Serializable
data class Shelf(val library: Library, val kind: String, val items: List<ItemCard>)

@Serializable
data class Home(
    @SerialName("continue_watching") val continueWatching: List<ItemCard> = emptyList(),
    val shelves: List<Shelf> = emptyList(),
)

@Serializable
data class PlaybackPlan(
    val mode: String,
    @SerialName("duration_s") val durationS: Double? = null,
    val audio: Int? = null,
    /** Where a remux or transcode stream really starts: the player's time offset. */
    val start: Double = 0.0,
    /** Subtitle stream id the server's language settings pick; applied on first load. */
    val subtitle: Long? = null,
)

/** What this phone decodes beyond H.264/AAC; the server picks direct play, remux or transcode from it. */
data class PlayCaps(val hevc: Boolean, val av1: Boolean)
