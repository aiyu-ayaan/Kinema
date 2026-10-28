import type { MessageKey } from '@/lib/i18n'
import type { ChannelKind, NotifyChannel } from '@/lib/types'

export const channelKinds: ChannelKind[] = ['ntfy', 'gotify', 'discord', 'slack', 'telegram', 'webhook', 'smtp']

export type ChannelField = 'url' | 'topic' | 'token' | 'to' | 'smtp_host' | 'smtp_port' | 'smtp_security' | 'smtp_username' | 'smtp_password' | 'from'

export interface FieldSpec {
  field: ChannelField
  label: MessageKey
  hint?: MessageKey
  /** Stored by the server and sent back masked. */
  secret?: boolean
}

const url = (label: MessageKey): FieldSpec => ({ field: 'url', label })

/** The inputs each kind of channel needs; the rest of `NotifyChannel` stays at its default. */
export const channelFields: Record<ChannelKind, FieldSpec[]> = {
  ntfy: [url('notify.field.server'), { field: 'topic', label: 'notify.field.topic' }, { field: 'token', label: 'notify.field.accessToken', hint: 'notify.hint.optional', secret: true }],
  gotify: [url('notify.field.server'), { field: 'token', label: 'notify.field.appToken', secret: true }],
  discord: [url('notify.field.webhookUrl')],
  slack: [url('notify.field.webhookUrl')],
  webhook: [url('notify.field.webhookUrl')],
  telegram: [
    { field: 'token', label: 'notify.field.botToken', secret: true },
    { field: 'to', label: 'notify.field.chatId' },
  ],
  smtp: [
    { field: 'smtp_host', label: 'notify.field.smtpHost' },
    { field: 'smtp_port', label: 'notify.field.smtpPort' },
    { field: 'smtp_security', label: 'notify.field.smtpSecurity' },
    { field: 'smtp_username', label: 'notify.field.smtpUsername' },
    { field: 'smtp_password', label: 'notify.field.smtpPassword', secret: true },
    { field: 'from', label: 'notify.field.from' },
    { field: 'to', label: 'notify.field.recipients', hint: 'notify.hint.recipients' },
  ],
}

export function newChannel(kind: ChannelKind): NotifyChannel {
  return {
    kind,
    label: '',
    enabled: true,
    url: kind === 'ntfy' ? 'https://ntfy.sh' : '',
    topic: '',
    token: '',
    to: '',
    smtp_host: '',
    smtp_port: 587,
    smtp_security: 'starttls',
    smtp_username: '',
    smtp_password: '',
    from: '',
  }
}
