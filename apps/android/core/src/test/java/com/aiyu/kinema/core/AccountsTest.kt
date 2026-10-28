package com.aiyu.kinema.core

import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AccountsTest {
    private val user = """{"id":3,"name":"Mia","is_admin":false,"avatar":null,"has_password":true,"has_pin":false}"""

    @Test
    fun loginSession() {
        val r = parseLogin("""{"token":"abc","user":$user}""") as LoginResult.SignedIn
        assertEquals("abc", r.session.token)
        assertEquals("Mia", r.session.user.name)
    }

    @Test
    fun loginNeedsApproval() {
        val r = parseLogin("""{"approval_required":{"poll_token":"p1","device_name":"Pixel 8"}}""")
        assertEquals(LoginResult.ApprovalRequired("p1", "Pixel 8"), r)
    }

    @Test(expected = SerializationException::class)
    fun loginWithoutSessionFails() {
        parseLogin("""{"other":1}""")
    }

    @Test
    fun pollResults() {
        assertEquals(PollResult.Pending, parsePoll("""{"pending":true}"""))
        val ok = parsePoll("""{"token":"t","user":$user}""") as PollResult.SignedIn
        assertEquals("t", ok.session.token)
    }

    @Test
    fun deviceLabels() {
        assertEquals("Google Pixel 8", deviceLabel("Google", "Pixel 8"))
        assertEquals("Google Pixel 8", deviceLabel("Google", "Google Pixel 8"))
        assertEquals("samsung SM-S918B", deviceLabel("samsung", "SM-S918B"))
        assertEquals("Google", deviceLabel("Google", " "))
    }

    @Test
    fun requestList() {
        val json = Json { ignoreUnknownKeys = true }
        val list = json.decodeFromString<List<MediaRequest>>(
            """[{"id":1,"user_id":3,"user_name":"Mia","kind":"movie","title":"Heat","year":1995,"tmdb_id":949,"poster":"w500/x.jpg",
            "overview":null,"note":null,"status":"pending","admin_note":null,"created_at":100,"updated_at":100},
            {"id":2,"user_id":3,"kind":"show","title":"Dark","status":"declined","admin_note":"No","created_at":200}]""",
        )
        assertEquals(2, list.size)
        assertTrue(list[0].pending)
        assertEquals("w500/x.jpg", list[0].poster)
        assertEquals(1995, list[0].year)
        assertEquals("No", list[1].adminNote)
        assertNull(list[1].year)
        assertTrue(!list[1].pending)
    }

    @Test
    fun requestSearch() {
        val json = Json { ignoreUnknownKeys = true }
        val s = json.decodeFromString<RequestSearch>("""{"tmdb":true,"results":[{"tmdb_id":949,"title":"Heat","year":1995,"overview":"Cops","poster":"/abc.jpg"}]}""")
        assertTrue(s.tmdb)
        assertEquals("w500/abc.jpg", tmdbPosterKey(s.results[0].poster))
        assertTrue(!json.decodeFromString<RequestSearch>("""{"tmdb":false,"results":[]}""").tmdb)
    }

    @Test
    fun posterKeys() {
        assertEquals("w500/abc.jpg", tmdbPosterKey("/abc.jpg"))
        assertEquals("w500/abc.jpg", tmdbPosterKey("abc.jpg"))
        assertEquals("w780/abc.jpg", tmdbPosterKey("w780/abc.jpg"))
        assertNull(tmdbPosterKey(null))
        assertNull(tmdbPosterKey("/"))
    }

    @Test
    fun stats() {
        val json = Json { ignoreUnknownKeys = true }
        val s = json.decodeFromString<WatchStats>(
            """{"totals":{"total_s":7200,"last_7d_s":600,"last_30d_s":3600,"titles":4,"completed":2,"last_played_at":null},
            "daily":[{"day":19999,"seconds":60},{"day":20000,"seconds":120}],
            "top_titles":[{"item_id":9,"title":"Dark","kind":"show","poster":null,"seconds":5000}],
            "top_genres":[{"genre":"Drama","seconds":4000}],
            "history":[{"item":{"id":5,"kind":"movie","title":"Heat"},"last_played_at":1700000000}]}""",
        )
        assertEquals(7200, s.totals.totalS)
        assertNull(s.totals.lastPlayedAt)
        assertEquals(9L, s.topTitles[0].itemId)
        assertEquals("Drama", s.topGenres[0].genre)
        assertEquals("Heat", s.history[0].item.title)
        assertEquals(1700000000L, s.history[0].lastPlayedAt)
    }

    @Test
    fun dailySeriesFillsGaps() {
        val v = fillDaily(listOf(DayTime(20000, 120), DayTime(19998, 30), DayTime(10, 5)), today = 20000, days = 5)
        assertEquals(listOf(0L, 0L, 30L, 0L, 120L), v)
        assertEquals(30, fillDaily(emptyList(), 100).size)
    }

    @Test
    fun durations() {
        assertEquals(HoursMinutes(0, 0), hoursMinutes(0))
        assertEquals(HoursMinutes(0, 1), hoursMinutes(31))
        assertEquals(HoursMinutes(2, 5), hoursMinutes(2 * 3600 + 5 * 60 + 10))
        assertEquals(HoursMinutes(0, 0), hoursMinutes(-5))
    }
}
