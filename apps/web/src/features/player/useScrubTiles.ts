import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import { parseTimeline, type ScrubTile } from '@/lib/scrub'

/** How long to wait before asking again while the server makes the sheets, and how many times. */
const RETRY_MS = 15_000
const MAX_TRIES = 40

/**
 * The scrub tiles of a title. The first request makes the server start the sheets, so the preview appears some
 * time into playback; until then (or when the file has none) this stays empty and the bar shows the time only.
 */
export function useScrubTiles(itemId: number): ScrubTile[] {
  const [tiles, setTiles] = useState<ScrubTile[]>([])
  useEffect(() => {
    let cancelled = false
    let timer: number | undefined
    let tries = 0
    const load = async () => {
      try {
        const text = await api.timelineText(itemId)
        if (cancelled) return
        if (text != null) {
          setTiles(parseTimeline(text, `/api/items/${itemId}`))
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
  }, [itemId])
  return tiles
}
