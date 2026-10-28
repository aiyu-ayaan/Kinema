import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { PaperPlaneTilt, Plus, Trash } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Card, CardHeader, ErrorText, Skeleton } from '@/components/ui/Card'
import { SettingRow, Select, Switch } from '@/components/ui/Controls'
import { Field } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { ChannelKind, NotifyChannel, NotifySettings } from '@/lib/types'
import { channelFields, channelKinds, newChannel } from './notifyChannels'

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b)

function ChannelCard({
  channel: c,
  index,
  testable,
  onChange,
  onRemove,
}: {
  channel: NotifyChannel
  index: number
  /** Only a saved channel can be tested: the server sends with what it stored. */
  testable: boolean
  onChange: (c: NotifyChannel) => void
  onRemove: () => void
}) {
  const [result, setResult] = useState<string | null>(null)
  const test = useMutation({
    mutationFn: () => api.testNotification(index),
    onMutate: () => setResult(null),
    onSuccess: () => setResult(t('settings.notifications.testSent')),
  })
  const set = (patch: Partial<NotifyChannel>) => onChange({ ...c, ...patch })
  return (
    <Card>
      <CardHeader
        title={`${t(`notify.kind.${c.kind}`)}${c.label ? ` · ${c.label}` : ''}`}
        action={
          <div className="flex items-center gap-2">
            <Switch checked={c.enabled} onChange={(enabled) => set({ enabled })} label={t('settings.notifications.enabled')} />
            <Button size="sm" variant="danger" icon={<Trash size={14} />} onClick={onRemove} aria-label={t('common.remove')} title={t('common.remove')} />
          </div>
        }
      />
      <div className="grid gap-4 p-4 sm:grid-cols-2">
        <Field label={t('settings.notifications.label')} maxLength={60} value={c.label} onChange={(e) => set({ label: e.target.value })} />
        {channelFields[c.kind].map(({ field, label, hint, secret }) =>
          field === 'smtp_security' ? (
            <div key={field} className="flex flex-col gap-1.5">
              <label htmlFor={`sec-${index}`} className="text-label font-medium">
                {t(label)}
              </label>
              <Select id={`sec-${index}`} className="[&_select]:w-full" value={c.smtp_security} onChange={(e) => set({ smtp_security: e.target.value as NotifyChannel['smtp_security'] })}>
                <option value="tls">{t('notify.security.tls')}</option>
                <option value="starttls">{t('notify.security.starttls')}</option>
                <option value="none">{t('notify.security.none')}</option>
              </Select>
            </div>
          ) : (
            <Field
              key={field}
              label={t(label)}
              hint={hint && t(hint)}
              type={secret ? 'password' : field === 'smtp_port' ? 'number' : 'text'}
              autoComplete="off"
              value={c[field]}
              onChange={(e) => set(field === 'smtp_port' ? { smtp_port: Number(e.target.value) } : { [field]: e.target.value })}
            />
          ),
        )}
      </div>
      <div className="flex flex-wrap items-center gap-3 border-t border-border px-4 py-3">
        <Button size="sm" icon={<PaperPlaneTilt size={14} />} loading={test.isPending} disabled={!testable} onClick={() => test.mutate()}>
          {t('settings.notifications.test')}
        </Button>
        <span aria-live="polite" className="text-caption text-muted">
          {testable ? result : t('settings.notifications.saveFirst')}
        </span>
        <ErrorText>{test.error?.message}</ErrorText>
      </div>
    </Card>
  )
}

/** Admin: where to send a message when a device asks to be let in or a title is requested. */
export function NotificationsSection() {
  const qc = useQueryClient()
  const { data, isLoading } = useQuery({ queryKey: ['notifications'], queryFn: api.notifications })
  const [draft, setDraft] = useState<NotifySettings | null>(null)
  const [kind, setKind] = useState<ChannelKind>('ntfy')
  useEffect(() => {
    if (data) setDraft(data)
  }, [data])
  const save = useMutation({
    mutationFn: api.saveNotifications,
    onSuccess: (saved) => qc.setQueryData(['notifications'], saved),
  })
  if (isLoading || !data || !draft) return <Skeleton className="h-64" />

  const dirty = !same(draft, data)
  const edit = (patch: Partial<NotifySettings>) => setDraft({ ...draft, ...patch })
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader title={t('settings.notifications.events')} />
        <div className="divide-y divide-border">
          <SettingRow label={t('settings.notifications.onDevice')} description={t('settings.notifications.onDeviceHint')}>
            <Switch checked={draft.on_device_request} onChange={(v) => edit({ on_device_request: v })} label={t('settings.notifications.onDevice')} />
          </SettingRow>
          <SettingRow label={t('settings.notifications.onMedia')} description={t('settings.notifications.onMediaHint')}>
            <Switch checked={draft.on_media_request} onChange={(v) => edit({ on_media_request: v })} label={t('settings.notifications.onMedia')} />
          </SettingRow>
        </div>
      </Card>
      {draft.channels.length === 0 && <p className="text-body text-muted">{t('settings.notifications.empty')}</p>}
      {draft.channels.map((c, i) => (
        <ChannelCard
          // Channels have no id; the position is what the server's test route takes too.
          key={i}
          channel={c}
          index={i}
          testable={i < data.channels.length && same(c, data.channels[i])}
          onChange={(next) => edit({ channels: draft.channels.map((x, j) => (j === i ? next : x)) })}
          onRemove={() => edit({ channels: draft.channels.filter((_, j) => j !== i) })}
        />
      ))}
      <div className="flex flex-wrap items-center gap-2">
        <Select aria-label={t('settings.notifications.kind')} value={kind} onChange={(e) => setKind(e.target.value as ChannelKind)}>
          {channelKinds.map((k) => (
            <option key={k} value={k}>
              {t(`notify.kind.${k}`)}
            </option>
          ))}
        </Select>
        <Button icon={<Plus size={16} />} onClick={() => edit({ channels: [...draft.channels, newChannel(kind)] })}>
          {t('settings.notifications.add')}
        </Button>
        <Button variant="primary" className="ms-auto" loading={save.isPending} disabled={!dirty} onClick={() => save.mutate(draft)}>
          {t('common.save')}
        </Button>
      </div>
      <ErrorText>{save.error?.message}</ErrorText>
    </div>
  )
}
