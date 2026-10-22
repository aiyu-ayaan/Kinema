import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { LinkBreak, ShareNetwork } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, EmptyState, ErrorText, Skeleton } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { Share } from '@/lib/types'

const date = (s: number) => new Date(s * 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })

function state(s: Share): 'live' | 'expired' | 'revoked' {
  if (s.revoked_at) return 'revoked'
  return s.expires_at * 1000 <= Date.now() ? 'expired' : 'live'
}

/** Admin: every share link, live ones first, with revoke. */
export function SharesSection() {
  const qc = useQueryClient()
  const { data, isLoading } = useQuery({ queryKey: ['shares'], queryFn: api.shares })
  const revoke = useMutation({
    mutationFn: api.revokeShare,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['shares'] }),
  })
  return (
    <Card>
      <CardHeader title={t('shares.title')} />
      {isLoading && <Skeleton className="m-4 h-24" />}
      {data?.length === 0 && <EmptyState icon={<ShareNetwork size={28} />} title={t('shares.title')} message={t('shares.empty')} />}
      <ul className="divide-y divide-border">
        {data?.map((s) => {
          const st = state(s)
          return (
            <li key={s.id} className="flex flex-wrap items-center gap-3 px-4 py-3">
              <div className="min-w-0 flex-1">
                <p className="flex flex-wrap items-center gap-2 text-body font-medium">
                  <Link to={`/item/${s.item_id}`} className="hover:underline">
                    {s.item_title ?? `#${s.item_id}`}
                  </Link>
                  {s.label && <span className="text-muted">· {s.label}</span>}
                  {st === 'revoked' && <Badge tone="danger">{t('shares.revoked')}</Badge>}
                  {st === 'expired' && <Badge tone="warn">{t('shares.expired')}</Badge>}
                </p>
                <p className="text-caption text-muted">
                  {t('shares.expires', { date: date(s.expires_at) })}
                  {s.creator_name && ` · ${t('shares.by', { name: s.creator_name })}`}
                </p>
              </div>
              {st === 'live' && (
                <Button
                  size="sm"
                  variant="danger"
                  icon={<LinkBreak size={16} />}
                  loading={revoke.isPending && revoke.variables === s.id}
                  onClick={() => window.confirm(t('shares.revokeConfirm', { title: s.item_title ?? '' })) && revoke.mutate(s.id)}
                >
                  {t('shares.revoke')}
                </Button>
              )}
            </li>
          )
        })}
      </ul>
      <div className="px-4 pb-3">
        <ErrorText>{revoke.error?.message}</ErrorText>
      </div>
    </Card>
  )
}
