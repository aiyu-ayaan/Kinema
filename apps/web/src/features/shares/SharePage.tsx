import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, LinkBreak, Play } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { EmptyState, Skeleton } from '@/components/ui/Card'
import { shareApi, type PlayCaps } from '@/lib/api'
import { isDev } from '@/lib/flavor'
import { t } from '@/lib/i18n'
import type { ItemCard, ShareFile } from '@/lib/types'

/** Same codec probe idea as the main player, kept tiny: HEVC/AV1 only where the browser says so. */
function caps(): PlayCaps {
  const v = document.createElement('video')
  return {
    hevc: v.canPlayType('video/mp4; codecs="hvc1.1.6.L93.B0"') === 'probably',
    av1: v.canPlayType('video/mp4; codecs="av01.0.05M.08"') === 'probably',
  }
}

function itemLabel(c: ItemCard) {
  if (c.kind === 'episode' && c.episode_no != null) return `${c.episode_no}. ${c.title}`
  return c.title
}

function Player({ token, file }: { token: string; file: ShareFile }) {
  const api = useMemo(() => shareApi(token), [token])
  const playCaps = useMemo(caps, [])
  const { data: plan, error } = useQuery({ queryKey: ['share-plan', token, file.id], queryFn: () => api.plan(file.id, null, playCaps) })
  if (error) return <p className="text-body text-danger">{(error as Error).message}</p>
  if (!plan) return <Skeleton className="aspect-video w-full" />
  const subtitles = file.streams.filter((s) => s.type === 'subtitle')
  return (
    <video className="aspect-video w-full rounded-lg bg-black" controls autoPlay src={api.streamUrl(file.id, plan.audio, playCaps)}>
      {subtitles.map((s) => (
        <track key={s.id} kind="subtitles" src={api.subtitleUrl(s.id)} srcLang={s.language ?? undefined} label={s.title ?? s.language ?? `#${s.idx}`} default={s.id === plan.subtitle} />
      ))}
    </video>
  )
}

/** Public page for a share link: one title, what is under it, and a player. No profile, no session. */
export function SharePage({ token }: { token: string }) {
  const api = useMemo(() => shareApi(token), [token])
  const [itemId, setItemId] = useState<number | undefined>(undefined)
  const [playing, setPlaying] = useState(false)
  const { data, isLoading, error } = useQuery({ queryKey: ['share', token, itemId], queryFn: () => api.view(itemId), retry: false })

  const open = (id: number | undefined) => {
    setPlaying(false)
    setItemId(id)
  }

  return (
    <main className="mx-auto flex min-h-dvh max-w-5xl flex-col gap-6 px-4 py-8">
      <header className="flex items-center gap-3">
        <img src={isDev ? '/favicon-dev.svg' : '/favicon.svg'} alt="" className="h-8 w-8" />
        <span className="text-label font-medium text-muted">{t('sharePage.sharedWithYou')}</span>
      </header>
      {isLoading && <Skeleton className="h-72" />}
      {error && <EmptyState icon={<LinkBreak size={28} />} title={t('share.title')} message={t('sharePage.unavailable')} />}
      {data && (
        <>
          {itemId != null && itemId !== data.share.target_id && (
            <Button variant="ghost" className="self-start" icon={<ArrowLeft size={16} />} onClick={() => open(data.item.parent_id === data.share.target_id ? undefined : data.item.parent_id ?? undefined)}>
              {t('sharePage.backToTop')}
            </Button>
          )}
          <section className="flex flex-col gap-4 sm:flex-row">
            {data.item.poster && <img src={api.imageUrl(data.item.poster) ?? undefined} alt="" className="aspect-[2/3] w-40 shrink-0 rounded-lg object-cover" />}
            <div className="min-w-0">
              {data.item.show_title && <p className="text-label text-muted">{data.item.show_title}</p>}
              <h1 className="text-page font-semibold">{itemLabel(data.item)}</h1>
              {data.item.year && <p className="text-caption text-muted">{data.item.year}</p>}
              {data.item.summary && <p className="mt-3 max-w-prose text-body">{data.item.summary}</p>}
              <p className="mt-3 text-caption text-subtle">{t('sharePage.expires', { date: new Date(data.share.expires_at * 1000).toLocaleString() })}</p>
              {data.files.length > 0 && !playing && (
                <Button variant="primary" className="mt-4" icon={<Play size={16} weight="fill" />} onClick={() => setPlaying(true)}>
                  {t('sharePage.play')}
                </Button>
              )}
            </div>
          </section>
          {playing && data.files[0] && <Player token={token} file={data.files[0]} />}
          {data.children.length > 0 && (
            <section>
              <h2 className="mb-3 text-title font-semibold">{t(data.item.kind === 'show' ? 'sharePage.seasons' : 'sharePage.episodes')}</h2>
              <ul className="divide-y divide-border rounded-lg border border-border">
                {data.children.map((c) => (
                  <li key={c.id}>
                    <button type="button" onClick={() => open(c.id)} className="flex min-h-11 w-full cursor-pointer items-center gap-3 px-4 py-2 text-start hover:bg-raised">
                      <span className="text-body">{itemLabel(c)}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          )}
        </>
      )}
    </main>
  )
}
