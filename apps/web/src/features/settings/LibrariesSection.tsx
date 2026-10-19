import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowsClockwise, ArrowUp, Folder, FolderPlus, ImagesSquare, Plus, Sparkle, Trash, X } from '@phosphor-icons/react'
import { libraryIcon } from '@/components/media/libraryIcon'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, EmptyState, ErrorText } from '@/components/ui/Card'
import { Field, inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import { useLibraries } from '@/lib/queries'
import type { LibraryKind } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/** Browses the server's folders; the add-library dialog needs paths on the server, not this device. */
function FolderPicker({ onPick }: { onPick: (path: string) => void }) {
  const [path, setPath] = useState('')
  const { data, error, isFetching } = useQuery({ queryKey: ['fs', path], queryFn: () => api.browse(path) })
  return (
    <div className="rounded-md border border-border">
      <div className="flex items-center gap-2 border-b border-border p-2">
        <Button size="sm" variant="ghost" disabled={data?.parent == null} onClick={() => data?.parent != null && setPath(data.parent)} aria-label={t('settings.folder.up')} icon={<ArrowUp size={16} />} />
        <p className="min-w-0 flex-1 truncate font-mono text-caption text-muted" title={data?.path}>
          {data?.path || t('settings.folder.roots')}
        </p>
        <Button size="sm" variant="primary" disabled={!data?.path || !data.parent} onClick={() => data?.path && onPick(data.path)}>
          {t('settings.folder.use')}
        </Button>
      </div>
      <ul className={cn('max-h-56 overflow-y-auto p-1', isFetching && 'opacity-60')}>
        {error && <li className="p-2 text-label text-danger">{errorText(error)}</li>}
        {data?.dirs.length === 0 && <li className="p-2 text-label text-subtle">{t('settings.folder.empty')}</li>}
        {data?.dirs.map((d) => (
          <li key={d.path}>
            <button type="button" onClick={() => setPath(d.path)} className="flex h-8 w-full cursor-pointer items-center gap-2 rounded px-2 text-start text-label hover:bg-raised">
              <Folder size={16} weight="fill" className="shrink-0 text-info" aria-hidden />
              <span className="truncate">{d.name}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}

function AddLibrary({ open, onClose }: { open: boolean; onClose: () => void }) {
  const qc = useQueryClient()
  const [name, setName] = useState('')
  const [kind, setKind] = useState<LibraryKind>('movie')
  const [paths, setPaths] = useState<string[]>([])
  const [typed, setTyped] = useState('')
  const [browsing, setBrowsing] = useState(false)
  const create = useMutation({
    mutationFn: () => api.createLibrary({ name: name.trim() || t(`library.kind.${kind}`), kind, paths }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['libraries'] })
      qc.invalidateQueries({ queryKey: ['home'] })
      setName('')
      setPaths([])
      onClose()
    },
  })
  const addPath = (p: string) => {
    const v = p.trim()
    if (v && !paths.includes(v)) setPaths([...paths, v])
    setBrowsing(false)
    setTyped('')
  }
  function submit(e: FormEvent) {
    e.preventDefault()
    if (paths.length) create.mutate()
  }
  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t('library.add')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" type="submit" form="add-library" loading={create.isPending} disabled={paths.length === 0}>
            {t('library.addSubmit')}
          </Button>
        </>
      }
    >
      <form id="add-library" onSubmit={submit} className="flex flex-col gap-5">
        <div className="flex flex-col gap-1.5">
          <span className="text-label font-medium">{t('library.type')}</span>
          <div className="grid grid-cols-3 gap-2">
            {(['movie', 'show', 'other'] as LibraryKind[]).map((k) => {
              const I = libraryIcon[k]
              return (
                <button
                  key={k}
                  type="button"
                  onClick={() => setKind(k)}
                  aria-pressed={kind === k}
                  className={cn(
                    'flex cursor-pointer flex-col items-center gap-1.5 rounded-md border p-3 text-label font-medium transition-colors',
                    kind === k ? 'border-primary bg-primary-container text-on-primary-container' : 'border-border text-muted hover:border-border-strong hover:text-fg',
                  )}
                >
                  <I size={22} weight={kind === k ? 'fill' : 'regular'} aria-hidden />
                  {t(`library.kind.${k}`)}
                </button>
              )
            })}
          </div>
        </div>
        <Field label={t('library.name')} placeholder={t(`library.kind.${kind}`)} value={name} onChange={(e) => setName(e.target.value)} />
        <div className="flex flex-col gap-2">
          <span className="text-label font-medium">{t('library.folders')}</span>
          {paths.map((p) => (
            <div key={p} className="flex items-center gap-2 rounded-md border border-border bg-raised px-3 py-1.5">
              <Folder size={16} weight="fill" className="shrink-0 text-info" aria-hidden />
              <span className="min-w-0 flex-1 truncate font-mono text-caption">{p}</span>
              <Button size="sm" variant="ghost" onClick={() => setPaths(paths.filter((x) => x !== p))} aria-label={t('common.remove')} icon={<X size={14} />} />
            </div>
          ))}
          {browsing ? (
            <FolderPicker onPick={addPath} />
          ) : (
            <div className="flex gap-2">
              <input
                className={cn(inputCls, 'font-mono text-label')}
                placeholder={t('library.pathPlaceholder')}
                value={typed}
                onChange={(e) => setTyped(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    addPath(typed)
                  }
                }}
              />
              {typed.trim() ? (
                <Button onClick={() => addPath(typed)} icon={<Plus size={16} />}>
                  {t('common.add')}
                </Button>
              ) : (
                <Button onClick={() => setBrowsing(true)} icon={<FolderPlus size={16} />}>
                  {t('library.browse')}
                </Button>
              )}
            </div>
          )}
          <p className="text-caption text-subtle">{t(`library.hint.${kind}`)}</p>
        </div>
        <ErrorText>{create.error && errorText(create.error)}</ErrorText>
      </form>
    </Modal>
  )
}

