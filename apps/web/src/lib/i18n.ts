import { useSyncExternalStore } from 'react'
import en from '@/locales/en.json'
import { storageKeys } from './flavor'
import { formatMessage, isRtl, resolveLanguage, SUPPORTED_LANGUAGES, type LanguageCode } from './languages'

export { isRtl, resolveLanguage, SUPPORTED_LANGUAGES, type LanguageCode }
export type MessageKey = keyof typeof en

const localeModules = import.meta.glob<Record<string, string>>('../locales/*.json', { import: 'default' })
const catalogs: Partial<Record<LanguageCode, Record<string, string>>> = { en }
const listeners = new Set<() => void>()

function readStoredLanguage(): LanguageCode | null {
  try {
    const raw = localStorage.getItem(storageKeys.language)
    return raw ? resolveLanguage(raw) : null
  } catch {
    return null
  }
}

function detectInitialLanguage(): LanguageCode {
  const stored = readStoredLanguage()
  if (stored) return stored
  if (typeof navigator !== 'undefined') return resolveLanguage(navigator.language)
  return 'en'
}

let current: LanguageCode = detectInitialLanguage()

function syncDocument(lang: LanguageCode) {
  if (typeof document === 'undefined') return
  document.documentElement.lang = lang
  document.documentElement.dir = isRtl(lang) ? 'rtl' : 'ltr'
}

async function loadCatalog(lang: LanguageCode): Promise<void> {
  if (catalogs[lang]) return
  const loader = localeModules[`../locales/${lang}.json`]
  if (!loader) return
  try {
    catalogs[lang] = await loader()
  } catch {
    // Fall back to `en` if a locale chunk fails to load.
  }
}

/** Load the initial locale catalog (if non-English) and apply `<html lang dir>`. */
export async function initLanguage(): Promise<LanguageCode> {
  await loadCatalog(current)
  syncDocument(current)
  return current
}

export function getLanguage(): LanguageCode {
  return current
}

export async function setLanguage(lang: string): Promise<LanguageCode> {
  const next = resolveLanguage(lang)
  await loadCatalog(next)
  current = next
  try {
    localStorage.setItem(storageKeys.language, next)
  } catch {
    /* not persisted in private mode */
  }
  syncDocument(next)
  listeners.forEach((l) => l())
  return next
}

export function useLanguage(): LanguageCode {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => current,
  )
}

/** `t('home.greeting', { name })` fills `{name}` placeholders. */
export function t(key: MessageKey, vars?: Record<string, string | number>): string {
  const text = catalogs[current]?.[key] ?? en[key] ?? key
  return formatMessage(text, vars)
}
