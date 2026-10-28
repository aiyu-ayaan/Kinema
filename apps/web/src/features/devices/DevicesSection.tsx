import { useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Check, DesktopTower, DeviceMobile, Devices, Globe, Trash, X } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, EmptyState, ErrorText, Skeleton } from '@/components/ui/Card'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { Device } from '@/lib/types'
import { useDevices } from './usePendingDevices'

const kindIcon = { desktop: DesktopTower, mobile: DeviceMobile, web: Globe }
const date = (s: number) => new Date(s * 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })

function DeviceRow({ d, actions }: { d: Device; actions: ReactNode }) {
  const I = kindIcon[d.kind] ?? Globe
  return (
    <li className="flex flex-wrap items-center gap-3 px-4 py-3">
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-raised text-muted">
        <I size={18} aria-hidden />
      </span>
      <div className="min-w-0 flex-1">
        <p className="flex flex-wrap items-center gap-2 text-body font-medium">
          <span className="truncate">{d.name}</span>
          {d.status === 'pending' && <Badge tone="warn">{t('devices.pending')}</Badge>}
        </p>
        <p className="text-caption text-muted">
          {d.user_name}
          {' · '}
          {d.status === 'pending' ? t('devices.requested', { date: date(d.created_at) }) : t('devices.lastSeen', { date: date(d.last_seen) })}
        </p>
      </div>
      <div className="flex gap-2">{actions}</div>
    </li>
  )
}

/** Admin: devices waiting to be let in, and the ones profiles signed in from. */
export function DevicesSection() {
  const qc = useQueryClient()
  const { data, isLoading } = useDevices()
  const [removing, setRemoving] = useState<Device | null>(null)
  const refresh = () => qc.invalidateQueries({ queryKey: ['devices'] })
  const approve = useMutation({ mutationFn: api.approveDevice, onSuccess: refresh })
  const remove = useMutation({
    mutationFn: api.removeDevice,
    onSuccess: () => {
      setRemoving(null)
      refresh()
    },
  })
  const pending = data?.filter((d) => d.status === 'pending') ?? []
  const approved = data?.filter((d) => d.status === 'approved') ?? []

  return (
    <div className="space-y-6">
      <ErrorText>{approve.error?.message || (!removing && remove.error?.message)}</ErrorText>
      {isLoading && <Skeleton className="h-24" />}
      {data?.length === 0 && <EmptyState icon={<Devices size={28} />} title={t('devices.title')} message={t('devices.empty')} />}
      {pending.length > 0 && (
        <Card>
          <CardHeader title={t('devices.waiting')} />
          <ul className="divide-y divide-border">
            {pending.map((d) => (
              <DeviceRow
                key={d.id}
                d={d}
                actions={
                  <>
                    <Button size="sm" variant="primary" icon={<Check size={16} />} loading={approve.isPending && approve.variables === d.id} onClick={() => approve.mutate(d.id)}>
                      {t('devices.approve')}
                    </Button>
                    <Button size="sm" variant="danger" icon={<X size={16} />} loading={remove.isPending && remove.variables === d.id} onClick={() => remove.mutate(d.id)}>
                      {t('devices.decline')}
                    </Button>
                  </>
                }
              />
            ))}
          </ul>
        </Card>
      )}
      {approved.length > 0 && (
        <Card>
          <CardHeader title={t('devices.approved')} />
          <ul className="divide-y divide-border">
            {approved.map((d) => (
              <DeviceRow
                key={d.id}
                d={d}
                actions={
                  <Button size="sm" variant="danger" icon={<Trash size={16} />} onClick={() => setRemoving(d)}>
                    {t('common.remove')}
                  </Button>
                }
              />
            ))}
          </ul>
        </Card>
      )}
      <Modal
        open={!!removing}
        onClose={() => setRemoving(null)}
        title={t('devices.removeTitle')}
        footer={
          <>
            <Button variant="ghost" onClick={() => setRemoving(null)}>
              {t('common.cancel')}
            </Button>
            <Button variant="danger" loading={remove.isPending} onClick={() => removing && remove.mutate(removing.id)}>
              {t('common.remove')}
            </Button>
          </>
        }
      >
        <p className="text-body">{t('devices.removeConfirm', { device: removing?.name ?? '', user: removing?.user_name ?? '' })}</p>
        <div className="mt-3">
          <ErrorText>{remove.error?.message}</ErrorText>
        </div>
      </Modal>
    </div>
  )
}
