import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { CheckCircle, Play } from '@phosphor-icons/react'
import { Artwork } from './Artwork'
import { progressOf, subtitleOf } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ItemCard } from '@/lib/types'

function ProgressBar({ value }: { value: number }) {
  return (
    <div className="absolute inset-x-2 bottom-2 h-1 overflow-hidden rounded-full bg-black/50" aria-hidden>
      <div className="h-full rounded-full bg-primary" style={{ width: `${value * 100}%` }} />
    </div>
  )
}

/** 2:3 poster for movies and shows; `wide` shows a 16:9 still (episodes, continue watching). */
export function PosterCard({ item, wide }: { item: ItemCard; wide?: boolean }) {
  const progress = progressOf(item)
  const playable = item.kind === 'movie' || item.kind === 'episode' || item.kind === 'video'
  // Continue Watching jumps straight back into the player.
  const to = wide && playable ? `/play/${item.id}` : `/item/${item.id}`
  const title = item.kind === 'episode' && wide ? (item.show_title ?? item.title) : item.title
  const sub = item.kind === 'episode' && wide ? `${subtitleOf(item)} · ${item.title}` : subtitleOf(item)
  return (
    <Link to={to} className="group block min-w-0 rounded-lg focus-visible:outline-offset-4" title={item.title}>
      <div className="relative">
        <Artwork
          image={wide ? (item.backdrop ?? item.poster) : item.poster}
          title={item.show_title && item.kind !== 'movie' ? item.show_title : item.title}
          aspect={wide ? 'wide' : 'poster'}
          className="ring-1 ring-inset ring-white/5 transition-transform duration-base ease-std group-hover:scale-[1.02]"
        />
        {playable && (
          <div className="absolute inset-0 flex items-center justify-center rounded-lg bg-black/0 opacity-0 transition-all duration-base group-hover:bg-black/35 group-hover:opacity-100">
            <span className="flex h-11 w-11 items-center justify-center rounded-full bg-primary text-on-primary shadow-overlay">
              <Play size={20} weight="fill" aria-label={t('player.play')} />
            </span>
          </div>
        )}
        {item.watched && (
          <CheckCircle size={22} weight="fill" className="absolute end-2 top-2 rounded-full bg-black/40 text-primary" aria-label={t('item.watched')} />
        )}
        {progress != null && <ProgressBar value={progress} />}
      </div>
      <p className="mt-2 truncate text-label font-medium text-fg">{title}</p>
      {sub && <p className="truncate text-caption text-muted">{sub}</p>}
    </Link>
  )
}

export function PosterGrid({ items, wide }: { items: ItemCard[]; wide?: boolean }) {
  return (
    <div className={wide ? 'grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-x-4 gap-y-6' : 'grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-x-4 gap-y-6'}>
      {items.map((i) => (
        <PosterCard key={i.id} item={i} wide={wide} />
      ))}
    </div>
  )
}

/** Horizontal scrolling row for the home screen. */
export function Shelf({ title, items, wide, action }: { title: string; items: ItemCard[]; wide?: boolean; action?: ReactNode }) {
  if (items.length === 0) return null
  return (
    <section className="anim-enter">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 className="text-title font-semibold">{title}</h2>
        {action}
      </div>
      <div className="scrollbar-none -mx-4 flex snap-x gap-4 overflow-x-auto px-4 pb-1 md:-mx-8 md:px-8">
        {items.map((i) => (
          <div key={i.id} className={wide ? 'w-64 shrink-0 snap-start' : 'w-36 shrink-0 snap-start md:w-40'}>
            <PosterCard item={i} wide={wide} />
          </div>
        ))}
      </div>
    </section>
  )
}
