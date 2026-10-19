import { useState, type ReactNode } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowCounterClockwise, Check, CheckCircle, LinkSimple, Play, Star, User } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { PosterGrid } from '@/components/media/PosterCard'
import { Badge, Skeleton } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { api, frameUrl, imageUrl } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { FixMatch } from './FixMatch'
import { SeriesTracks } from './SeriesTracks'
import { clock, fileSize, progressOf, runtime, subtitleOf, trackLabel } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { CastMember, ItemCard, ItemDetail } from '@/lib/types'

function Hero({ detail, children }: { detail: ItemDetail; children?: ReactNode }) {
  const { item, genres, files } = detail
  // No TMDB backdrop: a blurred video frame still gives the page its colour.
  const [frameFailed, setFrameFailed] = useState(false)
  const backdrop = imageUrl(item.backdrop) ?? (frameFailed ? null : frameUrl(item.id))
  const blurred = !item.backdrop
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
          <img src={backdrop} alt="" onError={() => setFrameFailed(true)} className={blurred ? 'h-full w-full scale-110 object-cover opacity-40 blur-2xl' : 'h-full w-full object-cover opacity-40'} />
          <div className="absolute inset-0 bg-gradient-to-t from-bg via-bg/80 to-bg/30" />
          <div className="absolute inset-0 bg-gradient-to-r from-bg/90 to-transparent" />
        </div>
      )}
      <div className="relative flex flex-col gap-6 px-4 pb-6 pt-8 sm:flex-row sm:items-end md:px-8 md:pt-16">
        <Artwork image={item.kind === 'episode' ? (item.backdrop ?? item.poster) : item.poster} frame={frameUrl(item.id)} title={item.show_title ?? item.title} aspect={item.kind === 'episode' ? 'wide' : 'poster'} eager className={item.kind === 'episode' ? 'w-72 shrink-0 shadow-overlay' : 'w-40 shrink-0 shadow-overlay md:w-52'} />
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

/** `episode`: started from a show or season page, so the label names the episode. */
function PlayButtons({ item, episode }: { item: ItemCard; episode?: boolean }) {
  const navigate = useNavigate()
  const progress = progressOf(item)
  const ep = subtitleOf(item) ?? ''
  const label = episode
    ? t(progress != null ? 'item.resumeEpisode' : 'item.playEpisode', { ep })
    : progress != null
      ? t('item.resume', { time: clock(item.position_s ?? 0) })
      : t('player.play')
  return (
    <>
      <Button size="lg" variant="primary" icon={<Play size={18} weight="fill" />} onClick={() => navigate(`/play/${item.id}`)}>
        {label}
      </Button>
      {progress != null && (
        <Button size="lg" icon={<ArrowCounterClockwise size={18} />} onClick={() => navigate(`/play/${item.id}?t=0`)}>
          {t('item.fromStart')}
        </Button>
      )}
    </>
  )
}

/** Marks the item, or every episode of a show or season, watched or not. */
function WatchedButton({ id, watched }: { id: number; watched: boolean }) {
  const qc = useQueryClient()
  const set = useMutation({
    mutationFn: () => api.setWatched(id, !watched),
    onSuccess: () => {
      for (const key of ['item', 'library', 'home']) qc.invalidateQueries({ queryKey: [key] })
    },
  })
  const label = t(watched ? 'item.markUnwatched' : 'item.markWatched')
  return (
    <Button
      size="lg"
      loading={set.isPending}
      onClick={() => set.mutate()}
      aria-label={label}
      title={label}
      aria-pressed={watched}
      icon={watched ? <CheckCircle size={20} weight="fill" className="text-primary" /> : <Check size={20} />}
    />
  )
}

function CopyLinkButton() {
  const [copied, setCopied] = useState(false)
  const label = t(copied ? 'item.linkCopied' : 'item.copyLink')
  return (
    <Button
      size="lg"
      aria-label={label}
      title={label}
      icon={copied ? <Check size={20} /> : <LinkSimple size={20} />}
      onClick={() =>
        navigator.clipboard
          ?.writeText(window.location.href)
          .then(() => {
            setCopied(true)
            window.setTimeout(() => setCopied(false), 2000)
          })
          .catch(() => {})
      }
    />
  )
}

