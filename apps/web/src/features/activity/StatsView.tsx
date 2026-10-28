import { Link } from 'react-router-dom'
import { Artwork } from '@/components/media/Artwork'
import { Card, CardHeader } from '@/components/ui/Card'
import { watchTime } from '@/lib/format'
import { t } from '@/lib/i18n'
import { fillDaily } from '@/lib/stats'
import type { WatchStats } from '@/lib/types'

const day = (d: number) => new Date(d * 86_400_000).toLocaleDateString(undefined, { month: 'short', day: 'numeric', timeZone: 'UTC' })
const when = (s: number) => new Date(s * 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <Card className="p-4">
      <p className="text-label text-muted">{label}</p>
      <p className="mt-1 text-page font-semibold tabular">{value}</p>
    </Card>
  )
}

/** Watch time per day over the last 30 days; plain bars on the theme colours. */
function DailyChart({ daily }: { daily: WatchStats['daily'] }) {
  const days = fillDaily(daily)
  const max = Math.max(...days.map((d) => d.seconds), 1)
  return (
    <div className="p-4">
      <div className="flex h-32 items-end gap-0.5" role="img" aria-label={t('stats.chartLabel', { total: watchTime(days.reduce((a, d) => a + d.seconds, 0)) })}>
        {days.map((d) => (
          <div key={d.day} className="flex h-full min-w-0 flex-1 items-end rounded-sm hover:bg-raised" title={`${day(d.day)}: ${watchTime(d.seconds)}`}>
            <div className="w-full rounded-sm bg-primary" style={{ height: d.seconds ? `${Math.max(3, (d.seconds / max) * 100)}%` : 0 }} />
          </div>
        ))}
      </div>
      <div className="mt-2 flex justify-between text-caption text-subtle">
        <span>{day(days[0].day)}</span>
        <span>{day(days[days.length - 1].day)}</span>
      </div>
    </div>
  )
}

/** Totals, daily chart, favourites and recent history for one profile. Used for the admin's per-user view and the profile page. */
export function StatsView({ stats }: { stats: WatchStats }) {
  const { totals } = stats
  const topGenre = stats.top_genres[0]?.seconds || 1
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Stat label={t('stats.total')} value={watchTime(totals.total_s)} />
        <Stat label={t('stats.last7')} value={watchTime(totals.last_7d_s)} />
        <Stat label={t('stats.last30')} value={watchTime(totals.last_30d_s)} />
        <Stat label={t('stats.completed')} value={totals.completed} />
      </div>
      <Card>
        <CardHeader title={t('stats.daily')} />
        <DailyChart daily={stats.daily} />
      </Card>
      {stats.top_titles.length > 0 && (
        <Card>
          <CardHeader title={t('stats.topTitles')} />
          <ul className="grid grid-cols-2 gap-4 p-4 sm:grid-cols-4">
            {stats.top_titles.map((x) => (
              <li key={x.item_id} className="min-w-0">
                <Link to={`/item/${x.item_id}`} className="block">
                  <Artwork image={x.poster} title={x.title} />
                  <p className="mt-2 truncate text-label font-medium" title={x.title}>
                    {x.title}
                  </p>
                </Link>
                <p className="text-caption tabular text-muted">{watchTime(x.seconds)}</p>
              </li>
            ))}
          </ul>
        </Card>
      )}
      {stats.top_genres.length > 0 && (
        <Card>
          <CardHeader title={t('stats.topGenres')} />
          <ul className="space-y-3 p-4">
            {stats.top_genres.map((g) => (
              <li key={g.genre} className="flex items-center gap-3 text-label">
                <span className="w-28 shrink-0 truncate" title={g.genre}>
                  {g.genre}
                </span>
                <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-raised" aria-hidden>
                  <div className="h-full rounded-full bg-primary" style={{ width: `${Math.max(2, (g.seconds / topGenre) * 100)}%` }} />
                </div>
                <span className="w-16 shrink-0 text-end tabular text-muted">{watchTime(g.seconds)}</span>
              </li>
            ))}
          </ul>
        </Card>
      )}
      <Card>
        <CardHeader title={t('stats.history')} />
        {stats.history.length === 0 ? (
          <p className="p-4 text-body text-muted">{t('stats.historyEmpty')}</p>
        ) : (
          <ul className="divide-y divide-border">
            {stats.history.map((h) => (
              <li key={h.item.id} className="flex items-center justify-between gap-3 px-4 py-2.5 text-body">
                <Link to={`/item/${h.item.id}`} className="min-w-0 truncate hover:underline">
                  {h.item.show_title ? `${h.item.show_title} · ${h.item.title}` : h.item.title}
                </Link>
                <span className="shrink-0 text-caption text-muted">{when(h.last_played_at)}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  )
}
