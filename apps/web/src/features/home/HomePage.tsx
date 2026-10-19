import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { FilmReel, Plus } from '@phosphor-icons/react'
import { Shelf } from '@/components/media/PosterCard'
import { Button } from '@/components/ui/Button'
import { EmptyState, Skeleton } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { t } from '@/lib/i18n'
import { useScanning } from '@/lib/queries'

export function HomePage() {
  const scanning = useScanning()
  const admin = useIsAdmin()
  const { data, isLoading } = useQuery({ queryKey: ['home'], queryFn: api.home, refetchInterval: scanning ? 3000 : false })

  if (isLoading) {
    return (
      <div className="space-y-8">
        {[0, 1].map((r) => (
          <div key={r} className="flex gap-4 overflow-hidden">
            {Array.from({ length: 7 }, (_, i) => (
              <Skeleton key={i} className="aspect-[2/3] w-36 shrink-0 md:w-40" />
            ))}
          </div>
        ))}
      </div>
    )
  }

  const empty = !data || (data.continue_watching.length === 0 && data.recently_added.every((s) => s.items.length === 0))
  if (empty) {
    return (
      <EmptyState
        icon={<FilmReel size={40} weight="duotone" />}
        title={t('home.empty.title')}
        message={t(scanning ? 'home.empty.scanning' : admin ? 'home.empty.admin' : 'home.empty.user')}
        action={
          admin &&
          !scanning && (
            <Link to="/settings">
              <Button variant="primary" icon={<Plus size={16} />}>
                {t('library.add')}
              </Button>
            </Link>
          )
        }
      />
    )
  }

  return (
    <div className="space-y-10">
      <Shelf title={t('home.continue')} items={data.continue_watching} wide />
      {data.recently_added.map(({ library, items }) => (
        <Shelf
          key={library.id}
          title={t('home.recentlyAdded', { library: library.name })}
          items={items}
          action={
            <Link to={`/library/${library.id}`} className="text-label font-medium text-primary hover:underline">
              {t('home.seeAll')}
            </Link>
          }
        />
      ))}
    </div>
  )
}
