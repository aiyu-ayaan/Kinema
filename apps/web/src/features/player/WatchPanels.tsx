import { useEffect, useMemo, useRef, useState, type Ref } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, CircleNotch, Info, Play, SkipForward, Star, Subtitles, Waveform, X } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { EpisodeLinks } from '@/components/media/EpisodeLinks'
import { api, frameUrl } from '@/lib/api'
import { cn } from '@/lib/cn'
import { clock, progressOf, runtime, subtitleOf } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ItemCard, ItemDetail } from '@/lib/types'

/** How long before the end the next episode is offered. */
const UP_NEXT_S = 60

/** Opens another title in the player; replaces this one so Back still leads out of the player. */
function usePlayNext() {
  const navigate = useNavigate()
  return (id: number) => navigate(`/play/${id}?t=0`, { replace: true })
}

/**
 * The next episode, offered only in the last minute of this one (YouTube's end card). It slides in from the end edge
 * and out again; the shade over its still draws back as the countdown runs out, and the video ending plays it.
 */
export function UpNextCard({ nextId, remaining, lifted, compact }: { nextId: number | null; remaining: number | null; lifted: boolean; compact: boolean }) {
  const playNext = usePlayNext()
  const [hidden, setHidden] = useState(false)
  // A small video has no room for the card and the controls at once: it waits for them to fade.
  const visible = nextId != null && remaining != null && remaining >= 0 && remaining <= UP_NEXT_S && !hidden && !(compact && lifted)
  // Fetched once the card is due, so a title watched only part way costs no extra request.
  const { data } = useQuery({ queryKey: ['item', nextId], queryFn: () => api.item(nextId as number), enabled: visible })
  if (nextId == null) return null
  const next = data?.item
  const shade = Math.min(1, Math.max(0, (remaining ?? 0) / UP_NEXT_S))
  return (
    <div
      onClick={(e) => e.stopPropagation()}
      aria-hidden={!visible}
      className={cn(
        'absolute end-4 z-10 flex w-[min(380px,calc(100%-2rem))] items-center gap-3 rounded-lg bg-black/80 p-2.5 text-white shadow-overlay backdrop-blur-md transition-[opacity,transform,bottom] duration-500 ease-std',
        lifted ? 'bottom-28' : 'bottom-6',
        visible ? 'translate-x-0 scale-100 opacity-100' : 'pointer-events-none translate-x-12 scale-95 opacity-0',
      )}
    >
      <button type="button" tabIndex={visible ? 0 : -1} onClick={() => playNext(nextId)} className="relative w-32 shrink-0 cursor-pointer overflow-hidden rounded-md" aria-label={t('player.playNow')}>
        <Artwork image={next?.backdrop ?? next?.poster ?? null} frame={frameUrl(nextId)} title={next?.title ?? ''} aspect="wide" className="rounded-md" />
        <span className="absolute inset-y-0 end-0 bg-black/50 transition-[width] duration-1000 ease-linear" style={{ width: `${shade * 100}%` }} />
        <Play size={28} weight="fill" className="absolute inset-0 m-auto" aria-hidden />
      </button>
      <div className="min-w-0 flex-1">
        <p className="text-caption font-semibold text-primary">{t('player.upNextIn', { time: clock(remaining ?? 0) })}</p>
        <p className="line-clamp-2 text-label font-semibold">{next ? [subtitleOf(next), next.title].filter(Boolean).join(' · ') : t('player.next')}</p>
        <button
          type="button"
          tabIndex={visible ? 0 : -1}
          onClick={() => playNext(nextId)}
          className="mt-2 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-full bg-primary px-3 text-label font-semibold text-on-primary hover:bg-primary/90"
        >
          <SkipForward size={14} weight="fill" aria-hidden />
          {t('player.playNow')}
        </button>
      </div>
      <button
        type="button"
        tabIndex={visible ? 0 : -1}
        onClick={() => setHidden(true)}
        aria-label={t('player.hide')}
        title={t('player.hide')}
        className="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center self-start rounded-full text-white/70 hover:bg-white/15 hover:text-white"
      >
        <X size={16} />
      </button>
    </div>
  )
}

