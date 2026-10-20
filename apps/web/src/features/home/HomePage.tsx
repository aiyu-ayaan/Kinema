import { useEffect, useMemo, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { FilmReel, Info, Play, Plus } from '@phosphor-icons/react'
import { Shelf } from '@/components/media/PosterCard'
import { Button } from '@/components/ui/Button'
import { EmptyState, Skeleton } from '@/components/ui/Card'
import { api, frameUrl, imageUrl, sharperImageUrl } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { clock, progressOf, runtime, subtitleOf } from '@/lib/format'
import { t } from '@/lib/i18n'
import { useScanning } from '@/lib/queries'
import type { Home, ItemCard } from '@/lib/types'

interface Feature {
  item: ItemCard
  reason: 'continue' | 'new'
}

/** Up to 5 hero slides: what you were watching, then the newest titles with art first. */
function pickFeatures(home: Home): Feature[] {
  const seen = new Set<number>()
  const out: Feature[] = []
  const add = (item: ItemCard, reason: Feature['reason']) => {
    if (out.length >= 5 || seen.has(item.id)) return
    seen.add(item.id)
    out.push({ item, reason })
  }
  home.continue_watching.slice(0, 2).forEach((i) => add(i, 'continue'))
  const fresh = home.shelves.filter((s) => s.kind !== 'episodes').flatMap((s) => s.items)
  fresh.sort((a, b) => Number(!!b.backdrop) - Number(!!a.backdrop) || b.added_at - a.added_at)
  fresh.forEach((i) => add(i, 'new'))
  return out
}

function Hero({ features }: { features: Feature[] }) {
  const navigate = useNavigate()
  const [index, setIndex] = useState(0)
  const [paused, setPaused] = useState(false)
  useEffect(() => {
    if (paused || features.length < 2) return
    const id = window.setInterval(() => setIndex((i) => (i + 1) % features.length), 8000)
    return () => window.clearInterval(id)
  }, [paused, features.length])
  const { item, reason } = features[Math.min(index, features.length - 1)]
  const playable = item.kind === 'movie' || item.kind === 'episode' || item.kind === 'video'
  const progress = progressOf(item)
  const title = item.kind === 'episode' ? (item.show_title ?? item.title) : item.title
  const facts = [item.kind === 'episode' ? `${subtitleOf(item)} · ${item.title}` : subtitleOf(item), runtime(item.duration_s)].filter(Boolean)

  return (
    <section
      className="relative -mx-4 -mt-6 mb-10 h-[min(64vh,560px)] min-h-[380px] overflow-hidden md:-mx-8 md:-mt-8"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      aria-roledescription="carousel"
    >
      {features.map((f, i) => {
        // Full width: the 1280 copy first (episode stills are stored at 300 px), then the stored art, then a frame.
        const sources = [sharperImageUrl(f.item.backdrop, 1280), imageUrl(f.item.backdrop), frameUrl(f.item.id)].filter((s): s is string => !!s)
        return (
          <img
            key={f.item.id}
            src={sources[0]}
            alt=""
            aria-hidden
            loading={i === 0 ? 'eager' : 'lazy'}
            onError={(e) => {
              const el = e.currentTarget
              const next = Number(el.dataset.source ?? 0) + 1
              el.dataset.source = String(next)
              if (sources[next]) el.src = sources[next]
              else el.style.visibility = 'hidden'
            }}
            className={cn('absolute inset-0 h-full w-full object-cover transition-opacity duration-700', i === index ? 'opacity-100' : 'opacity-0')}
          />
        )
      })}
      <div className="absolute inset-0 bg-gradient-to-t from-bg via-bg/40 to-bg/10" aria-hidden />
      <div className="absolute inset-0 bg-gradient-to-r from-bg/95 via-bg/50 to-transparent" aria-hidden />

      <div key={item.id} className="anim-enter absolute inset-x-0 bottom-0 max-w-2xl px-4 pb-10 md:px-8 md:pb-14">
        <p className="text-label font-semibold uppercase tracking-wider text-primary">{t(reason === 'continue' ? 'home.hero.continue' : 'home.hero.new')}</p>
        <h1 className="text-over-media mt-2 text-[clamp(28px,4.5vw,48px)] font-bold leading-[1.05] tracking-[-0.02em] text-white">{title}</h1>
        <p className="text-over-media mt-3 flex flex-wrap gap-x-3 text-body text-white/80">
          {facts.map((f) => (
            <span key={f}>{f}</span>
          ))}
        </p>
        {item.summary && <p className="text-over-media mt-3 line-clamp-3 max-w-xl text-body leading-relaxed text-white/85">{item.summary}</p>}
        {progress != null && (
          <div className="mt-4 h-1 w-64 overflow-hidden rounded-full bg-white/20" aria-hidden>
            <div className="h-full bg-primary" style={{ width: `${progress * 100}%` }} />
          </div>
        )}
        <div className="mt-6 flex flex-wrap gap-2">
          {playable && (
            <Button size="lg" variant="primary" icon={<Play size={18} weight="fill" />} onClick={() => navigate(`/play/${item.id}`)}>
              {progress != null ? t('item.resume', { time: clock(item.position_s ?? 0) }) : t('player.play')}
            </Button>
          )}
          <Button size="lg" className="border-white/20 bg-white/10 text-white backdrop-blur-md hover:border-white/40" icon={<Info size={18} />} onClick={() => navigate(`/item/${item.id}`)}>
            {t('home.hero.info')}
          </Button>
        </div>
      </div>

      {features.length > 1 && (
        <div className="absolute bottom-5 end-4 flex gap-1.5 md:end-8" role="tablist">
          {features.map((f, i) => (
            <button
              key={f.item.id}
              type="button"
              role="tab"
              aria-selected={i === index}
              aria-label={f.item.show_title ?? f.item.title}
              onClick={() => setIndex(i)}
              className={cn('h-1.5 cursor-pointer rounded-full transition-all duration-base', i === index ? 'w-6 bg-primary' : 'w-1.5 bg-white/40 hover:bg-white/70')}
            />
          ))}
        </div>
      )}
    </section>
  )
}

const shelfTitle = { movies: 'home.shelf.movies', shows: 'home.shelf.shows', episodes: 'home.shelf.episodes' } as const
const shelfType = { movies: 'movie', shows: 'show', episodes: 'show' } as const

export function HomePage() {
  const scanning = useScanning()
  const admin = useIsAdmin()
  const { data, isLoading } = useQuery({ queryKey: ['home'], queryFn: api.home, refetchInterval: scanning ? 3000 : false })
  const features = useMemo(() => (data ? pickFeatures(data) : []), [data])

  if (isLoading) {
    return (
      <div className="space-y-8">
        <Skeleton className="-mx-4 -mt-6 h-[min(64vh,560px)] rounded-none md:-mx-8 md:-mt-8" />
        <div className="flex gap-4 overflow-hidden">
          {Array.from({ length: 7 }, (_, i) => (
            <Skeleton key={i} className="aspect-[2/3] w-36 shrink-0 md:w-40" />
          ))}
        </div>
      </div>
    )
  }

  if (!data || (data.continue_watching.length === 0 && data.shelves.length === 0)) {
    return (
      <EmptyState
        icon={<FilmReel size={40} weight="duotone" />}
        title={t('home.empty.title')}
        message={t(scanning ? 'home.empty.scanning' : admin ? 'home.empty.admin' : 'home.empty.user')}
        action={
          admin &&
          !scanning && (
            <Link to="/settings/libraries">
              <Button variant="primary" icon={<Plus size={16} />}>
                {t('library.add')}
              </Button>
            </Link>
          )
        }
      />
    )
  }

  const multipleLibraries = new Set(data.shelves.map((s) => s.library.id)).size > 1
  return (
    <div>
      {features.length > 0 && <Hero features={features} />}
      <div className="space-y-10">
        <Shelf title={t('home.continue')} items={data.continue_watching} wide />
        {data.shelves.map((s) => (
          <Shelf
            key={`${s.library.id}-${s.kind}`}
            title={multipleLibraries ? `${t(shelfTitle[s.kind])} · ${s.library.name}` : t(shelfTitle[s.kind])}
            items={s.items}
            wide={s.kind === 'episodes'}
            action={
              <Link to={`/library/${s.library.id}?type=${shelfType[s.kind]}`} className="text-label font-medium text-primary hover:underline">
                {t('home.seeAll')}
              </Link>
            }
          />
        ))}
      </div>
    </div>
  )
}
