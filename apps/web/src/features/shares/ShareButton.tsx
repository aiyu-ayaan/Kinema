import { useState, type FormEvent } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Check, Copy, ShareNetwork } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Select } from '@/components/ui/Controls'
import { Field } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { t, type MessageKey } from '@/lib/i18n'
import type { ItemCard } from '@/lib/types'

const HOURS = [24, 168, 720, 2160] as const
/** 0 = no limit. */
const VIEWERS = [0, 1, 2, 5, 10] as const

/** Admin: makes an expiring link to this title. The full link is shown once. */
export function ShareButton({ item }: { item: ItemCard }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [label, setLabel] = useState('')
  const [hours, setHours] = useState<number>(168)
  const [viewers, setViewers] = useState<number>(0)
  const [copied, setCopied] = useState(false)
  const create = useMutation({
    mutationFn: () => api.createShare({ item_id: item.id, expires_in_hours: hours, label: label.trim() || undefined, max_uses: viewers || undefined }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['shares'] }),
  })
  const link = create.data ? `${window.location.origin}${create.data.path}` : ''

  function close() {
    setOpen(false)
    setCopied(false)
    setLabel('')
    setViewers(0)
    create.reset()
  }

  function submit(e: FormEvent) {
    e.preventDefault()
    create.mutate()
  }

  return (
    <>
      <Button icon={<ShareNetwork size={16} />} onClick={() => setOpen(true)}>
        {t('share.button')}
      </Button>
      <Modal
        open={open}
        onClose={close}
        title={t('share.title')}
        footer={
          create.data ? (
            <Button variant="primary" onClick={close}>
              {t('common.close')}
            </Button>
          ) : (
            <>
              <Button variant="ghost" onClick={close}>
                {t('common.cancel')}
              </Button>
              <Button variant="primary" type="submit" form="share-form" loading={create.isPending}>
                {t('share.create')}
              </Button>
            </>
          )
        }
      >
        <p className="text-body text-muted">{t('share.description', { title: item.title })}</p>
        {create.data ? (
          <div className="mt-4 flex flex-col gap-2">
            <p className="text-label font-medium" role="status">
              {t('share.created')}
            </p>
            <div className="flex gap-2">
              <input readOnly value={link} aria-label={t('share.title')} onFocus={(e) => e.target.select()} className="min-w-0 flex-1 rounded-md border border-border bg-raised px-3 text-label [overflow-wrap:anywhere]" />
              <Button
                icon={copied ? <Check size={16} /> : <Copy size={16} />}
                onClick={() => navigator.clipboard?.writeText(link).then(() => setCopied(true))}
              >
                {copied ? t('share.copied') : t('common.copy')}
              </Button>
            </div>
          </div>
        ) : (
          <form id="share-form" onSubmit={submit} className="mt-4 flex flex-col gap-4">
            <Field label={t('share.label')} hint={t('share.labelHint')} maxLength={60} value={label} onChange={(e) => setLabel(e.target.value)} />
            <div className="flex flex-col gap-1.5">
              <label htmlFor="share-hours" className="text-label font-medium">
                {t('share.expires')}
              </label>
              <Select id="share-hours" value={hours} onChange={(e) => setHours(Number(e.target.value))}>
                {HOURS.map((h) => (
                  <option key={h} value={h}>
                    {t(`share.hours.${h}` as MessageKey)}
                  </option>
                ))}
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <label htmlFor="share-viewers" className="text-label font-medium">
                {t('share.viewers')}
              </label>
              <Select id="share-viewers" value={viewers} onChange={(e) => setViewers(Number(e.target.value))}>
                {VIEWERS.map((n) => (
                  <option key={n} value={n}>
                    {n === 0 ? t('share.viewers.any') : n === 1 ? t('share.viewers.one') : t('share.viewers.n', { count: n })}
                  </option>
                ))}
              </Select>
              <p className="text-caption text-muted">{t('share.viewersHint')}</p>
            </div>
            <ErrorText>{create.error?.message}</ErrorText>
          </form>
        )}
      </Modal>
    </>
  )
}
