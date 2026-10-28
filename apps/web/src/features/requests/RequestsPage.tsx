import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { FilmSlate, Plus, Trash } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Card, EmptyState, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { useAuth } from '@/lib/auth'
import { t } from '@/lib/i18n'
import { RequestDialog } from './RequestDialog'
import { RequestRow } from './RequestRow'

/** A profile's own title requests, and the way to make a new one. */
export function RequestsPage() {
  const qc = useQueryClient()
  const { user } = useAuth()
  const [creating, setCreating] = useState(false)
  const { data, isLoading } = useQuery({ queryKey: ['requests'], queryFn: api.requests })
  const withdraw = useMutation({ mutationFn: api.deleteRequest, onSuccess: () => qc.invalidateQueries({ queryKey: ['requests'] }) })
  // An admin's list holds everyone's requests; this page is only theirs.
  const mine = data?.filter((r) => r.user_id === user?.id)
  return (
    <div className="mx-auto max-w-3xl">
      <PageHeader
        title={t('requests.title')}
        description={t('requests.description')}
        actions={
          <Button variant="primary" icon={<Plus size={16} />} onClick={() => setCreating(true)}>
            {t('requests.new')}
          </Button>
        }
      />
      {isLoading && <Skeleton className="h-32" />}
      {mine?.length === 0 && <EmptyState icon={<FilmSlate size={28} />} title={t('requests.emptyTitle')} message={t('requests.empty')} />}
      {mine && mine.length > 0 && (
        <Card>
          <ul className="divide-y divide-border">
            {mine.map((r) => (
              <RequestRow key={r.id} r={r}>
                {r.status === 'pending' && (
                  <div className="mt-2">
                    <Button size="sm" variant="danger" icon={<Trash size={16} />} loading={withdraw.isPending && withdraw.variables === r.id} onClick={() => withdraw.mutate(r.id)}>
                      {t('requests.withdraw')}
                    </Button>
                  </div>
                )}
              </RequestRow>
            ))}
          </ul>
        </Card>
      )}
      <div className="mt-3">
        <ErrorText>{withdraw.error?.message}</ErrorText>
      </div>
      {creating && <RequestDialog onClose={() => setCreating(false)} />}
    </div>
  )
}
