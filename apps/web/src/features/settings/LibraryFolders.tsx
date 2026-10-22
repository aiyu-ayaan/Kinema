import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Timer } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Switch } from '@/components/ui/Controls'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { Library, RootOption } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))
const MIN = 1
const MAX = 1440

/** Per-folder scan mode of a library: watched for filesystem events, or polled on a timer (network shares). */
export function LibraryFolders({ library }: { library: Library }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [roots, setRoots] = useState<RootOption[]>([])
  const save = useMutation({
    mutationFn: () => api.setRoots(library.id, roots),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['libraries'] })
      setOpen(false)
    },
  })
  const change = (path: string, patch: Partial<RootOption>) => setRoots((rs) => rs.map((r) => (r.path === path ? { ...r, ...patch } : r)))
  return (
    <>
      <Button
        size="sm"
        variant="ghost"
        aria-label={t('library.folders.button')}
        title={t('library.folders.button')}
        icon={<Timer size={16} />}
        onClick={() => {
          setRoots(library.root_options.map((r) => ({ ...r })))
          setOpen(true)
        }}
      />
      <Modal
        open={open}
        onClose={() => setOpen(false)}
        title={t('library.folders.title', { name: library.name })}
        footer={
          <Button variant="primary" loading={save.isPending} onClick={() => save.mutate()}>
            {t('library.folders.save')}
          </Button>
        }
      >
        <p className="mb-4 text-label text-muted">{t('library.folders.pollHint')}</p>
        <ul className="flex flex-col gap-4">
          {roots.map((r) => (
            <li key={r.path} className="rounded-md border border-border p-3">
              <p className="truncate font-mono text-caption text-subtle" title={r.path}>
                {r.path}
              </p>
              <div className="mt-2 flex items-center justify-between gap-3">
                <label htmlFor={`poll-${r.path}`} className="text-body font-medium">
                  {t('library.folders.poll')}
                </label>
                <Switch id={`poll-${r.path}`} checked={r.poll} onChange={(poll) => change(r.path, { poll })} />
              </div>
              {r.poll && (
                <div className="mt-3 flex items-center justify-between gap-3">
                  <label htmlFor={`interval-${r.path}`} className="text-label text-muted">
                    {t('library.folders.interval')}
                  </label>
                  <input
                    id={`interval-${r.path}`}
                    type="number"
                    min={MIN}
                    max={MAX}
                    className={`${inputCls} w-24 tabular`}
                    value={r.poll_interval_min}
                    onChange={(e) => change(r.path, { poll_interval_min: Math.min(MAX, Math.max(MIN, Math.round(Number(e.target.value)) || MIN)) })}
                  />
                </div>
              )}
            </li>
          ))}
        </ul>
        <ErrorText>{save.error && errorText(save.error)}</ErrorText>
      </Modal>
    </>
  )
}
