package com.aiyu.kinema.core

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json

// Device approval, title requests and own watch stats (backend routes auth, requests, activity).

/** What this phone says about itself at sign-in; [id] is random, made once and kept. */
@Serializable
data class DeviceInfo(val id: String, val kind: String = "mobile", val name: String)

/** "Google Pixel 8" rather than "Google Google Pixel 8": many models already start with the maker's name. */
fun deviceLabel(manufacturer: String, model: String): String {
    val make = manufacturer.trim()
    val name = model.trim()
    return when {
        name.isEmpty() -> make
        make.isEmpty() || name.startsWith(make, ignoreCase = true) -> name
        else -> "$make $name"
    }
}

/** A sign-in either opens a session or waits for the administrator to approve this device. */
sealed interface LoginResult {
    data class SignedIn(val session: SessionResponse) : LoginResult
    data class ApprovalRequired(val pollToken: String, val deviceName: String) : LoginResult
}

sealed interface PollResult {
    data object Pending : PollResult
    data class SignedIn(val session: SessionResponse) : PollResult
}

@Serializable
private class ApprovalBody(@SerialName("poll_token") val pollToken: String, @SerialName("device_name") val deviceName: String = "")

@Serializable
private class LoginBody(
    val token: String? = null,
    val user: User? = null,
    @SerialName("approval_required") val approval: ApprovalBody? = null,
    val pending: Boolean = false,
)

private val accountJson = Json { ignoreUnknownKeys = true; explicitNulls = false; coerceInputValues = true }

private fun LoginBody.session() = if (token != null && user != null) SessionResponse(token, user) else null

/** `{token,user}` or `{approval_required:{poll_token,device_name}}`; anything else is a [SerializationException]. */
fun parseLogin(text: String): LoginResult {
    val body = accountJson.decodeFromString(LoginBody.serializer(), text)
    body.approval?.let { return LoginResult.ApprovalRequired(it.pollToken, it.deviceName) }
    return LoginResult.SignedIn(body.session() ?: throw SerializationException("no session in login response"))
}

/** `{pending:true}` while the administrator has not decided, `{token,user}` once approved. */
fun parsePoll(text: String): PollResult {
    val body = accountJson.decodeFromString(LoginBody.serializer(), text)
    return body.session()?.let(PollResult::SignedIn) ?: PollResult.Pending
}

@Serializable
data class MediaRequest(
    val id: Long,
    @SerialName("user_id") val userId: Long = 0,
    val kind: String,
    val title: String,
    val year: Int? = null,
    /** Image key such as "w500/abc.jpg". */
    val poster: String? = null,
    val status: String = "pending",
    val note: String? = null,
    @SerialName("admin_note") val adminNote: String? = null,
    @SerialName("created_at") val createdAt: Long = 0,
) {
    val pending get() = status == "pending"
}

@Serializable
data class RequestCandidate(
    @SerialName("tmdb_id") val tmdbId: Long,
    val title: String,
    val year: Int? = null,
    val overview: String? = null,
    /** Raw TMDB path such as "/abc.jpg"; see [tmdbPosterKey]. */
    val poster: String? = null,
)

@Serializable
data class RequestSearch(val tmdb: Boolean = false, val results: List<RequestCandidate> = emptyList())

/** The server image key for a raw TMDB poster path: "/abc.jpg" becomes "w500/abc.jpg". */
fun tmdbPosterKey(path: String?): String? {
    val name = path?.trim()?.trimStart('/')?.takeIf { it.isNotEmpty() } ?: return null
    return if (Regex("""w\d+/.+""").matches(name)) name else "w500/$name"
}

@Serializable
data class WatchTotals(
    @SerialName("total_s") val totalS: Long = 0,
    @SerialName("last_7d_s") val last7dS: Long = 0,
    @SerialName("last_30d_s") val last30dS: Long = 0,
    val titles: Long = 0,
    val completed: Long = 0,
    @SerialName("last_played_at") val lastPlayedAt: Long? = null,
)

@Serializable
data class DayTime(val day: Long, val seconds: Long)

@Serializable
data class TopTitle(
    @SerialName("item_id") val itemId: Long,
    val title: String,
    val kind: String = "movie",
    val poster: String? = null,
    val seconds: Long = 0,
)

@Serializable
data class GenreTime(val genre: String, val seconds: Long = 0)

@Serializable
data class HistoryEntry(val item: ItemCard, @SerialName("last_played_at") val lastPlayedAt: Long = 0)

@Serializable
data class WatchStats(
    val totals: WatchTotals = WatchTotals(),
    val daily: List<DayTime> = emptyList(),
    @SerialName("top_titles") val topTitles: List<TopTitle> = emptyList(),
    @SerialName("top_genres") val topGenres: List<GenreTime> = emptyList(),
    val history: List<HistoryEntry> = emptyList(),
)

/** Seconds per day for the [days] days ending [today] (unix seconds / 86400), oldest first; days the server left out are 0. */
fun fillDaily(daily: List<DayTime>, today: Long, days: Int = 30): List<Long> {
    val bySecond = daily.associate { it.day to it.seconds }
    return List(days) { bySecond[today - (days - 1 - it)] ?: 0L }
}

/** Whole hours and the minutes left over, for "2h 5m"; the app picks the words. */
data class HoursMinutes(val hours: Long, val minutes: Long)

fun hoursMinutes(seconds: Long): HoursMinutes {
    val total = (seconds.coerceAtLeast(0) + 30) / 60
    return HoursMinutes(total / 60, total % 60)
}
