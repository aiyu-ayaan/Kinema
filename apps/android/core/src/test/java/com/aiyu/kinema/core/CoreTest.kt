package com.aiyu.kinema.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CoreTest {
    @Test
    fun discoveryReply() {
        val s = ServerDiscovery.parseReply("""{"service":"kinema","name":"Den","port":3000,"tls":true,"version":"0.1.0"}""", "192.168.1.5")
        assertEquals(FoundServer("Den", "https://192.168.1.5:3000", "0.1.0"), s)
        assertEquals("http://192.168.1.5:80", ServerDiscovery.parseReply("""{"service":"kinema","port":80,"tls":false}""", "192.168.1.5")?.url)
        assertEquals("https://[fe80::1]:3000", ServerDiscovery.parseReply("""{"service":"kinema","port":3000}""", "fe80::1%wlan0")?.url)
        assertNull(ServerDiscovery.parseReply("""{"service":"domus","port":8443}""", "1.2.3.4"))
        assertNull(ServerDiscovery.parseReply("""{"service":"kinema","port":0}""", "1.2.3.4"))
        assertNull(ServerDiscovery.parseReply("garbage", "1.2.3.4"))
    }

    @Test
    fun sharperArt() {
        val api = KinemaApi("https://h:3000", null) { null }
        assertEquals("https://h:3000/api/images/w780/abc.jpg", api.sharperImageUrl("w500/abc.jpg", 780))
        assertEquals("https://h:3000/api/images/w780/still.jpg", api.sharperImageUrl("w300/still.jpg", 780))
        assertNull(api.sharperImageUrl("w1280/back.jpg", 780))
        assertNull(api.sharperImageUrl("tvmaze/original_untouched-1-2.jpg", 780))
        assertNull(api.sharperImageUrl(null, 780))
    }

    @Test
    fun serverUrls() {
        assertEquals("https://192.168.1.10:3000", normalizeServerUrl("192.168.1.10"))
        assertEquals("https://192.168.1.10:3001", normalizeServerUrl(" 192.168.1.10:3001/ "))
        assertEquals("https://media.example.com", normalizeServerUrl("media.example.com"))
        assertEquals("http://nas.local:3000", normalizeServerUrl("http://nas.local:3000/api"))
        assertNull(normalizeServerUrl(""))
        assertNull(normalizeServerUrl("not a url"))
        assertTrue(isLanHost("172.20.0.1"))
        assertFalse(isLanHost("8.8.8.8"))
    }

    @Test
    fun vtt() {
        val cues = parseVtt("WEBVTT\n\n1\n00:00:01.000 --> 00:00:02.500 line:90%\n<i>Hello</i>\nthere\n\n01:02:03.000 --> 01:02:04.000\nBye\n")
        assertEquals(listOf(Cue(1.0, 2.5, "Hello\nthere"), Cue(3723.0, 3724.0, "Bye")), cues)
    }

    @Test
    fun progress() {
        val card = ItemCard(id = 1, kind = "movie", title = "A", positionS = 30.0, durationS = 120.0)
        assertEquals(0.25f, card.progress)
        assertNull(card.copy(watched = true).progress)
        assertTrue(card.playable)
        assertFalse(card.copy(kind = "show").playable)
    }
}
