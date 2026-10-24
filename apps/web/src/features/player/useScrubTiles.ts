import { useEffect, useState } from 'react'
import { api, session } from '@/lib/api'
import { parseTimeline, type ScrubTile } from '@/lib/scrub'

/** How long to wait before asking again while the server makes the sheets, and how many times. */
const RETRY_MS = 15_000
const MAX_TRIES = 40

/**
 * The scrub tiles of a title, or none without asking when the server has the feature off (`detail.features`).
 * The first request makes the server start the sheets, so the preview appears some
 * time into playback; until then (or when the file has none) this stays empty and the bar shows the time only.
 */
export function useScrubTiles(itemId: number, enabled: boolean): ScrubTile[] {
  const [tiles, setTiles] = useState<ScrubTile[]>([])
  useEffect(() => {
    if (!enabled) return
    let cancelled = false
    let timer: number | undefined
    let tries = 0
    const load = async () => {
      try {
        const text = await api.timelineText(itemId)
        if (cancelled) return
        if (text != null) {
          const parsed = parseTimeline(text, `/api/items/${itemId}`)
          const token = session.token
          const query = token ? `?token=${encodeURIComponent(token)}` : ''
          setTiles(query ? parsed.map((t) => ({ ...t, url: `${t.url}${query}` })) : parsed)
        } else if (++tries < MAX_TRIES) {
          timer = window.setTimeout(load, RETRY_MS)
        }
      } catch {
        // No sheets for this file: the time label alone is the preview.
      }
    }
    load()
    return () => {
      cancelled = true
      window.clearTimeout(timer)
    }
  }, [itemId, enabled])
  return tiles
}
