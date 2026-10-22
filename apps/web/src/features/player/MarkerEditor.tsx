import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { clock, parseClock } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { Marker } from '@/lib/types'
import { Button } from '@/components/ui/Button'
import { inputCls } from '@/components/ui/Field'

type Kind = Marker['kind']
const KINDS: Kind[] = ['intro', 'credits']

/** Admin panel over the player: view, set from the playhead, or clear the intro and credits of the playing file. */
export function MarkerEditor({ itemId, markers, time, duration }: { itemId: number; markers: Marker[]; time: number; duration: number | null }) {
  return (
    <div className="anim-enter absolute bottom-28 start-4 z-10 flex w-[min(420px,calc(100vw-2rem))] flex-col gap-4 rounded-lg border border-white/10 bg-black/85 p-3 shadow-overlay backdrop-blur-md">
      <div className="px-1">
        <p className="text-caption font-semibold uppercase tracking-wider text-white/50">{t('player.markers.title')}</p>
        <p className="mt-1 text-caption text-white/60">{t('player.markers.note')}</p>
      </div>
      {KINDS.map((kind) => (
        <MarkerRow key={kind} itemId={itemId} kind={kind} marker={markers.find((m) => m.kind === kind)} time={time} duration={duration} />
      ))}
    </div>
  )
}

function MarkerRow({ itemId, kind, marker, time, duration }: { itemId: number; kind: Kind; marker: Marker | undefined; time: number; duration: number | null }) {
  const qc = useQueryClient()
  // Text fields, so a time can be typed as well as taken from the playhead.
  const [start, setStart] = useState(marker ? clock(marker.start_s) : '')
  const [end, setEnd] = useState(marker ? clock(marker.end_s) : '')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const from = parseClock(start)
  const to = parseClock(end)
  const valid = from != null && to != null && to > from && (duration == null || to <= duration + 1)

  const run = async (work: () => Promise<void>) => {
    setBusy(true)
    setError(null)
    try {
      await work()
      await qc.invalidateQueries({ queryKey: ['item', itemId] })
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const field = (label: string, value: string, set: (v: string) => void, parsed: number | null) => (
    <div className="flex min-w-0 flex-1 flex-col gap-1">
      <label className="text-caption text-white/60">{label}</label>
      <div className="flex gap-1">
        <input
          value={value}
          onChange={(e) => set(e.target.value)}
          inputMode="numeric"
          aria-invalid={value !== '' && parsed == null}
          placeholder="0:00"
          className={cn(inputCls, 'h-8 min-w-0 flex-1 px-2 tabular text-label', value !== '' && parsed == null && 'border-danger')}
        />
        <Button size="sm" variant="secondary" onClick={() => set(clock(time))} title={t('player.markers.useNow')} aria-label={`${label}: ${t('player.markers.useNow')}`}>
          {t('player.markers.now')}
        </Button>
      </div>
    </div>
  )

  return (
    <section className="flex flex-col gap-2 rounded-md bg-white/5 p-2.5" aria-label={t(`player.markers.${kind}`)}>
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="text-label font-semibold text-white">{t(`player.markers.${kind}`)}</h3>
        <p className="truncate text-caption text-white/60">
          {marker ? `${clock(marker.start_s)} – ${clock(marker.end_s)} · ${t(`player.markers.source.${marker.source}`)}` : t('player.markers.none')}
        </p>
      </div>
      <div className="flex gap-2">
        {field(t('player.markers.start'), start, setStart, from)}
        {field(t('player.markers.end'), end, setEnd, to)}
      </div>
      {error && (
        <p role="alert" className="text-caption text-danger">
          {error}
        </p>
      )}
      <div className="flex items-center gap-2">
        <Button size="sm" variant="primary" disabled={!valid} loading={busy} onClick={() => run(() => api.setMarker(itemId, kind, from ?? 0, to ?? 0))}>
          {t('player.markers.save')}
        </Button>
        {marker && (
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            className="text-white/80 hover:text-white"
            onClick={() =>
              run(async () => {
                await api.clearMarker(itemId, kind)
                setStart('')
                setEnd('')
              })
            }
          >
            {t('player.markers.clear')}
          </Button>
        )}
        {start !== '' && end !== '' && !valid && <span className="text-caption text-white/60">{t('player.markers.invalid')}</span>}
      </div>
    </section>
  )
}