function EpisodeCard({ ep }: { ep: ItemCard }) {
  const progress = progressOf(ep)
  return (
    <div className="group flex min-w-0 flex-col rounded-lg">
      <Link
        to={`/play/${ep.id}`}
        className="relative block overflow-hidden rounded-lg focus-visible:outline-offset-4"
        title={t('player.play')}
      >
        <Artwork image={ep.backdrop} frame={frameUrl(ep.id)} title={ep.title} aspect="wide" className="ring-1 ring-inset ring-white/5 transition-transform duration-base ease-std group-hover:scale-[1.02]" />
        <span className="absolute start-2 top-2 rounded bg-black/70 px-1.5 py-0.5 text-caption font-semibold tabular text-white">
          {t('item.episodeShort', { s: ep.season_no ?? 0, e: ep.episode_no ?? 0 })}
        </span>
        {ep.watched && <CheckCircle size={22} weight="fill" className="absolute end-2 top-2 rounded-full bg-black/40 text-primary" aria-label={t('item.watched')} />}
        <div className="absolute inset-0 flex items-center justify-center rounded-lg bg-black/0 opacity-0 transition-all duration-base group-hover:bg-black/40 group-hover:opacity-100">
          <span className="flex h-12 w-12 items-center justify-center rounded-full bg-primary text-on-primary shadow-overlay transition-transform duration-base group-hover:scale-110">
            <Play size={22} weight="fill" aria-hidden />
          </span>
        </div>
        {progress != null && (
          <div className="pointer-events-none absolute inset-x-2 bottom-2 h-1 overflow-hidden rounded-full bg-black/50">
            <div className="h-full bg-primary" style={{ width: `${progress * 100}%` }} />
          </div>
        )}
      </Link>
      <Link to={`/item/${ep.id}`} className="mt-2 block min-w-0 group/text focus-visible:outline-offset-2">
        <div className="flex items-baseline justify-between gap-2">
          <p className="min-w-0 truncate text-body font-semibold text-fg transition-colors group-hover/text:text-primary">{ep.title}</p>
          <span className="shrink-0 text-caption tabular text-subtle">{runtime(ep.duration_s)}</span>
        </div>
        {ep.summary && <p className="mt-0.5 line-clamp-2 text-label text-muted">{ep.summary}</p>}
      </Link>
    </div>
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

function CastAvatar({ member }: { member: CastMember }) {
  const [failed, setFailed] = useState(false)
  const src = member.image && !failed ? imageUrl(member.image) : null
  const initials = member.name
    .split(' ')
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase())
    .join('')

  return (
    <div className="flex w-24 shrink-0 flex-col items-center text-center md:w-28" title={`${member.name} (${member.role})`}>
      <div className="relative flex h-20 w-20 items-center justify-center overflow-hidden rounded-full bg-white/10 ring-1 ring-white/10 md:h-24 md:w-24">
        {src ? (
          <img
            src={src}
            alt={member.name}
            loading="lazy"
            decoding="async"
            onError={() => setFailed(true)}
            className="h-full w-full object-cover"
          />
        ) : initials ? (
          <span className="select-none text-title font-semibold text-white/70">{initials}</span>
        ) : (
          <User size={32} weight="duotone" className="text-white/40" />
        )}
      </div>
      <p className="mt-2 w-full truncate text-label font-medium text-fg">{member.name}</p>
      <p className="w-full truncate text-caption text-muted">{member.role}</p>
    </div>
  )
}

function CastSection({ cast }: { cast: CastMember[] }) {
  if (!cast || cast.length === 0) return null
  return (
    <section>
      <h2 className="mb-4 text-title font-semibold">{t('item.cast')}</h2>
      <div className="scrollbar-none -mx-4 flex gap-4 overflow-x-auto px-4 pb-2 md:-mx-8 md:px-8">
        {cast.map((m) => (
          <CastAvatar key={`${m.id}-${m.role}`} member={m} />
        ))}
      </div>
    </section>
  )
}

export function ItemPage() {
  const id = Number(useParams().id)
  const admin = useIsAdmin()
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
  const { item, children, cast } = data
  const playable = item.kind === 'movie' || item.kind === 'episode' || item.kind === 'video'
  const canPlay = playable && data.files.length > 0
  const nextUp = data.next_up
  return (
    <div className="anim-enter space-y-8">
      <Hero detail={data}>
        <div className="mt-6 flex flex-wrap gap-2">
          {canPlay && <PlayButtons item={item} />}
          {nextUp && <PlayButtons item={nextUp} episode />}
          {(canPlay || nextUp) && <WatchedButton id={item.id} watched={canPlay ? item.watched : !!nextUp?.watched} />}
          <CopyLinkButton />
          {admin && data.tracks && <SeriesTracks showId={item.id} tracks={data.tracks} />}
          {admin && (item.kind === 'movie' || item.kind === 'show') && <FixMatch item={item} />}
        </div>
      </Hero>
      {item.kind === 'show' && (
        <section>
          <h2 className="mb-4 text-title font-semibold">{t('item.seasons')}</h2>
          <PosterGrid items={children} />
        </section>
      )}
      {item.kind === 'season' && (
        <section>
          <h2 className="mb-4 text-title font-semibold">{t('item.episodes')}</h2>
          <div className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-x-5 gap-y-7">
            {children.map((ep) => (
              <EpisodeCard key={ep.id} ep={ep} />
            ))}
          </div>
        </section>
      )}
      <CastSection cast={cast} />
      {playable && <TechInfo detail={data} />}
    </div>
  )
}
