import { useMemo, useState } from 'react'
import { useParams, useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { ArrowsClockwise, FolderOpen } from '@phosphor-icons/react'
import { PosterGrid } from '@/components/media/PosterCard'
import { Badge, EmptyState, PageHeader, Skeleton } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { api } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import { useLibraries } from '@/lib/queries'
import type { ItemCard } from '@/lib/types'

type Sort = 'title' | 'added' | 'year'

const sorters: Record<Sort, (a: ItemCard, b: ItemCard) => number> = {
  // The server already orders by sort title (articles ignored).
  title: () => 0,
  added: (a, b) => b.added_at - a.added_at,
  year: (a, b) => (b.year ?? 0) - (a.year ?? 0),
}

export function LibraryPage() {
  const id = Number(useParams().id)
  const admin = useIsAdmin()
  const { data: libraries, refetch } = useLibraries()
  const library = libraries?.find((l) => l.id === id)
  const { data: items, isLoading } = useQuery({
    queryKey: ['library', id],
    queryFn: () => api.libraryItems(id),
    refetchInterval: library?.scanning ? 2500 : false,
  })
  const [sort, setSort] = useState<Sort>('title')
  // ?type=movie|show: the sidebar's segregated entries of a library that holds both.
  const raw = useSearchParams()[0].get('type')
  const mixed = Boolean(library && library.show_count > 0 && library.show_count < library.item_count)
  const type =
    raw === 'movie' || raw === 'show'
      ? raw
      : mixed
        ? (library?.kind === 'show' ? 'show' : 'movie')
        : null
  const sorted = useMemo(() => {
    const shown = (items ?? []).filter((i) => !type || (type === 'show' ? i.kind === 'show' : i.kind !== 'show'))
    return shown.sort(sorters[sort])
  }, [items, sort, type])

  const title = useMemo(() => {
    if (!library) return ''
    if (!type) return library.name
    const isMovieLib =
      library.name.toLowerCase() === 'movies' ||
      library.name.toLowerCase() === t('library.kind.movie').toLowerCase() ||
      library.kind === 'movie'
    const isShowLib =
      library.name.toLowerCase() === 'tv shows' ||
      library.name.toLowerCase() === t('library.kind.show').toLowerCase() ||
      library.kind === 'show'

    if (type === 'show') {
      return isMovieLib ? t('library.kind.show') : `${library.name} · ${t('library.kind.show')}`
    }
    if (type === 'movie') {
      return isShowLib ? t('library.kind.movie') : library.name
    }
    return library.name
  }, [library, type])

  return (
    <div>
      <PageHeader
        title={title}
        description={
          <span className="flex items-center gap-2">
            {t(sorted.length === 1 ? 'library.count.one' : 'library.count', { n: sorted.length })}
            {library?.scanning && <Badge tone="primary">{t('library.scanning')}</Badge>}
          </span>
        }
        actions={
          <>
            <div className="flex rounded-md border border-border bg-raised p-0.5" role="group" aria-label={t('library.sort')}>
              {(['title', 'added', 'year'] as Sort[]).map((s) => (
                <button
                  key={s}
                  type="button"
                  onClick={() => setSort(s)}
                  aria-pressed={sort === s}
                  className={cn('h-7 cursor-pointer rounded px-3 text-label font-medium transition-colors', sort === s ? 'bg-panel text-fg shadow-sm' : 'text-muted hover:text-fg')}
                >
                  {t(`library.sort.${s}`)}
                </button>
              ))}
            </div>
            {admin && (
              <Button
                size="sm"
                icon={<ArrowsClockwise size={16} />}
                loading={library?.scanning}
                onClick={async () => {
                  await api.scanLibrary(id)
                  refetch()
                }}
              >
                {t('library.scan')}
              </Button>
            )}
          </>
        }
      />
      {isLoading ? (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-4">
          {Array.from({ length: 12 }, (_, i) => (
            <Skeleton key={i} className="aspect-[2/3]" />
          ))}
        </div>
      ) : sorted.length === 0 ? (
        <EmptyState icon={<FolderOpen size={40} weight="duotone" />} title={t('library.empty.title')} message={t(library?.scanning ? 'library.empty.scanning' : 'library.empty.message')} />
      ) : (
        <PosterGrid items={sorted} wide={library?.kind === 'other'} />
      )}
    </div>
  )
}
