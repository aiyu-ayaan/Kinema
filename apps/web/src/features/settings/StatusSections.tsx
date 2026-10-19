import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Copy, Trash } from '@phosphor-icons/react'
import { libraryIcon } from '@/components/media/libraryIcon'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, Notice, Skeleton } from '@/components/ui/Card'
import { SettingRow } from '@/components/ui/Controls'
import { api } from '@/lib/api'
import { fileSize } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ServerStatus } from '@/lib/types'

function useStatus() {
  // Polls while open so streams, scans and thumbnail progress move live.
  return useQuery({ queryKey: ['server-status'], queryFn: api.status, refetchInterval: 3000 })
}

function uptime(s: number) {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  return d > 0 ? t('settings.dashboard.uptimeDays', { d, h }) : t('settings.dashboard.uptimeHours', { h, m })
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <Card className="p-4">
      <p className="text-label text-muted">{label}</p>
      <p className="mt-1 text-page font-semibold tabular">{value}</p>
    </Card>
  )
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="grid grid-cols-[10rem_1fr] gap-3 px-4 py-2.5 text-label">
      <dt className="text-subtle">{k}</dt>
      <dd className="min-w-0 font-mono text-caption text-muted [overflow-wrap:anywhere]">{v}</dd>
    </div>
  )
}

function LibraryProgress({ lib }: { lib: ServerStatus['libraries'][number] }) {
  const I = libraryIcon[lib.kind]
  const pct = lib.files ? Math.round((lib.thumbnails / lib.files) * 100) : 0
  return (
    <li className="flex items-center gap-4 px-4 py-3">
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-primary-container text-on-primary-container">
        <I size={18} weight="fill" aria-hidden />
      </span>
      <div className="min-w-0 flex-1">
        <p className="flex flex-wrap items-center gap-2 text-body font-medium">
          {lib.name}
          {lib.scanning && <Badge tone="primary">{t('library.scanning')}</Badge>}
          {lib.generating_thumbnails && <Badge tone="info">{t('library.thumbnailsRunning')}</Badge>}
        </p>
        <div className="mt-1.5 flex items-center gap-3">
          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-raised" aria-hidden>
            <div className="h-full rounded-full bg-primary transition-all duration-slow" style={{ width: `${pct}%` }} />
          </div>
          <span className="shrink-0 text-caption tabular text-muted">{t('settings.dashboard.thumbs', { done: lib.thumbnails, total: lib.files })}</span>
        </div>
      </div>
      <span className="hidden text-label tabular text-muted sm:block">{t(lib.titles === 1 ? 'library.count.one' : 'library.count', { n: lib.titles })}</span>
    </li>
  )
}

export function DashboardSection() {
  const { data } = useStatus()
  if (!data) return <Skeleton className="h-96" />
  const busy = data.libraries.filter((l) => l.scanning || l.generating_thumbnails).length
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Stat label={t('settings.dashboard.movies')} value={data.stats.movies} />
        <Stat label={t('settings.dashboard.shows')} value={data.stats.shows} />
        <Stat label={t('settings.dashboard.episodes')} value={data.stats.episodes} />
        <Stat label={t('settings.dashboard.storage')} value={fileSize(data.stats.bytes)} />
        <Stat label={t('settings.dashboard.hours')} value={Math.round(data.stats.hours)} />
        <Stat label={t('settings.dashboard.streams')} value={data.active_streams} />
        <Stat label={t('settings.dashboard.transcodes')} value={data.active_transcodes} />
        <Stat label={t('settings.dashboard.tasks')} value={busy} />
      </div>
      <Card>
        <CardHeader title={t('settings.dashboard.libraries')} />
        <ul className="divide-y divide-border">
          {data.libraries.map((l) => (
            <LibraryProgress key={l.id} lib={l} />
          ))}
        </ul>
      </Card>
      <Card>
        <CardHeader title={t('settings.dashboard.server')} />
        <dl className="divide-y divide-border">
          <Row k={t('settings.dashboard.name')} v={data.name} />
          <Row k={t('settings.dashboard.version')} v={data.version} />
          <Row k={t('settings.dashboard.uptime')} v={uptime(data.uptime_s)} />
          <Row k="ffmpeg" v={data.ffmpeg ?? t('settings.dashboard.notFound')} />
          <Row k="ffprobe" v={data.ffprobe ?? t('settings.dashboard.notFound')} />
          <Row k={t('settings.dashboard.dataDir')} v={data.data_dir} />
        </dl>
      </Card>
    </div>
  )
}

export function NetworkSection() {
  const { data } = useStatus()
  if (!data) return <Skeleton className="h-48" />
  const urls = data.lan_urls.length ? data.lan_urls : [window.location.origin]
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader title={t('settings.nav.network')} />
        <div className="divide-y divide-border">
          <SettingRow label={t('settings.network.listen')} description={t('settings.network.listenHint')}>
            <code className="rounded bg-raised px-2 py-1 font-mono text-label">{data.listen}</code>
          </SettingRow>
          {urls.map((u) => (
            <SettingRow key={u} label={t('settings.network.url')} description={t('settings.network.urlHint')}>
              <div className="flex items-center gap-2">
                <code className="rounded bg-raised px-2 py-1 font-mono text-label">{u}</code>
                <Button size="sm" variant="ghost" aria-label={t('common.copy')} title={t('common.copy')} icon={<Copy size={16} />} onClick={() => navigator.clipboard?.writeText(u)} />
              </div>
            </SettingRow>
          ))}
        </div>
      </Card>
      {data.lan_urls.length === 0 && <Notice tone="warn">{t('settings.network.localOnly')}</Notice>}
    </div>
  )
}

const caches = ['thumbnails', 'subtitles', 'images'] as const

export function TroubleshootingSection() {
  const qc = useQueryClient()
  const { data } = useStatus()
  const clear = useMutation({
    mutationFn: api.clearCache,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['server-status'] }),
  })
  return (
    <Card>
      <CardHeader title={t('settings.troubleshooting.caches')} />
      <div className="divide-y divide-border">
        {caches.map((c) => (
          <SettingRow key={c} label={t(`settings.troubleshooting.${c}`)} description={t(`settings.troubleshooting.${c}Hint`)}>
            <div className="flex items-center gap-3">
              <span className="text-label tabular text-muted">{data ? fileSize(data.caches[c]) : '…'}</span>
              <Button size="sm" variant="danger" icon={<Trash size={14} />} loading={clear.isPending && clear.variables === c} onClick={() => clear.mutate(c)}>
                {t('settings.troubleshooting.clear')}
              </Button>
            </div>
          </SettingRow>
        ))}
      </div>
    </Card>
  )
}
