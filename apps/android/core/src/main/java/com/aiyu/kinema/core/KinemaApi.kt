package com.aiyu.kinema.core

import java.io.IOException
import java.net.URLEncoder
import java.util.concurrent.TimeUnit
import javax.net.ssl.SSLException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.KSerializer
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import kotlinx.serialization.json.add
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

/** [status] is the HTTP status, [OFFLINE] when the server can't be reached, [UNTRUSTED] when its certificate isn't accepted. */
class ApiException(message: String, val status: Int, val code: String? = null) : Exception(message) {
    companion object {
        const val OFFLINE = 0
        const val UNTRUSTED = -1
    }
}

/** REST client of one Kinema server. [pin] is the approved certificate fingerprint of a self-signed server. */
class KinemaApi(val baseUrl: String, pin: String?, connectSeconds: Long = 6, private val token: () -> String?) {
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false; coerceInputValues = true }

    /** Also used by the image loader and the player, so artwork and video carry the same pin and session. */
    val client: OkHttpClient = run {
        val trust = Tls.PinnedTrustManager(pin)
        OkHttpClient.Builder()
            .sslSocketFactory(Tls.sslContext(trust).socketFactory, trust)
            .hostnameVerifier(Tls.hostnameVerifier(pin))
            .connectTimeout(connectSeconds, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .addInterceptor { chain ->
                val request = chain.request()
                // Only this server gets the session: a public URL behind a proxy is the same server, other hosts are not.
                val authed = token()?.takeIf { request.url.toString().startsWith(baseUrl) }
                    ?.let { request.newBuilder().header("Authorization", "Bearer $it").build() }
                chain.proceed(authed ?: request)
            }
            .build()
    }

    fun url(path: String) = "$baseUrl/api$path"

    /** The response body as text; a failed status becomes an [ApiException] with the server's message. */
    private suspend fun send(method: String, path: String, body: JsonObject? = null, readSeconds: Long? = null): String = withContext(Dispatchers.IO) {
        val payload = (body?.toString() ?: if (method == "GET") null else "{}")?.toRequestBody(JSON)
        val http = readSeconds?.let { client.newBuilder().readTimeout(it, TimeUnit.SECONDS).build() } ?: client
        try {
            http.newCall(Request.Builder().url(url(path)).method(method, payload).build()).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) {
                    val error = runCatching { json.parseToJsonElement(text).jsonObject["error"]?.jsonObject }.getOrNull()
                    throw ApiException(
                        error?.get("message")?.jsonPrimitive?.contentOrNull ?: res.message.ifBlank { "HTTP ${res.code}" },
                        res.code,
                        error?.get("code")?.jsonPrimitive?.contentOrNull,
                    )
                }
                text
            }
        } catch (e: SSLException) {
            throw ApiException(e.message ?: "TLS", ApiException.UNTRUSTED)
        } catch (e: IOException) {
            throw ApiException(e.message ?: "offline", ApiException.OFFLINE)
        }
    }

    private fun <T> parsed(text: String, parse: (String) -> T): T = try {
        parse(text)
    } catch (e: kotlinx.serialization.SerializationException) {
        throw ApiException(e.message ?: "bad response", 500)
    }

    private suspend fun <T> call(method: String, path: String, out: KSerializer<T>, body: JsonObject? = null): T {
        val text = send(method, path, body)
        return try {
            json.decodeFromString(out, text)
        } catch (e: kotlinx.serialization.SerializationException) {
            throw ApiException(e.message ?: "bad response", 500)
        }
    }

    suspend fun serverInfo() = call("GET", "/server/info", ServerInfo.serializer())
    suspend fun authStatus() = call("GET", "/auth/status", AuthStatus.serializer())
    suspend fun profiles() = call("GET", "/auth/profiles", ListSerializer(User.serializer()))
    suspend fun me() = call("GET", "/auth/me", User.serializer())
    suspend fun logout() = send("POST", "/auth/logout").let { }

    suspend fun login(userId: Long, password: String?, pin: String?, device: DeviceInfo): LoginResult = parsed(
        send("POST", "/auth/login", buildJsonObject {
            put("user_id", userId)
            password?.let { put("password", it) }
            pin?.let { put("pin", it) }
            putJsonObject("device") { put("id", device.id); put("kind", device.kind); put("name", device.name) }
        }),
        ::parseLogin,
    )

    /** 404 (`auth.device_denied`) when the administrator refused the device or the request is gone. */
    suspend fun devicePoll(pollToken: String): PollResult =
        parsed(send("POST", "/auth/device/poll", buildJsonObject { put("poll_token", pollToken) }), ::parsePoll)

    suspend fun requests() = call("GET", "/requests", ListSerializer(MediaRequest.serializer()))
    suspend fun requestSearch(q: String, kind: String) = call("GET", "/requests/search?q=${enc(q)}&kind=${enc(kind)}", RequestSearch.serializer())
    suspend fun createRequest(kind: String, title: String, year: Int?, tmdbId: Long?, poster: String?, overview: String?, note: String?) =
        call("POST", "/requests", MediaRequest.serializer(), buildJsonObject {
            put("kind", kind)
            put("title", title)
            put("year", year)
            put("tmdb_id", tmdbId)
            put("poster", poster)
            put("overview", overview)
            put("note", note)
        })
    suspend fun withdrawRequest(id: Long) = send("DELETE", "/requests/$id").let { }

    suspend fun myStats() = call("GET", "/profile/stats", WatchStats.serializer())

    suspend fun home() = call("GET", "/home", Home.serializer())
    suspend fun libraries() = call("GET", "/libraries", ListSerializer(Library.serializer()))
    /** Every show of a library, unpaged: a short list for the switcher and its chips. */
    suspend fun libraryShows(id: Long) = call("GET", "/libraries/$id/items?type=show", ListSerializer(ItemCard.serializer()))

    /** [type] is "movie" (everything but shows), "show" or null for both; [sort] is title, added, year or rating. */
    suspend fun libraryPage(id: Long, type: String?, sort: String, offset: Long, limit: Int) = call(
        "GET",
        "/libraries/$id/items?sort=$sort&offset=$offset&limit=$limit" + (type?.let { "&type=$it" } ?: ""),
        LibraryPage.serializer(),
    )
    suspend fun search(q: String) = call("GET", "/search?q=${enc(q)}", ListSerializer(ItemCard.serializer()))
    suspend fun item(id: Long) = call("GET", "/items/$id", ItemDetail.serializer())

    suspend fun playlists() = call("GET", "/playlists", ListSerializer(PlaylistSummary.serializer()))
    suspend fun playlist(id: Long) = call("GET", "/playlists/$id", PlaylistDetail.serializer())
    suspend fun createPlaylist(name: String, itemIds: List<Long> = emptyList()) =
        call("POST", "/playlists", PlaylistDetail.serializer(), buildJsonObject {
            put("name", name)
            putJsonArray("item_ids") { itemIds.forEach { add(it) } }
        })
    suspend fun addToPlaylist(id: Long, itemIds: List<Long>) =
        call("POST", "/playlists/$id/items", PlaylistDetail.serializer(), buildJsonObject {
            putJsonArray("item_ids") { itemIds.forEach { add(it) } }
        })
    suspend fun setPlaylistItems(id: Long, itemIds: List<Long>) =
        call("PUT", "/playlists/$id/items", PlaylistDetail.serializer(), buildJsonObject {
            putJsonArray("item_ids") { itemIds.forEach { add(it) } }
        })
    suspend fun deletePlaylist(id: Long) = send("DELETE", "/playlists/$id")

    suspend fun collections() = call("GET", "/collections", ListSerializer(CollectionSummary.serializer()))
    suspend fun collection(id: Long) = call("GET", "/collections/$id", CollectionDetail.serializer())

    suspend fun timelineVtt(id: Long): String? = withContext(Dispatchers.IO) {
        runCatching {
            val req = Request.Builder().url(url("/items/$id/timeline.vtt")).get().build()
            client.newCall(req).execute().use { resp ->
                // 202 means the server is still making the sheets (""); anything else, such as 404 when the admin
                // turned scrub thumbnails off, means there will be none (null).
                when (resp.code) {
                    200 -> resp.body?.string()
                    202 -> ""
                    else -> null
                }
            }
        }.getOrNull()
    }

    fun spriteUrl(itemId: Long, relativePath: String): String {
        val base = if (relativePath.startsWith("/")) url(relativePath) else url("/items/$itemId/$relativePath")
        val t = token()
        return if (!t.isNullOrEmpty()) "$base?token=${enc(t)}" else base
    }

    suspend fun progress(id: Long, positionS: Double, watched: Boolean) {
        send("PUT", "/items/$id/progress", buildJsonObject { put("position_s", positionS); put("watched", watched) })
    }

    suspend fun setWatched(id: Long, watched: Boolean) {
        send("PUT", "/items/$id/watched", buildJsonObject { put("watched", watched) })
    }

    /**
     * Remembers this profile's audio and subtitle pick for the title (a show for all its episodes), on every device.
     * Positions among the file's audio / subtitle streams; subtitle -1 is off.
     */
    suspend fun setTrackChoice(itemId: Long, audio: Int?, subtitle: Int?) {
        send("PUT", "/items/$itemId/track-choice", buildJsonObject { put("audio", audio); put("subtitle", subtitle) })
    }

    /** Admin: the audio and subtitle pick for every episode of a show, as positions; null is the server default. */
    suspend fun setSeriesTracks(showId: Long, audio: Int?, subtitle: Int?) {
        send("PUT", "/items/$showId/tracks", buildJsonObject { put("audio", audio); put("subtitle", subtitle) })
    }

    suspend fun plan(fileId: Long, audio: Int?, caps: PlayCaps, start: Double) =
        call("GET", "/files/$fileId/plan?${playQuery(audio, caps, start)}", PlaybackPlan.serializer())

    fun streamUrl(fileId: Long, audio: Int?, caps: PlayCaps, start: Double) = url("/stream/$fileId?${playQuery(audio, caps, start)}")

    /** WebVTT text of a text subtitle stream. The first request extracts it from the whole file, which can take minutes. */
    suspend fun subtitle(streamId: Long) = send("GET", "/subtitles/$streamId", readSeconds = 300)

    /** The raw bytes behind an absolute URL of this server, e.g. artwork for the media notification. */
    suspend fun bytes(absoluteUrl: String): ByteArray = withContext(Dispatchers.IO) {
        try {
            client.newCall(Request.Builder().url(absoluteUrl).build()).execute().use { res ->
                if (!res.isSuccessful) throw ApiException(res.message.ifBlank { "HTTP ${res.code}" }, res.code)
                res.body?.bytes() ?: ByteArray(0)
            }
        } catch (e: IOException) {
            throw ApiException(e.message ?: "offline", ApiException.OFFLINE)
        }
    }

    /** `images/w500/abc.jpg` keys from the server, as an absolute URL. */
    fun imageUrl(key: String?) = key?.let { url("/images/$it") }

    /**
     * The same TMDB art at a larger [width] (one of the server's sizes: 780, 1280), for art shown bigger than the
     * stored key's size. Null when the key is already that big or is not TMDB art (TVmaze keys are full size).
     */
    fun sharperImageUrl(key: String?, width: Int): String? {
        val match = key?.let(TMDB_KEY::matchEntire) ?: return null
        val (size, name) = match.destructured
        return if (size.toInt() < width) url("/images/w$width/$name") else null
    }

    /** A video frame for an item without artwork; 404 until the server made it. */
    fun frameUrl(itemId: Long) = url("/items/$itemId/thumb")

    private companion object {
        val JSON = "application/json".toMediaType()
        val TMDB_KEY = Regex("""w(\d+)/([^/]+)""")

        fun enc(s: String): String = URLEncoder.encode(s, "UTF-8")

        fun playQuery(audio: Int?, caps: PlayCaps, start: Double) = buildList {
            audio?.let { add("audio=$it") }
            if (caps.hevc) add("hevc=true")
            if (caps.av1) add("av1=true")
            if (start > 0) add("start=${"%.6f".format(java.util.Locale.ROOT, start)}")
        }.joinToString("&")
    }
}