/** Manage → Libraries: add, rescan, re-match, regenerate thumbnails, remove. */
export function LibrariesSection() {
  const qc = useQueryClient()
  const { data: libraries, refetch } = useLibraries()
  const [adding, setAdding] = useState(false)
  const remove = useMutation({
    mutationFn: api.deleteLibrary,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['libraries'] })
      qc.invalidateQueries({ queryKey: ['home'] })
    },
  })
  return (
    <Card>
      <CardHeader
        title={t('settings.nav.libraries')}
        action={
          <Button size="sm" variant="primary" icon={<Plus size={16} />} onClick={() => setAdding(true)}>
            {t('library.add')}
          </Button>
        }
      />
      {libraries?.length === 0 ? (
        <div className="p-4">
          <EmptyState icon={<FolderPlus size={36} weight="duotone" />} title={t('settings.noLibraries.title')} message={t('settings.noLibraries.message')} />
        </div>
      ) : (
        <ul className="divide-y divide-border">
          {libraries?.map((l) => {
            const I = libraryIcon[l.kind]
            return (
              <li key={l.id} className="flex items-center gap-4 px-4 py-3">
                <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-primary-container text-on-primary-container">
                  <I size={20} weight="fill" aria-hidden />
                </span>
                <div className="min-w-0 flex-1">
                  <p className="flex items-center gap-2 text-body font-medium">
                    {l.name}
                    {l.scanning && <Badge tone="primary">{t('library.scanning')}</Badge>}
                    {l.generating_thumbnails && <Badge tone="info">{t('library.thumbnailsRunning')}</Badge>}
                  </p>
                  <p className="truncate font-mono text-caption text-subtle" title={l.roots.join('\n')}>
                    {l.roots.join(' · ')}
                  </p>
                </div>
                <span className="hidden text-label text-muted tabular sm:block">{t(l.item_count === 1 ? 'library.count.one' : 'library.count', { n: l.item_count })}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  loading={l.scanning}
                  onClick={async () => {
                    await api.scanLibrary(l.id)
                    refetch()
                  }}
                  aria-label={t('library.scan')}
                  title={t('library.scan')}
                  icon={<ArrowsClockwise size={16} />}
                />
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={async () => {
                    await api.matchMissing(l.id)
                    refetch()
                  }}
                  aria-label={t('library.matchMissing')}
                  title={t('library.matchMissing')}
                  icon={<Sparkle size={16} />}
                />
                <Button
                  size="sm"
                  variant="ghost"
                  loading={l.generating_thumbnails}
                  onClick={async () => {
                    await api.regenerateThumbnails(l.id)
                    refetch()
                  }}
                  aria-label={t('library.regenerateThumbnails')}
                  title={t('library.regenerateThumbnails')}
                  icon={<ImagesSquare size={16} />}
                />
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => window.confirm(t('library.deleteConfirm', { name: l.name })) && remove.mutate(l.id)}
                  aria-label={t('library.delete')}
                  title={t('library.delete')}
                  icon={<Trash size={16} />}
                />
              </li>
            )
          })}
        </ul>
      )}
      <AddLibrary open={adding} onClose={() => setAdding(false)} />
    </Card>
  )
}

