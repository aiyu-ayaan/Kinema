import { useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
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
  const sorted = useMemo(() => (items ? [...items].sort(sorters[sort]) : []), [items, sort])

  return (
    <div>
      <PageHeader
        title={library?.name ?? ''}
        description={
          <span className="flex items-center gap-2">
            {t(items?.length === 1 ? 'library.count.one' : 'library.count', { n: items?.length ?? 0 })}
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
