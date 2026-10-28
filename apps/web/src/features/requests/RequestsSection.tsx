import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Check, Eye, FilmSlate, Trash, X } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Card, EmptyState, ErrorText, Skeleton } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import type { MediaRequest, RequestStatus } from '@/lib/types'
import { RequestRow } from './RequestRow'

const tabs: (RequestStatus | 'all')[] = ['pending', 'approved', 'available', 'declined', 'all']

function AdminRow({ r }: { r: MediaRequest }) {
  const qc = useQueryClient()
  const [note, setNote] = useState(r.admin_note ?? '')
  const refresh = () => qc.invalidateQueries({ queryKey: ['requests'] })
  const answer = useMutation({ mutationFn: (status: RequestStatus) => api.answerRequest(r.id, status, note.trim() || undefined), onSuccess: refresh })
  const remove = useMutation({ mutationFn: () => api.deleteRequest(r.id), onSuccess: refresh })
  const set = (status: RequestStatus, label: string, icon: JSX.Element, variant: 'primary' | 'secondary' | 'danger' = 'secondary') =>
    r.status !== status && (
      <Button size="sm" variant={variant} icon={icon} loading={answer.isPending && answer.variables === status} onClick={() => answer.mutate(status)}>
        {label}
      </Button>
    )
  return (
    <RequestRow r={r} showUser>
      <div className="mt-2 flex flex-wrap items-center gap-2">
        <input className={cn(inputCls, 'h-8 min-w-40 flex-1 text-label')} value={note} maxLength={500} onChange={(e) => setNote(e.target.value)} aria-label={t('requests.adminNoteLabel')} placeholder={t('requests.adminNoteLabel')} />
        {set('approved', t('requests.approve'), <Check size={16} />, r.status === 'pending' ? 'primary' : 'secondary')}
        {set('available', t('requests.markAvailable'), <Eye size={16} />)}
        {set('declined', t('requests.decline'), <X size={16} />, 'danger')}
        {r.status !== 'pending' && set('pending', t('requests.reopen'), <FilmSlate size={16} />)}
        <Button
          size="sm"
          variant="danger"
          icon={<Trash size={16} />}
          aria-label={t('common.remove')}
          title={t('common.remove')}
          loading={remove.isPending}
          onClick={() => window.confirm(t('requests.deleteConfirm', { title: r.title })) && remove.mutate()}
        />
      </div>
      <div className="mt-2">
        <ErrorText>{answer.error?.message || remove.error?.message}</ErrorText>
      </div>
    </RequestRow>
  )
}

/** Admin: every title request, filtered by status, with the answer buttons. */
export function RequestsSection() {
  const { data, isLoading } = useQuery({ queryKey: ['requests'], queryFn: api.requests })
  const [tab, setTab] = useState<RequestStatus | 'all'>('pending')
  const shown = data?.filter((r) => tab === 'all' || r.status === tab) ?? []
  const count = (s: RequestStatus | 'all') => data?.filter((r) => s === 'all' || r.status === s).length ?? 0
  return (
    <div className="space-y-4">
      <div role="tablist" className="flex flex-wrap gap-1">
        {tabs.map((s) => (
          <button
            key={s}
            type="button"
            role="tab"
            aria-selected={tab === s}
            onClick={() => setTab(s)}
            className={cn(
              'h-8 cursor-pointer rounded-md px-3 text-label font-medium transition-colors duration-fast',
              tab === s ? 'bg-primary-container text-on-primary-container' : 'text-muted hover:bg-raised hover:text-fg',
            )}
          >
            {t(s === 'all' ? 'requests.tab.all' : `requests.status.${s}`)} <span className="tabular opacity-70">{count(s)}</span>
          </button>
        ))}
      </div>
      {isLoading && <Skeleton className="h-32" />}
      {data && shown.length === 0 && <EmptyState icon={<FilmSlate size={28} />} title={t('requests.title')} message={t('requests.adminEmpty')} />}
      {shown.length > 0 && (
        <Card>
          <ul className="divide-y divide-border">
            {shown.map((r) => (
              <AdminRow key={r.id} r={r} />
            ))}
          </ul>
        </Card>
      )}
    </div>
  )
}
