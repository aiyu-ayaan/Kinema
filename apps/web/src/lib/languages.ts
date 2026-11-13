export const SUPPORTED_LANGUAGES = [
  { code: 'en', nativeName: 'English', rtl: false },
  { code: 'hi', nativeName: 'हिन्दी', rtl: false },
  { code: 'es', nativeName: 'Español', rtl: false },
  { code: 'pt-BR', nativeName: 'Português (Brasil)', rtl: false },
  { code: 'id', nativeName: 'Bahasa Indonesia', rtl: false },
  { code: 'ar', nativeName: 'العربية', rtl: true },
  { code: 'fr', nativeName: 'Français', rtl: false },
  { code: 'ru', nativeName: 'Русский', rtl: false },
  { code: 'de', nativeName: 'Deutsch', rtl: false },
  { code: 'tr', nativeName: 'Türkçe', rtl: false },
  { code: 'ja', nativeName: '日本語', rtl: false },
  { code: 'ko', nativeName: '한국어', rtl: false },
  { code: 'it', nativeName: 'Italiano', rtl: false },
  { code: 'vi', nativeName: 'Tiếng Việt', rtl: false },
  { code: 'th', nativeName: 'ไทย', rtl: false },
  { code: 'zh-CN', nativeName: '中文 (简体)', rtl: false },
  { code: 'zh-TW', nativeName: '中文 (繁體)', rtl: false },
  { code: 'pl', nativeName: 'Polski', rtl: false },
  { code: 'bn', nativeName: 'বাংলা', rtl: false },
  { code: 'ta', nativeName: 'தமிழ்', rtl: false },
  { code: 'te', nativeName: 'తెలుగు', rtl: false },
  { code: 'mr', nativeName: 'मराठी', rtl: false },
  { code: 'gu', nativeName: 'ગુજરાતી', rtl: false },
  { code: 'kn', nativeName: 'ಕನ್ನಡ', rtl: false },
  { code: 'ml', nativeName: 'മലയാളം', rtl: false },
  { code: 'pa', nativeName: 'ਪੰਜਾਬੀ', rtl: false },
  { code: 'ur', nativeName: 'اردو', rtl: true },
] as const

export type LanguageCode = (typeof SUPPORTED_LANGUAGES)[number]['code']

export const RTL_LANGUAGES: ReadonlySet<string> = new Set(
  SUPPORTED_LANGUAGES.filter((l) => l.rtl).map((l) => l.code),
)

const byLowerCode = new Map<string, LanguageCode>(
  SUPPORTED_LANGUAGES.map((l) => [l.code.toLowerCase(), l.code]),
)

export function isRtl(code: string): boolean {
  return RTL_LANGUAGES.has(code)
}

/** Resolve any BCP-47 / ISO tag (e.g. `zh_TW`, `in`, `pt-PT`, `ar-EG`) to a supported locale, falling back to `en`. */
export function resolveLanguage(tag: string | null | undefined): LanguageCode {
  if (!tag) return 'en'
  const normalized = tag.trim().replace(/_/g, '-').toLowerCase()
  if (!normalized) return 'en'

  const exact = byLowerCode.get(normalized)
  if (exact) return exact

  if (normalized === 'in' || normalized.startsWith('in-')) return 'id'
  if (normalized === 'zh-hant' || normalized.startsWith('zh-hant-') || normalized === 'zh-hk' || normalized === 'zh-mo') {
    return 'zh-TW'
  }
  if (normalized === 'zh' || normalized === 'zh-hans' || normalized.startsWith('zh-hans-') || normalized === 'zh-sg') {
    return 'zh-CN'
  }
  if (normalized === 'pt' || normalized.startsWith('pt-')) return 'pt-BR'

  const base = normalized.split('-')[0]
  return byLowerCode.get(base) ?? 'en'
}

export function formatMessage(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template
  return template.replace(/\{(\w+)\}/g, (m, name: string) => (name in vars ? String(vars[name]) : m))
}
