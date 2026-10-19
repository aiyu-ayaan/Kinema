import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { MagnifyingGlass } from '@phosphor-icons/react'
import { PosterGrid } from '@/components/media/PosterCard'
import { EmptyState, PageHeader } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'

export function SearchPage() {
  const q = useSearchParams()[0].get('q') ?? ''
  const { data, isLoading } = useQuery({ queryKey: ['search', q], queryFn: () => api.search(q), enabled: q.length > 0 })
  return (
    <div>
      <PageHeader title={t('search.title', { q })} />
      {!isLoading && data?.length === 0 ? (
        <EmptyState icon={<MagnifyingGlass size={40} weight="duotone" />} title={t('search.none.title')} message={t('search.none.message')} />
      ) : (
        <PosterGrid items={data ?? []} />
      )}
    </div>
  )
}
