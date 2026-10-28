import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, DesktopTower, DeviceMobile, Globe, Pulse } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { Avatar } from '@/components/ui/Avatar'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, EmptyState, ErrorText, Skeleton } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { clock, watchTime } from '@/lib/format'
import { t } from '@/lib/i18n'
import { episodeCode, PAUSED_AFTER_S } from '@/lib/stats'
import type { NowPlaying, User } from '@/lib/types'
import { StatsView } from './StatsView'

const kindIcon = { desktop: DesktopTower, mobile: DeviceMobile, web: Globe }

function NowPlayingRow({ n }: { n: NowPlaying }) {
  const { item } = n
  const title = item.kind === 'episode' ? `${item.show_title ?? item.title} · ${episodeCode(item.season_no, item.episode_no)}` : item.title
  const pct = item.duration_s ? Math.min(100, (n.position_s / item.duration_s) * 100) : 0
  const Device = kindIcon[n.device_kind as keyof typeof kindIcon] ?? Globe
  return (
    <li className="flex items-center gap-4 px-4 py-3">
      <Artwork image={item.poster} title={item.title} className="w-12 shrink-0" />
      <div className="min-w-0 flex-1">
        <p className="flex flex-wrap items-center gap-2 text-body font-medium">
          <span className="truncate">{title}</span>
          {n.idle_s > PAUSED_AFTER_S && <Badge tone="warn">{t('activity.paused')}</Badge>}
        </p>
        <p className="mt-0.5 flex flex-wrap items-center gap-x-2 text-caption text-muted">
          <span>{n.user_name}</span>
          <span className="inline-flex items-center gap-1">
            <Device size={14} aria-hidden />
            {n.device_name ?? t('activity.unknownDevice')}
          </span>
        </p>
        <div className="mt-2 flex items-center gap-3">
          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-raised" aria-hidden>
            <div className="h-full rounded-full bg-primary transition-all duration-slow" style={{ width: `${pct}%` }} />
          </div>
          <span className="shrink-0 text-caption tabular text-muted">
            {clock(n.position_s)}
            {item.duration_s ? ` / ${clock(item.duration_s)}` : ''}
          </span>
        </div>
      </div>
    </li>
  )
}

function NowPlayingCard() {
  const { data, isLoading } = useQuery({ queryKey: ['activity-now'], queryFn: api.activityNow, refetchInterval: 10_000 })
  return (
    <Card>
      <CardHeader title={t('activity.now')} />
      {isLoading && <Skeleton className="m-4 h-20" />}
      {data?.length === 0 && <p className="p-4 text-body text-muted">{t('activity.nowEmpty')}</p>}
      <ul className="divide-y divide-border">
        {data?.map((n) => (
          <NowPlayingRow key={`${n.user_id}-${n.item.id}`} n={n} />
        ))}
      </ul>
    </Card>
  )
}

function UserDetail({ user, onBack }: { user: User; onBack: () => void }) {
  const { data, error } = useQuery({ queryKey: ['activity-user', user.id], queryFn: () => api.activityUser(user.id) })
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Button variant="ghost" icon={<ArrowLeft size={16} />} onClick={onBack}>
          {t('common.back')}
        </Button>
        <Avatar name={user.name} color={user.avatar} size={32} />
        <h2 className="text-title font-semibold">{user.name}</h2>
      </div>
      <ErrorText>{error?.message}</ErrorText>
      {data ? <StatsView stats={data} /> : <Skeleton className="h-64" />}
    </div>
  )
}

/** Admin: who is watching now, and watch time per profile. */
export function ActivitySection() {
  const [selected, setSelected] = useState<User | null>(null)
  const { data, isLoading } = useQuery({ queryKey: ['activity-users'], queryFn: api.activityUsers })
  if (selected) return <UserDetail user={selected} onBack={() => setSelected(null)} />
  return (
    <div className="space-y-6">
      <NowPlayingCard />
      <Card>
        <CardHeader title={t('activity.users')} />
        {isLoading && <Skeleton className="m-4 h-24" />}
        {data?.length === 0 && <EmptyState icon={<Pulse size={28} />} title={t('activity.users')} message={t('activity.usersEmpty')} />}
        <ul className="divide-y divide-border">
          {data?.map(({ user, totals }) => (
            <li key={user.id}>
              <button type="button" onClick={() => setSelected(user)} className="flex w-full cursor-pointer flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3 text-start transition-colors hover:bg-raised">
                <span className="flex min-w-40 flex-1 items-center gap-3">
                  <Avatar name={user.name} color={user.avatar} size={32} />
                  <span className="min-w-0">
                    <span className="block truncate text-body font-medium">{user.name}</span>
                    <span className="block text-caption text-muted">
                      {totals.last_played_at ? t('activity.lastPlayed', { date: new Date(totals.last_played_at * 1000).toLocaleDateString() }) : t('activity.neverPlayed')}
                    </span>
                  </span>
                </span>
                <dl className="grid grid-cols-3 gap-x-5 gap-y-2 sm:grid-cols-5 text-label tabular">
                  {(
                    [
                      ['stats.total', watchTime(totals.total_s)],
                      ['stats.last7', watchTime(totals.last_7d_s)],
                      ['stats.last30', watchTime(totals.last_30d_s)],
                      ['stats.titles', totals.titles],
                      ['stats.completed', totals.completed],
                    ] as const
                  ).map(([k, v]) => (
                    <div key={k}>
                      <dt className="text-caption text-subtle">{t(k)}</dt>
                      <dd>{v}</dd>
                    </div>
                  ))}
                </dl>
              </button>
            </li>
          ))}
        </ul>
      </Card>
    </div>
  )
}