/** Title, links and story under the video, with Details and the track picker as pills. */
export function WatchDetails({ detail, onTracks }: { detail: ItemDetail; onTracks: (() => void) | null }) {
  const { item } = detail
  const [open, setOpen] = useState(false)
  const meta = [item.year ? String(item.year) : null, runtime(item.duration_s ?? detail.files[0]?.duration_s)].filter(Boolean).join(' · ')
  const pill = 'inline-flex h-9 shrink-0 cursor-pointer items-center gap-2 rounded-full bg-raised px-4 text-label font-medium text-fg transition-colors hover:bg-border'
  return (
    <div className="flex flex-col gap-3 px-4 pt-4 lg:px-0">
      <div>
        {item.kind === 'episode' && item.show_id != null && (
          <Link to={`/item/${item.show_id}`} className="text-label font-semibold text-primary hover:underline">
            {item.show_title}
          </Link>
        )}
        <h1 className="text-page font-semibold">{item.title}</h1>
        <p className="mt-0.5 flex flex-wrap items-center gap-2 text-label text-muted">
          {item.kind === 'episode' && item.episode_no != null && (
            <span className="text-primary">
              <EpisodeLinks item={item} className="underline-offset-2 hover:underline" />
            </span>
          )}
          {item.rating != null && item.rating > 0 && (
            <span className="inline-flex items-center gap-1 text-warn">
              <Star size={14} weight="fill" aria-hidden />
              {item.rating.toFixed(1)}
            </span>
          )}
          {meta && <span>{meta}</span>}
        </p>
      </div>
      <div className="flex gap-2 overflow-x-auto">
        <Link to={`/item/${item.id}`} className={pill}>
          <Info size={18} aria-hidden />
          {t('watch.details')}
        </Link>
        {onTracks && (
          <button type="button" onClick={onTracks} className={pill}>
            <Subtitles size={18} aria-hidden />
            {t('player.tracks')}
          </button>
        )}
      </div>
      {(item.summary || detail.genres.length > 0) && (
        <div className="rounded-lg bg-raised p-3">
          {item.summary && <p className={cn('whitespace-pre-line text-body text-muted', !open && 'line-clamp-3')}>{item.summary}</p>}
          {detail.genres.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1.5">
              {detail.genres.map((g) => (
                <span key={g} className="rounded-full border border-border px-2.5 py-0.5 text-caption text-muted">
                  {g}
                </span>
              ))}
            </div>
          )}
          {item.summary && item.summary.length > 160 && (
            <button type="button" onClick={() => setOpen((o) => !o)} className="mt-1 cursor-pointer text-label font-semibold text-fg hover:underline">
              {open ? t('watch.less') : t('watch.more')}
            </button>
          )}
        </div>
      )}
    </div>
  )
}

/**
 * What to watch after this: the season's episodes with season chips for a show (the playing one marked), else more
 * titles of the same kind from the library, the ones in progress and unwatched first.
 */
