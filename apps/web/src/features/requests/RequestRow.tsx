import type { ReactNode } from 'react'
import { Artwork } from '@/components/media/Artwork'
import { Badge } from '@/components/ui/Card'
import { t } from '@/lib/i18n'
import type { MediaRequest, RequestStatus } from '@/lib/types'

const tones = { pending: 'warn', approved: 'info', declined: 'danger', available: 'primary' } as const

export function StatusChip({ status }: { status: RequestStatus }) {
  return <Badge tone={tones[status]}>{t(`requests.status.${status}`)}</Badge>
}

/** One request: poster, title, who asked and what was said. Actions come from the caller. */
export function RequestRow({ r, showUser, children }: { r: MediaRequest; showUser?: boolean; children?: ReactNode }) {
  return (
    <li className="flex gap-4 px-4 py-3">
      <Artwork image={r.poster} title={r.title} className="w-14 shrink-0" />
      <div className="min-w-0 flex-1">
        <p className="flex flex-wrap items-center gap-2 text-body font-medium">
          <span>
            {r.title}
            {r.year && <span className="ms-2 text-muted">{r.year}</span>}
          </span>
          <Badge>{t(r.kind === 'movie' ? 'requests.kind.movie' : 'requests.kind.show')}</Badge>
          <StatusChip status={r.status} />
        </p>
        <p className="text-caption text-muted">
          {showUser && `${r.user_name} · `}
          {new Date(r.created_at * 1000).toLocaleDateString()}
        </p>
        {r.overview && <p className="mt-1 line-clamp-2 text-label text-muted">{r.overview}</p>}
        {r.note && <p className="mt-1 text-label">{t('requests.noteFrom', { note: r.note })}</p>}
        {r.admin_note && <p className="mt-1 text-label text-primary">{t('requests.adminNote', { note: r.admin_note })}</p>}
        {children}
      </div>
    </li>
  )
}
