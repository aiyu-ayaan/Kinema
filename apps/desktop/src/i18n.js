const en = require('./locales/en.json')

/** Catalogs by language tag. To add a language: create `locales/<tag>.json` with the same keys and register it here. */
const catalogs = { en }

function resolveLanguage(requested) {
  for (const tag of requested) {
    if (!tag) continue
    const base = String(tag).split('-')[0].toLowerCase()
    if (base === 'en') return 'en'
    if (catalogs[base]) return base
  }
  return 'en'
}

let language = 'en'
const setLanguage = (...requested) => {
  language = resolveLanguage(requested.flat())
}
const getLanguage = () => language

const messages = () => catalogs[language] ?? en

/** Looks up a message in the active catalog and replaces `{name}` placeholders. */
function t(key, params) {
  const dict = messages()
  const text = dict[key] ?? en[key] ?? key
  return params ? text.replace(/\{(\w+)\}/g, (m, name) => (name in params ? String(params[name]) : m)) : text
}

module.exports = { t, setLanguage, getLanguage, messages, resolveLanguage }
