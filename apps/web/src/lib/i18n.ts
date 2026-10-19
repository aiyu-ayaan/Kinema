import en from '@/locales/en.json'

export type MessageKey = keyof typeof en

/**
 * Every user-facing string lives in `locales/<lang>.json`. English only for now;
 * to add a language, add a catalog with the same keys and pick it here.
 */
const catalog: Record<string, string> = en

/** `t('home.greeting', { name })` fills `{name}` placeholders. */
export function t(key: MessageKey, vars?: Record<string, string | number>): string {
  const text = catalog[key] ?? key
  if (!vars) return text
  return text.replace(/\{(\w+)\}/g, (m, name: string) => (name in vars ? String(vars[name]) : m))
}