export function UpNextList({ detail, scrollToCurrent }: { detail: ItemDetail; scrollToCurrent: boolean }) {
  const { item } = detail
  const playNext = usePlayNext()
  const episode = item.kind === 'episode'
  const [season, setSeason] = useState(item.parent_id)
  const seasons = useQuery({
    queryKey: ['item', item.show_id],
    queryFn: () => api.item(item.show_id as number),
    enabled: episode && item.show_id != null,
    select: (d) => d.children.filter((c) => c.kind === 'season'),
  })
  const episodes = useQuery({
    queryKey: ['item', season],
    queryFn: () => api.item(season as number),
    enabled: episode && season != null,
    select: (d) => d.children,
  })
  const library = useQuery({ queryKey: ['library-items', item.library_id], queryFn: () => api.libraryItems(item.library_id), enabled: !episode })
  const more = useMemo(
    () =>
      (library.data ?? [])
        .filter((c) => c.id !== item.id && c.kind === item.kind)
        .sort((a, b) => Number(progressOf(b) != null) - Number(progressOf(a) != null) || Number(a.watched) - Number(b.watched) || (b.rating ?? 0) - (a.rating ?? 0))
        .slice(0, 40),
    [library.data, item.id, item.kind],
  )
  const list = episode ? episodes.data : library.data && more
  const loading = episode ? episodes.isPending : library.isPending

  // Beside the video the list opens on the playing episode.
  const current = useRef<HTMLLIElement>(null)
  useEffect(() => {
    if (scrollToCurrent) current.current?.scrollIntoView({ block: 'nearest' })
  }, [scrollToCurrent, list])

  return (
    <section aria-labelledby="up-next-title" className="flex flex-col">
      <div className="sticky top-0 z-[1] flex flex-col gap-2 bg-bg px-4 pb-2 pt-3 lg:px-0 lg:pt-0">
        <h2 id="up-next-title" className="text-title font-semibold">
          {t(episode ? 'watch.episodes' : 'watch.moreToWatch')}
        </h2>
        {(seasons.data?.length ?? 0) > 1 && (
          <div className="flex gap-2 overflow-x-auto">
            {seasons.data?.map((s) => (
              <button
                key={s.id}
                type="button"
                aria-pressed={s.id === season}
                onClick={() => setSeason(s.id)}
                className={cn(
                  'h-8 shrink-0 cursor-pointer rounded-full px-3 text-label font-medium transition-colors',
                  s.id === season ? 'bg-fg text-bg' : 'bg-raised text-fg hover:bg-border',
                )}
              >
                {s.season_no != null ? t('item.seasonNo', { s: s.season_no }) : s.title}
              </button>
            ))}
          </div>
        )}
      </div>
      {loading ? (
        <CircleNotch size={28} className="m-6 animate-spin self-center text-primary" aria-label={t('common.loading')} />
      ) : list && list.length > 0 ? (
        <ul className="flex flex-col gap-1 px-2 pb-6 lg:px-0">
          {list.map((card) => (
            <UpNextRow key={card.id} card={card} current={card.id === item.id} rowRef={card.id === item.id ? current : undefined} onPlay={() => playNext(card.id)} />
          ))}
        </ul>
      ) : (
        <p className="px-4 py-3 text-body text-muted lg:px-0">{t('watch.nothing')}</p>
      )}
    </section>
  )
}

/** A YouTube-style row: 16:9 still with its length and progress, title, then season and episode or year and rating. */
function UpNextRow({ card, current, rowRef, onPlay }: { card: ItemCard; current: boolean; rowRef?: Ref<HTMLLIElement>; onPlay: () => void }) {
  const p = progressOf(card)
  const length = runtime(card.duration_s)
  const meta = [subtitleOf(card), card.kind !== 'episode' && card.rating ? `★ ${card.rating.toFixed(1)}` : null].filter(Boolean).join(' · ')
  return (
    <li ref={rowRef}>
      <button
        type="button"
        disabled={current}
        onClick={onPlay}
        aria-current={current || undefined}
        className={cn(
          'flex w-full cursor-pointer gap-3 rounded-lg p-2 text-start transition-colors disabled:cursor-default',
          current ? 'bg-primary-container text-on-primary-container' : 'hover:bg-raised',
        )}
      >
        <div className="relative w-40 shrink-0">
          <Artwork image={card.backdrop ?? card.poster} frame={frameUrl(card.id)} title={card.title} aspect="wide" className="rounded-md" />
          {current ? (
            <span className="absolute inset-0 flex items-center justify-center rounded-md bg-black/55 text-white">
              <Waveform size={24} aria-hidden />
            </span>
          ) : (
            length && <span className="absolute bottom-1 end-1 rounded bg-black/80 px-1 text-caption font-medium text-white">{length}</span>
          )}
          {p != null && (
            <span className="absolute inset-x-0 bottom-0 h-1 overflow-hidden rounded-b-md bg-white/30">
              <span className="block h-full bg-primary" style={{ width: `${p * 100}%` }} />
            </span>
          )}
        </div>
        <div className="min-w-0 flex-1 py-0.5">
          {current && <p className="text-caption font-semibold text-primary">{t('watch.nowPlaying')}</p>}
          <p className="line-clamp-2 text-label font-semibold">{card.title}</p>
          {meta && <p className="mt-0.5 truncate text-caption opacity-70">{meta}</p>}
          {card.watched && (
            <p className="mt-0.5 inline-flex items-center gap-1 text-caption opacity-70">
              <CheckCircle size={12} weight="fill" aria-hidden />
              {t('item.watched')}
            </p>
          )}
        </div>
      </button>
    </li>
  )
}
