export interface DayTotal {
  /** Unix seconds / 86400, as the server buckets watch time. */
  day: number
  seconds: number
}

/**
 * The last `days` days ending today, oldest first. The server leaves out days without play,
 * so the chart gets them back as zeros.
 */
export function fillDaily(daily: DayTotal[], days = 30, nowMs = Date.now()): DayTotal[] {
  const today = Math.floor(nowMs / 86_400_000)
  const byDay = new Map(daily.map((d) => [d.day, d.seconds]))
  return Array.from({ length: days }, (_, i) => {
    const day = today - days + 1 + i
    return { day, seconds: byDay.get(day) ?? 0 }
  })
}

/** 1, 2 -> "S01E02". */
export function episodeCode(season: number | null, episode: number | null): string {
  const pad = (n: number | null) => String(n ?? 0).padStart(2, '0')
  return `S${pad(season)}E${pad(episode)}`
}

/** A player that has not reported for this long is most likely paused. */
export const PAUSED_AFTER_S = 15
