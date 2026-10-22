import { t, type MessageKey } from '@/lib/i18n'

/** "PG-13" for a limit on the minimum-age scale (backend `kinema_core::content_rating`). */
export function ratingLabel(age: number | null): string {
  if (age == null) return t('household.noLimit')
  const key = `household.level.${age}` as MessageKey
  const text = t(key)
  return text === key ? t('household.level.other', { age }) : text
}
