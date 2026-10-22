import { parseVtt } from './vtt'

/** One scrub thumbnail: the part `[x, y, w, h]` of a sprite sheet that shows the video from `start` to `end`. */
export interface ScrubTile {
  start: number
  end: number
  url: string
  x: number
  y: number
  w: number
  h: number
}

/** Tiles per sheet row and column; the server cuts its sheets 10 x 10 (media-pipeline.md § 5). */
export const SHEET_GRID = 10

/** The server's `timeline.vtt` as tiles. Cue text is `<sheet url>#xywh=x,y,w,h`, the url relative to `base`. */
export function parseTimeline(vtt: string, base: string): ScrubTile[] {
  const tiles: ScrubTile[] = []
  for (const cue of parseVtt(vtt)) {
    const m = cue.text.match(/^(.+?)#xywh=(\d+),(\d+),(\d+),(\d+)$/)
    if (!m) continue
    tiles.push({ start: cue.start, end: cue.end, url: `${base}/${m[1]}`, x: +m[2], y: +m[3], w: +m[4], h: +m[5] })
  }
  return tiles
}

/** The tile showing second `t` (the last one past the end), or null without tiles. */
export function tileAt(tiles: ScrubTile[], t: number): ScrubTile | null {
  if (tiles.length === 0) return null
  let lo = 0
  let hi = tiles.length - 1
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1
    if (tiles[mid].start <= t) lo = mid
    else hi = mid - 1
  }
  return tiles[lo]
}
