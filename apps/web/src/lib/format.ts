import { t } from './i18n'
import type { ItemCard, Stream } from './types'

/** 5400 -> "1h 30m"; 300 -> "5m". */
export function runtime(seconds: number | null | undefined): string | null {
  if (!seconds || seconds <= 0) return null
  const m = Math.round(seconds / 60)
  const h = Math.floor(m / 60)
  return h > 0 ? t('format.hoursMinutes', { h, m: m % 60 }) : t('format.minutes', { m })
}

/** 3725 -> "1:02:05"; 65 -> "1:05". */
export function clock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds))
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = String(s % 60).padStart(2, '0')
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`
}

const languageNames = (() => {
  try {
    return new Intl.DisplayNames([navigator.language, 'en'], { type: 'language' })
  } catch {
    return null
  }
})()

/** "eng" or "en" -> "English". */
export function languageName(code: string | null): string | null {
  if (!code) return null
  try {
    return languageNames?.of(code) ?? code.toUpperCase()
  } catch {
    return code.toUpperCase()
  }
}

function channelLabel(ch: number | null): string | null {
  if (!ch) return null
  if (ch === 1) return t('track.mono')
  if (ch === 2) return t('track.stereo')
  return `${ch - 1}.1`
}

/** "English · AC3 5.1 · Commentary" */
export function trackLabel(s: Stream, n: number): string {
  const parts = [
    languageName(s.language) ?? t('track.numbered', { n }),
    [s.codec?.toUpperCase(), s.type === 'audio' ? channelLabel(s.channels) : null].filter(Boolean).join(' '),
    s.title,
  ]
  return parts.filter(Boolean).join(' · ')
}

/** "S1 · E5" for episodes, year for the rest. */
export function subtitleOf(item: ItemCard): string | null {
  if (item.kind === 'episode' && item.episode_no != null) {
    return t('item.episodeShort', { s: item.season_no ?? 0, e: item.episode_no })
  }
  if (item.kind === 'show' && item.child_count > 0) return t(item.child_count === 1 ? 'item.seasonCount.one' : 'item.seasonCount', { n: item.child_count })
  if (item.kind === 'season') return t(item.child_count === 1 ? 'item.episodeCount.one' : 'item.episodeCount', { n: item.child_count })
  return item.year ? String(item.year) : null
}

/** 0..1 watched fraction for a progress bar, or null when there is nothing to show. */
export function progressOf(item: ItemCard): number | null {
  if (!item.position_s || !item.duration_s || item.watched) return null
  return Math.min(1, item.position_s / item.duration_s)
}

export function fileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let v = bytes
  let i = 0
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i++
  }
  return `${v.toFixed(i >= 3 ? 1 : 0)} ${units[i]}`
}
