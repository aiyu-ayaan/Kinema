package com.aiyu.kinema.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AppLocaleTest {
    @Test
    fun supportedHasAll27CanonicalLanguages() {
        assertEquals(27, AppLocale.supported.size)
        val tags = AppLocale.supported.map { it.tag }
        assertEquals(tags.size, tags.distinct().size)
        assertTrue("en" in tags)
        assertTrue("pt-BR" in tags)
        assertTrue("id" in tags)
        assertTrue("zh-CN" in tags)
        assertTrue("zh-TW" in tags)
        assertTrue("ar" in tags)
        assertTrue("ur" in tags)
    }

    @Test
    fun displayNameResolvesCanonicalAndLegacyTags() {
        assertEquals("English", AppLocale.displayName("en"))
        assertEquals("Bahasa Indonesia", AppLocale.displayName("id"))
        assertEquals("Bahasa Indonesia", AppLocale.displayName("in"))
        assertEquals("Português (Brasil)", AppLocale.displayName("pt-BR"))
        assertEquals("中文 (简体)", AppLocale.displayName("zh-CN"))
        assertEquals("中文 (繁體)", AppLocale.displayName("zh-TW"))
        assertNull(AppLocale.displayName(null))
        assertNull(AppLocale.displayName(""))
    }
}
