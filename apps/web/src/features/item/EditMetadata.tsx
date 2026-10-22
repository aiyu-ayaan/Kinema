import { useState, type FormEvent } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Lock, LockOpen, PencilSimple } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { t, type MessageKey } from '@/lib/i18n'
import { lockableFields, type ItemDetail, type LockableField } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))
const labels: Record<LockableField, MessageKey> = {
  title: 'edit.field.title',
  year: 'edit.field.year',
  summary: 'edit.field.summary',
  rating: 'edit.field.rating',
  poster: 'edit.field.poster',
  backdrop: 'edit.field.backdrop',
  genres: 'edit.field.genres',
}

function LockToggle({ field, locked, onChange }: { field: LockableField; locked: boolean; onChange: (v: boolean) => void }) {
  const label = t('edit.lock', { field: t(labels[field]).toLowerCase() })
  return (
    <button
      type="button"
      role="switch"
      aria-checked={locked}
      aria-label={label}
      title={label}
      onClick={() => onChange(!locked)}
      className={cn(
        'flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-md border transition-colors duration-fast',
        locked ? 'border-primary/40 bg-primary/10 text-primary' : 'border-border text-subtle hover:text-fg',
      )}
    >
      {locked ? <Lock size={16} weight="fill" /> : <LockOpen size={16} />}
    </button>
  )
}

/**
 * Admin: edit the text fields by hand and choose which fields stay locked. A locked field survives
 * rescans, TMDB refreshes and Fix match; saving a changed text field locks it.
 */
export function EditMetadata({ detail }: { detail: ItemDetail }) {
  const { item } = detail
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [text, setText] = useState({ title: '', year: '', summary: '', rating: '' })
  const [locked, setLocked] = useState<Set<LockableField>>(new Set())

  const start = () => {
    setText({ title: item.title, year: item.year ? String(item.year) : '', summary: item.summary ?? '', rating: item.rating ? item.rating.toFixed(1) : '' })
    setLocked(new Set(detail.locked_fields))
    setOpen(true)
  }
  const initial: Record<'title' | 'year' | 'summary' | 'rating', string> = {
    title: item.title,
    year: item.year ? String(item.year) : '',
    summary: item.summary ?? '',
    rating: item.rating ? item.rating.toFixed(1) : '',
  }
  const edit = (field: keyof typeof text, value: string) => {
    setText((s) => ({ ...s, [field]: value }))
    // Changing a field locks it; the lock stays until switched off.
    setLocked((s) => new Set(s).add(field))
  }

  const save = useMutation({
    mutationFn: async () => {
      const body: { title?: string; year?: number; summary?: string; rating?: number } = {}
      if (text.title !== initial.title) body.title = text.title
      if (text.year !== initial.year) body.year = Number(text.year) || 0
      if (text.summary !== initial.summary) body.summary = text.summary
      if (text.rating !== initial.rating) body.rating = Number(text.rating) || 0
      if (Object.keys(body).length > 0) await api.editMetadata(item.id, body)
      await api.setLocks(item.id, lockableFields.filter((f) => locked.has(f)))
    },
    onSuccess: () => {
      for (const key of ['item', 'library', 'home', 'search', 'playlist', 'collection']) qc.invalidateQueries({ queryKey: [key] })
      setOpen(false)
    },
  })
  function submit(e: FormEvent) {
    e.preventDefault()
    if (text.title.trim()) save.mutate()
  }
  const toggle = (field: LockableField) => (v: boolean) =>
    setLocked((s) => {
      const next = new Set(s)
      if (v) next.add(field)
      else next.delete(field)
      return next
    })
  const row = (field: 'title' | 'year' | 'rating', extra?: string) => (
    <div className="flex items-end gap-2">
      <div className="flex min-w-0 flex-1 flex-col gap-1.5">
        <label htmlFor={`edit-${field}`} className="text-label font-medium">
          {t(labels[field])}
        </label>
        <input id={`edit-${field}`} className={cn(inputCls, extra)} value={text[field]} inputMode={field === 'title' ? undefined : 'decimal'} onChange={(e) => edit(field, e.target.value)} />
      </div>
      <LockToggle field={field} locked={locked.has(field)} onChange={toggle(field)} />
    </div>
  )
  return (
    <>
      <Button size="lg" icon={<PencilSimple size={18} />} onClick={start}>
        {t('edit.button')}
      </Button>
      <Modal open={open} onClose={() => setOpen(false)} title={t('edit.title')}>
        <form onSubmit={submit} className="flex flex-col gap-4">
          {row('title')}
          <div className="grid grid-cols-2 gap-4">
            {row('year')}
            {row('rating')}
          </div>
          <div className="flex items-end gap-2">
            <div className="flex min-w-0 flex-1 flex-col gap-1.5">
              <label htmlFor="edit-summary" className="text-label font-medium">
                {t('edit.field.summary')}
              </label>
              <textarea id="edit-summary" rows={5} className={cn(inputCls, 'h-auto py-2')} value={text.summary} onChange={(e) => edit('summary', e.target.value)} />
            </div>
            <LockToggle field="summary" locked={locked.has('summary')} onChange={toggle('summary')} />
          </div>
          {(['poster', 'backdrop', 'genres'] as const).map((f) => (
            <div key={f} className="flex items-center justify-between gap-2">
              <span className="text-label font-medium">{t(labels[f])}</span>
              <LockToggle field={f} locked={locked.has(f)} onChange={toggle(f)} />
            </div>
          ))}
          <p className="text-caption text-subtle">{t('edit.lockHint')}</p>
          <ErrorText>{save.error && errorText(save.error)}</ErrorText>
          <div className="flex justify-end">
            <Button type="submit" variant="primary" loading={save.isPending} disabled={!text.title.trim()}>
              {t('edit.save')}
            </Button>
          </div>
        </form>
      </Modal>
    </>
  )
}
