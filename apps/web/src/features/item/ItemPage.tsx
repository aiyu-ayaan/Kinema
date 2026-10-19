import type { ReactNode } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { ArrowCounterClockwise, CheckCircle, Play, Star } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { PosterGrid } from '@/components/media/PosterCard'
import { Badge, Skeleton } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { api, imageUrl } from '@/lib/api'
import { clock, fileSize, progressOf, runtime, subtitleOf, trackLabel } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ItemCard, ItemDetail } from '@/lib/types'

function Hero({ detail, children }: { detail: ItemDetail; children?: ReactNode }) {
  const { item, genres, files } = detail
  const backdrop = imageUrl(item.backdrop)
  const duration = files[0]?.duration_s ?? item.duration_s
  const facts = [
    item.kind === 'episode' ? subtitleOf(item) : item.year,
    runtime(duration),
    item.kind === 'show' || item.kind === 'season' ? subtitleOf(item) : null,
  ].filter(Boolean)
  return (
    <section className="relative -mx-4 -mt-6 mb-8 overflow-hidden md:-mx-8 md:-mt-8">
      {backdrop && (
        <div aria-hidden className="absolute inset-0">
          <img src={backdrop} alt="" className="h-full w-full object-cover opacity-40" />
          <div className="absolute inset-0 bg-gradient-to-t from-bg via-bg/80 to-bg/30" />
          <div className="absolute inset-0 bg-gradient-to-r from-bg/90 to-transparent" />
        </div>
      )}
      <div className="relative flex flex-col gap-6 px-4 pb-6 pt-8 sm:flex-row sm:items-end md:px-8 md:pt-16">
        <Artwork image={item.kind === 'episode' ? (item.backdrop ?? item.poster) : item.poster} title={item.show_title ?? item.title} aspect={item.kind === 'episode' ? 'wide' : 'poster'} eager className={item.kind === 'episode' ? 'w-72 shrink-0 shadow-overlay' : 'w-40 shrink-0 shadow-overlay md:w-52'} />
        <div className="min-w-0 max-w-3xl">
          {item.show_title && (
            <Link to={`/item/${item.parent_id}`} className="text-label font-semibold uppercase tracking-wider text-primary hover:underline">
              {item.show_title}
            </Link>
          )}
          <h1 className="mt-1 text-hero font-bold tracking-[-0.02em]">{item.title}</h1>
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-body text-muted">
            {facts.map((f, i) => (
              <span key={i}>{f}</span>
            ))}
            {item.rating != null && (
              <span className="flex items-center gap-1">
                <Star size={14} weight="fill" className="text-info" aria-hidden />
                {item.rating.toFixed(1)}
              </span>
            )}
            {item.watched && (
              <span className="flex items-center gap-1 text-primary">
                <CheckCircle size={14} weight="fill" aria-hidden />
                {t('item.watched')}
              </span>
            )}
          </div>
          {genres.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-1.5">
              {genres.map((g) => (
                <Badge key={g}>{g}</Badge>
              ))}
            </div>
          )}
          {item.summary && <p className="mt-4 line-clamp-5 text-body leading-relaxed text-fg/90">{item.summary}</p>}
          {children}
        </div>
      </div>
    </section>
  )
}

function PlayButtons({ item }: { item: ItemCard }) {
  const navigate = useNavigate()
  const progress = progressOf(item)
  return (
    <div className="mt-6 flex flex-wrap gap-2">
      <Button size="lg" variant="primary" icon={<Play size={18} weight="fill" />} onClick={() => navigate(`/play/${item.id}`)}>
        {progress != null ? t('item.resume', { time: clock(item.position_s ?? 0) }) : t('player.play')}
      </Button>
      {progress != null && (
        <Button size="lg" icon={<ArrowCounterClockwise size={18} />} onClick={() => navigate(`/play/${item.id}?t=0`)}>
          {t('item.fromStart')}
        </Button>
      )}
    </div>
  )
}

