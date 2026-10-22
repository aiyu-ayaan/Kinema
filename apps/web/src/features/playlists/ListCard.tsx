import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { Artwork } from '@/components/media/Artwork'
import { t } from '@/lib/i18n'

/** Cover card for a playlist or a collection: the first title's poster, a name and a count. */
export function ListCard({ to, title, count, poster, note }: { to: string; title: string; count: number; poster: string | null; note?: ReactNode }) {
  return (
    <Link to={to} className="group block min-w-0 focus-visible:outline-offset-4" title={title}>
      <Artwork image={poster} title={title} className="ring-1 ring-inset ring-white/5 transition-transform duration-base ease-std group-hover:scale-[1.02]" />
      <p className="mt-2 truncate text-label font-medium text-fg transition-colors group-hover:text-primary">{title}</p>
      <p className="truncate text-caption text-muted">{note ?? t(count === 1 ? 'lists.itemCount.one' : 'lists.itemCount', { n: count })}</p>
    </Link>
  )
}

/** Horizontal row of `ListCard`s on Home, sized like the poster shelves. */
export function ListRow({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="anim-enter">
      <h2 className="mb-3 text-title font-semibold">{title}</h2>
      <div className="scrollbar-none -mx-4 flex snap-x scroll-px-4 gap-4 overflow-x-auto px-4 pb-2 md:-mx-8 md:scroll-px-8 md:px-8">{children}</div>
    </section>
  )
}
