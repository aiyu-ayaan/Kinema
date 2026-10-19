import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Translate } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Select } from '@/components/ui/Controls'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { isTextSub, trackLabel } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { SeriesTracks as Tracks } from '@/lib/types'

// '' is "server default"; the server stores positions among the first episode's tracks.
const toValue = (n: number | null) => (n == null ? '' : String(n))
const fromValue = (v: string) => (v === '' ? null : Number(v))

/** Admin choice of audio and subtitle track for every episode of a show. */
export function SeriesTracks({ showId, tracks }: { showId: number; tracks: Tracks }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [audio, setAudio] = useState(toValue(tracks.audio))
  const [sub, setSub] = useState(toValue(tracks.subtitle))
  const audios = tracks.streams.filter((s) => s.type === 'audio')
  const subs = tracks.streams.filter((s) => s.type === 'subtitle')
  const save = useMutation({
    mutationFn: () => api.setSeriesTracks(showId, fromValue(audio), fromValue(sub)),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['item', showId] })
      setOpen(false)
    },
  })

  return (
    <>
      <Button
        size="lg"
        icon={<Translate size={20} />}
        aria-label={t('series.tracks')}
        title={t('series.tracks')}
        onClick={() => {
          setAudio(toValue(tracks.audio))
          setSub(toValue(tracks.subtitle))
          setOpen(true)
        }}
      />
      <Modal
        open={open}
        onClose={() => setOpen(false)}
        title={t('series.title')}
        footer={
          <>
            <Button onClick={() => setOpen(false)}>{t('common.cancel')}</Button>
            <Button variant="primary" loading={save.isPending} disabled={tracks.streams.length === 0} onClick={() => save.mutate()}>
              {t('common.save')}
            </Button>
          </>
        }
      >
        {tracks.streams.length === 0 ? (
          <p className="text-body text-muted">{t('series.noStreams')}</p>
        ) : (
          <div className="flex flex-col gap-4">
            <p className="text-label text-muted">{t('series.hint')}</p>
            <label className="flex flex-col gap-1.5 text-label font-medium">
              {t('series.audio')}
              <Select value={audio} onChange={(e) => setAudio(e.target.value)}>
                <option value="">{t('series.serverDefault')}</option>
                {audios.map((a, i) => (
                  <option key={a.id} value={i}>
                    {trackLabel(a, i + 1)}
                  </option>
                ))}
              </Select>
            </label>
            <label className="flex flex-col gap-1.5 text-label font-medium">
              {t('series.subtitles')}
              <Select value={sub} onChange={(e) => setSub(e.target.value)}>
                <option value="">{t('series.serverDefault')}</option>
                <option value="-1">{t('player.subtitlesOff')}</option>
                {subs.map((s, i) => {
                  const text = isTextSub(s)
                  return (
                    <option key={s.id} value={i} disabled={!text}>
                      {trackLabel(s, i + 1)}
                      {text ? '' : ` (${t('player.subtitleImage')})`}
                    </option>
                  )
                })}
              </Select>
            </label>
            <ErrorText>{save.error instanceof Error ? save.error.message : null}</ErrorText>
          </div>
        )}
      </Modal>
    </>
  )
}