function EpisodeRow({ ep }: { ep: ItemCard }) {
  const progress = progressOf(ep)
  return (
    <Link to={`/item/${ep.id}`} className="group flex gap-4 rounded-lg p-2 transition-colors hover:bg-raised">
      <div className="relative w-40 shrink-0 sm:w-48">
        <Artwork image={ep.backdrop} title={ep.title} aspect="wide" />
        {progress != null && (
          <div className="absolute inset-x-2 bottom-2 h-1 overflow-hidden rounded-full bg-black/50">
            <div className="h-full bg-primary" style={{ width: `${progress * 100}%` }} />
          </div>
        )}
        {ep.watched && <CheckCircle size={20} weight="fill" className="absolute end-1.5 top-1.5 text-primary" aria-label={t('item.watched')} />}
      </div>
      <div className="min-w-0 py-1">
        <p className="text-body font-semibold">
          <span className="text-muted">{ep.episode_no}. </span>
          {ep.title}
        </p>
        <p className="text-caption text-subtle">{runtime(ep.duration_s)}</p>
        {ep.summary && <p className="mt-1 line-clamp-2 text-label text-muted">{ep.summary}</p>}
      </div>
    </Link>
  )
}

function TechInfo({ detail }: { detail: ItemDetail }) {
  const file = detail.files[0]
  if (!file) return null
  const video = file.streams.find((s) => s.type === 'video')
  const audio = file.streams.filter((s) => s.type === 'audio')
  const subs = file.streams.filter((s) => s.type === 'subtitle')
  const rows: [string, string][] = [
    [t('item.file'), `${file.path.split(/[\\/]/).pop()} · ${fileSize(file.size)}`],
    ...(video ? [[t('item.video'), [video.codec?.toUpperCase(), video.height ? `${video.height}p` : null, video.profile].filter(Boolean).join(' · ')] as [string, string]] : []),
    ...(audio.length ? [[t('item.audio'), audio.map((a, i) => trackLabel(a, i + 1)).join(', ')] as [string, string]] : []),
    ...(subs.length ? [[t('item.subtitles'), subs.map((s, i) => trackLabel(s, i + 1)).join(', ')] as [string, string]] : []),
  ]
  return (
    <dl className="grid max-w-3xl grid-cols-[auto_1fr] gap-x-6 gap-y-2 text-label">
      {rows.map(([k, v]) => (
        <div key={k} className="contents">
          <dt className="text-subtle">{k}</dt>
          <dd className="min-w-0 text-muted [overflow-wrap:anywhere]">{v}</dd>
        </div>
      ))}
    </dl>
  )
}

export function ItemPage() {
  const id = Number(useParams().id)
  const { data, isLoading } = useQuery({ queryKey: ['item', id], queryFn: () => api.item(id) })
  if (isLoading || !data) {
    return (
      <div className="flex gap-6 pt-10">
        <Skeleton className="aspect-[2/3] w-52" />
        <div className="flex-1 space-y-3 pt-24">
          <Skeleton className="h-8 w-2/3" />
          <Skeleton className="h-4 w-1/3" />
        </div>
      </div>
    )
  }
  const { item, children } = data
  const playable = item.kind === 'movie' || item.kind === 'episode' || item.kind === 'video'
  return (
    <div className="anim-enter">
      <Hero detail={data}>{playable && data.files.length > 0 && <PlayButtons item={item} />}</Hero>
      {item.kind === 'show' && (
        <section>
          <h2 className="mb-4 text-title font-semibold">{t('item.seasons')}</h2>
          <PosterGrid items={children} />
        </section>
      )}
      {item.kind === 'season' && (
        <section className="max-w-4xl">
          <h2 className="mb-2 text-title font-semibold">{t('item.episodes')}</h2>
          <div className="flex flex-col gap-1">
            {children.map((ep) => (
              <EpisodeRow key={ep.id} ep={ep} />
            ))}
          </div>
        </section>
      )}
      {playable && <TechInfo detail={data} />}
    </div>
  )
}
