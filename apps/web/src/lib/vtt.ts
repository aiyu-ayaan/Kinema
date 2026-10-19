export interface Cue {
  start: number
  end: number
  text: string
}

/** "01:02:03.500" or "02:03.500" to seconds. */
function seconds(stamp: string): number {
  const parts = stamp.trim().replace(',', '.').split(':').map(Number)
  return parts.reduce((acc, p) => acc * 60 + p, 0)
}

/**
 * Minimal WebVTT reader. Cues are drawn by the player itself so a track switch
 * needs no video reload and cue times line up with the absolute play position.
 */
export function parseVtt(text: string): Cue[] {
  const cues: Cue[] = []
  for (const block of text.replace(/\r/g, '').split(/\n{2,}/)) {
    const lines = block.split('\n')
    const at = lines.findIndex((l) => l.includes('-->'))
    if (at < 0) continue
    const [from, rest] = lines[at].split('-->')
    const start = seconds(from)
    const end = seconds(rest.trim().split(/\s+/)[0])
    const body = lines
      .slice(at + 1)
      .join('\n')
      .replace(/<[^>]+>/g, '')
      .replace(/\{\\[^}]*\}/g, '')
      .trim()
    if (body && Number.isFinite(start) && Number.isFinite(end)) cues.push({ start, end, text: body })
  }
  return cues.sort((a, b) => a.start - b.start)
}
