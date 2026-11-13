const SUPPORTED_LANGUAGES = [
  { tag: 'en', name: 'English', rtl: false },
  { tag: 'hi', name: 'हिन्दी', rtl: false },
  { tag: 'es', name: 'Español', rtl: false },
  { tag: 'pt-BR', name: 'Português (Brasil)', rtl: false },
  { tag: 'id', name: 'Bahasa Indonesia', rtl: false },
  { tag: 'ar', name: 'العربية', rtl: true },
  { tag: 'fr', name: 'Français', rtl: false },
  { tag: 'ru', name: 'Русский', rtl: false },
  { tag: 'de', name: 'Deutsch', rtl: false },
  { tag: 'tr', name: 'Türkçe', rtl: false },
  { tag: 'ja', name: '日本語', rtl: false },
  { tag: 'ko', name: '한국어', rtl: false },
  { tag: 'it', name: 'Italiano', rtl: false },
  { tag: 'vi', name: 'Tiếng Việt', rtl: false },
  { tag: 'th', name: 'ไทย', rtl: false },
  { tag: 'zh-CN', name: '中文 (简体)', rtl: false },
  { tag: 'zh-TW', name: '中文 (繁體)', rtl: false },
  { tag: 'pl', name: 'Polski', rtl: false },
  { tag: 'bn', name: 'বাংলা', rtl: false },
  { tag: 'ta', name: 'தமிழ்', rtl: false },
  { tag: 'te', name: 'తెలుగు', rtl: false },
  { tag: 'mr', name: 'मराठी', rtl: false },
  { tag: 'gu', name: 'ગુજરાતી', rtl: false },
  { tag: 'kn', name: 'ಕನ್ನಡ', rtl: false },
  { tag: 'ml', name: 'മലയാളം', rtl: false },
  { tag: 'pa', name: 'ਪੰਜਾਬੀ', rtl: false },
  { tag: 'ur', name: 'اردو', rtl: true },
]

const en = require('./locales/en.json')

/** Catalogs by BCP-47 language tag. */
const catalogs = Object.fromEntries(
  SUPPORTED_LANGUAGES.map(({ tag }) => [tag, tag === 'en' ? en : require(`./locales/${tag}.json`)]),
)

const TAG_BY_LOWER = Object.fromEntries(Object.keys(catalogs).map((tag) => [tag.toLowerCase(), tag]))
const RTL_LANGUAGES = new Set(SUPPORTED_LANGUAGES.filter((l) => l.rtl).map((l) => l.tag))

function matchTag(raw) {
  if (!raw) return null
  const normalized = String(raw).trim().replace(/_/g, '-')
  const lower = normalized.toLowerCase()
  if (!lower || lower === 'system' || lower === 'auto') return null
  if (TAG_BY_LOWER[lower]) return TAG_BY_LOWER[lower]

  // Legacy and script-specific mappings
  if (lower === 'in' || lower.startsWith('in-')) return 'id'
  if (lower.startsWith('zh-hant') || lower === 'zh-hk' || lower === 'zh-mo' || lower.startsWith('zh-tw')) {
    return 'zh-TW'
  }
  if (lower === 'zh' || lower.startsWith('zh-hans') || lower === 'zh-sg' || lower.startsWith('zh-cn')) {
    return 'zh-CN'
  }
  if (lower === 'pt' || lower.startsWith('pt-')) return 'pt-BR'

  const base = lower.split('-')[0]
  if (TAG_BY_LOWER[base]) return TAG_BY_LOWER[base]
  return null
}

function resolveLanguage(requested) {
  for (const tag of requested) {
    const matched = matchTag(tag)
    if (matched) return matched
  }
  return 'en'
}

let language = 'en'
const setLanguage = (...requested) => {
  language = resolveLanguage(requested.flat())
}
const getLanguage = () => language
const isRtl = (lang = language) => RTL_LANGUAGES.has(lang)
const getDirection = (lang = language) => (isRtl(lang) ? 'rtl' : 'ltr')

const messages = () => catalogs[language] ?? en

/** Looks up a message in the active catalog and replaces `{name}` placeholders. */
function t(key, params) {
  const dict = messages()
  const text = dict[key] ?? en[key] ?? key
  return params ? text.replace(/\{(\w+)\}/g, (m, name) => (name in params ? String(params[name]) : m)) : text
}

module.exports = {
  t,
  setLanguage,
  getLanguage,
  isRtl,
  getDirection,
  messages,
  resolveLanguage,
  catalogs,
  SUPPORTED_LANGUAGES,
  RTL_LANGUAGES,
}
