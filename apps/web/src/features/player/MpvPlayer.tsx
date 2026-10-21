import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, CircleNotch, Stop } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { api } from '@/lib/api'
import { isTextSub } from '@/lib/format'
import { t } from '@/lib/i18n'
import { mpvBridge } from '@/lib/mpv'
import type { ItemDetail } from '@/lib/types'

/** Same rule as the in-page player: this far through counts as watched. */
const WATCHED_AT = 0.92

/**
 * Desktop app only: hands the title to mpv, which takes the file as it is (no remux, no transcode), and waits.
 * Progress mpv reports is saved as the in-page player would; closing mpv comes back here and leaves.
 */
export function MpvPlayer({ detail, startAt }: { detail: ItemDetail; startAt: number }) {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const { item } = detail
  const file = detail.files[0]
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!mpvBridge) return
    const bridge = mpvBridge
    let cancelled = false
    const leave = () => (window.history.length > 1 ? navigate(-1) : navigate(`/item/${item.id}`, { replace: true }))
    const unlisten = bridge.onProgress((pos) => api.progress(item.id, pos).catch(() => {}))
    ;(async () => {
      // The server's track picks (language settings, this profile's choice) apply in mpv too.
      const plan = await api.rawPlan(file.id).catch(() => null)
      if (cancelled) return
      const audios = file.streams.filter((s) => s.type === 'audio')
      const aid = audios.findIndex((s) => s.idx === plan?.audio) + 1 || null
      const sub = file.streams.find((s) => s.id === plan?.subtitle)
      const result = await bridge.play({
        fileId: file.id,
        start: startAt,
        title: item.title,
        aid,
        subtitleId: sub && isTextSub(sub) ? sub.id : null,
      })
      if (cancelled) return
      if (result.error) return setError(result.error)
      const pos = result.position ?? 0
      const duration = file.duration_s ?? 0
      const watched = !!result.ended || (duration > 0 && pos / duration > WATCHED_AT)
      if (pos > 0 || watched) await api.progress(item.id, pos, watched).catch(() => {})
      qc.invalidateQueries({ queryKey: ['home'] })
      qc.invalidateQueries({ queryKey: ['item'] })
      if (result.ended && detail.next_id) navigate(`/play/${detail.next_id}?t=0&mpv=1`, { replace: true })
      else leave()
    })()
    return () => {
      cancelled = true
      unlisten()
      bridge.stop()
    }
    // Once per title: the page is keyed by item id.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <div className="fixed inset-0 z-sheet flex flex-col items-center justify-center gap-4 bg-black p-6 text-center text-white">
      {error ? (
        <>
          <p className="max-w-md text-body text-white/80">{error}</p>
          <Button variant="ghost" icon={<ArrowLeft size={18} />} onClick={() => navigate(-1)}>
            {t('player.mpv.back')}
          </Button>
        </>
      ) : (
        <>
          <CircleNotch size={40} className="animate-spin text-primary" aria-hidden />
          <p className="text-title font-semibold">{t('player.mpv.playing')}</p>
          <p className="max-w-md truncate text-body text-white/80">{item.title}</p>
          <p className="max-w-md text-label text-white/60">{t('player.mpv.hint')}</p>
          <Button variant="ghost" icon={<Stop size={18} weight="fill" />} onClick={() => mpvBridge?.stop()}>
            {t('player.mpv.stop')}
          </Button>
        </>
      )}
    </div>
  )
}
