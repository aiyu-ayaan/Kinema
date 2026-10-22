import { clock } from '@/lib/format'
import { SHEET_GRID, tileAt, type ScrubTile } from '@/lib/scrub'

const SCALE = 1.25

/** The bubble above the seek bar: the frame at `time` when the sheets exist, and the time. `frac` is 0..1 along the bar. */
export function ScrubPreview({ tiles, time, frac }: { tiles: ScrubTile[]; time: number; frac: number }) {
  const tile = tileAt(tiles, time)
  // Without a frame the bubble is only the time label, about this wide.
  const w = tile ? tile.w * SCALE : 56
  return (
    <div
      className="pointer-events-none absolute bottom-full z-10 mb-3 flex -translate-x-1/2 flex-col items-center gap-1"
      style={{ left: `clamp(${w / 2}px, ${frac * 100}%, calc(100% - ${w / 2}px))` }}
    >
      {tile && (
        <div
          className="overflow-hidden rounded-md border border-white/20 bg-black shadow-overlay"
          style={{
            width: tile.w * SCALE,
            height: tile.h * SCALE,
            backgroundImage: `url(${tile.url})`,
            backgroundSize: `${tile.w * SHEET_GRID * SCALE}px ${tile.h * SHEET_GRID * SCALE}px`,
            backgroundPosition: `-${tile.x * SCALE}px -${tile.y * SCALE}px`,
          }}
        />
      )}
      <span className="rounded bg-black/80 px-1.5 py-0.5 text-caption font-medium tabular text-white">{clock(time)}</span>
    </div>
  )
}
