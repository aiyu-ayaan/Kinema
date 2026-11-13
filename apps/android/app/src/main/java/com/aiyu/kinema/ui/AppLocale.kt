package com.aiyu.kinema.ui

import android.app.Activity
import android.app.LocaleManager
import android.content.Context
import android.content.res.Configuration
import android.os.Build
import android.os.LocaleList
import androidx.core.content.edit
import java.util.Locale

data class LanguageOption(val tag: String, val nativeName: String)

/**
 * Canonical 27-language list and per-app locale persistence.
 * Uses platform LocaleManager on Android 13+ and SharedPreferences + Configuration wrapping on API 26–32.
 */
object AppLocale {
    private const val PREFS = "kinema_locale"
    private const val KEY_TAG = "tag"

    val supported: List<LanguageOption> = listOf(
        LanguageOption("en", "English"),
        LanguageOption("hi", "हिन्दी"),
        LanguageOption("es", "Español"),
        LanguageOption("pt-BR", "Português (Brasil)"),
        LanguageOption("id", "Bahasa Indonesia"),
        LanguageOption("ar", "العربية"),
        LanguageOption("fr", "Français"),
        LanguageOption("ru", "Русский"),
        LanguageOption("de", "Deutsch"),
        LanguageOption("tr", "Türkçe"),
        LanguageOption("ja", "日本語"),
        LanguageOption("ko", "한국어"),
        LanguageOption("it", "Italiano"),
        LanguageOption("vi", "Tiếng Việt"),
        LanguageOption("th", "ไทย"),
        LanguageOption("zh-CN", "中文 (简体)"),
        LanguageOption("zh-TW", "中文 (繁體)"),
        LanguageOption("pl", "Polski"),
        LanguageOption("bn", "বাংলা"),
        LanguageOption("ta", "தமிழ்"),
        LanguageOption("te", "తెలుగు"),
        LanguageOption("mr", "मराठी"),
        LanguageOption("gu", "ગુજરાતી"),
        LanguageOption("kn", "ಕನ್ನಡ"),
        LanguageOption("ml", "മലയാളം"),
        LanguageOption("pa", "ਪੰਜਾਬੀ"),
        LanguageOption("ur", "اردو"),
    )

    fun currentTag(context: Context): String? {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val locales = context.getSystemService(LocaleManager::class.java)?.applicationLocales
            if (locales == null || locales.isEmpty) return null
            return normalizeTag(locales[0]?.toLanguageTag())
        }
        return normalizeTag(context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY_TAG, null))
    }

    fun displayName(tag: String?): String? {
        val normalized = normalizeTag(tag) ?: return null
        return supported.firstOrNull { it.tag.equals(normalized, ignoreCase = true) }?.nativeName
    }

    fun apply(context: Context, tag: String?) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val lm = context.getSystemService(LocaleManager::class.java) ?: return
            lm.applicationLocales = if (tag == null) LocaleList.getEmptyLocaleList() else LocaleList.forLanguageTags(tag)
            return
        }
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit {
            if (tag == null) remove(KEY_TAG) else putString(KEY_TAG, tag)
        }
        (context as? Activity)?.recreate()
    }

    fun wrapContext(base: Context): Context {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) return base
        val tag = base.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY_TAG, null) ?: return base
        val locale = Locale.forLanguageTag(tag)
        Locale.setDefault(locale)
        val config = Configuration(base.resources.configuration)
        config.setLocale(locale)
        config.setLayoutDirection(locale)
        return base.createConfigurationContext(config)
    }

    private fun normalizeTag(raw: String?): String? {
        if (raw.isNullOrBlank()) return null
        // Java/Android sometimes reports legacy "in" for Indonesian.
        if (raw == "in" || raw.startsWith("in-")) return "id"
        val exact = supported.firstOrNull { it.tag.equals(raw, ignoreCase = true) }
        if (exact != null) return exact.tag
        val lang = raw.substringBefore('-')
        return supported.firstOrNull { it.tag.equals(lang, ignoreCase = true) }?.tag ?: raw
    }
}
