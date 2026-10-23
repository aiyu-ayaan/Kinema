import { useState, type FormEvent } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Folders, ListBullets, PencilSimple, SquaresFour, Trash, X } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { PosterCard } from '@/components/media/PosterCard'
import { Button } from '@/components/ui/Button'
import { EmptyState, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api, frameUrl } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { runtime, subtitleOf } from '@/lib/format'
import { t } from '@/lib/i18n'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/** A franchise collection: view as rows or poster grid. Admins remove items, rename, and delete it. */
export function CollectionPage() {
  const id = Number(useParams().id)
  const admin = useIsAdmin()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [renaming, setRenaming] = useState(false)
  const [name, setName] = useState('')
  const [view, setView] = useState<'list' | 'grid'>('list')
  const { data, isLoading, error } = useQuery({ queryKey: ['collection', id], queryFn: () => api.collection(id) })
  const refresh = () => {
    for (const key of ['collections', 'home', 'item']) qc.invalidateQueries({ queryKey: [key] })
  }
  const rename = useMutation({
    mutationFn: () => api.renameCollection(id, name.trim()),
    onSuccess: (d) => {
      qc.setQueryData(['collection', id], d)
      refresh()
      setRenaming(false)
    },
  })
  const remove = useMutation({
    mutationFn: () => api.deleteCollection(id),
    onSuccess: () => {
      refresh()
      navigate('/', { replace: true })
    },
  })
  const removeItem = useMutation({
    mutationFn: (remainingItemIds: number[]) => api.setCollectionItems(id, remainingItemIds),
    onSuccess: (d) => {
      qc.setQueryData(['collection', id], d)
      refresh()
    },
  })

  if (isLoading) return <Skeleton className="h-64 w-full" />
  if (error || !data) return <EmptyState icon={<Folders size={40} weight="duotone" />} title={t('home.collections')} message={errorText(error)} />

  const ids = data.items.map((i) => i.id)

  function submit(e: FormEvent) {
    e.preventDefault()
    if (name.trim()) rename.mutate()
  }

  return (
    <div className="anim-enter">
      <PageHeader
        title={data.name}
        description={t(data.item_count === 1 ? 'lists.itemCount.one' : 'lists.itemCount', { n: data.item_count })}
        actions={
          <div className="flex items-center gap-2">
            <div className="flex rounded-md border border-border bg-panel p-0.5">
              <Button
                size="sm"
                variant={view === 'list' ? 'secondary' : 'ghost'}
                icon={<ListBullets size={16} />}
                aria-label={t('collection.viewList')}
                title={t('collection.viewList')}
                onClick={() => setView('list')}
              />
              <Button
                size="sm"
                variant={view === 'grid' ? 'secondary' : 'ghost'}
                icon={<SquaresFour size={16} />}
                aria-label={t('collection.viewGrid')}
                title={t('collection.viewGrid')}
                onClick={() => setView('grid')}
              />
            </div>
            {admin && (
              <>
                <Button
                  icon={<PencilSimple size={16} />}
                  aria-label={t('collection.rename')}
                  title={t('collection.rename')}
                  onClick={() => {
                    setName(data.name)
                    setRenaming(true)
                  }}
                />
                <Button
                  variant="danger"
                  icon={<Trash size={16} />}
                  aria-label={t('collection.delete')}
                  title={t('collection.delete')}
                  loading={remove.isPending}
                  onClick={() => window.confirm(t('collection.deleteConfirm', { name: data.name })) && remove.mutate()}
                />
              </>
            )}
          </div>
        }
      />
      <ErrorText>{(remove.error && errorText(remove.error)) || (removeItem.error && errorText(removeItem.error))}</ErrorText>
      {data.items.length === 0 ? (
        <p className="py-10 text-center text-body text-muted">{t('collection.empty')}</p>
      ) : view === 'list' ? (
        <ol className="divide-y divide-border rounded-lg border border-border bg-panel">
          {data.items.map((item, i) => (
            <li key={`${item.id}-${i}`} className="flex items-center gap-3 px-3 py-2">
              <span className="w-6 shrink-0 text-center text-label tabular text-subtle">{i + 1}</span>
              <Link to={`/item/${item.id}`} className="flex min-w-0 flex-1 items-center gap-3">
                <Artwork image={item.backdrop ?? item.poster} frame={frameUrl(item.id)} title={item.show_title ?? item.title} aspect="wide" className="w-24 shrink-0 rounded-md" />
                <span className="min-w-0">
                  <span className="block truncate text-body font-medium">{item.kind === 'episode' ? (item.show_title ?? item.title) : item.title}</span>
                  <span className="block truncate text-caption text-muted">
                    {[item.kind === 'episode' ? `${subtitleOf(item)} · ${item.title}` : subtitleOf(item), runtime(item.duration_s)].filter(Boolean).join(' · ')}
                  </span>
                </span>
              </Link>
              {admin && (
                <div className="flex shrink-0">
                  <Button
                    size="sm"
                    variant="ghost"
                    icon={<X size={16} />}
                    disabled={removeItem.isPending}
                    onClick={() => removeItem.mutate(ids.filter((_, at) => at !== i))}
                    aria-label={t('collection.remove')}
                    title={t('collection.remove')}
                  />
                </div>
              )}
            </li>
          ))}
        </ol>
      ) : (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-x-4 gap-y-6">
          {data.items.map((item, i) => (
            <div key={item.id} className="relative group">
              <PosterCard item={item} />
              {admin && (
                <button
                  type="button"
                  disabled={removeItem.isPending}
                  onClick={(e) => {
                    e.preventDefault()
                    e.stopPropagation()
                    removeItem.mutate(ids.filter((_, at) => at !== i))
                  }}
                  className="absolute end-2 top-2 z-10 flex h-8 w-8 items-center justify-center rounded-full bg-black/60 text-white shadow-overlay backdrop-blur-sm opacity-80 transition-colors hover:bg-danger hover:text-white md:opacity-0 md:group-hover:opacity-100"
                  aria-label={t('collection.remove')}
                  title={t('collection.remove')}
                >
                  <X size={16} />
                </button>
              )}
            </div>
          ))}
        </div>
      )}
      <Modal open={renaming} onClose={() => setRenaming(false)} title={t('collection.rename')}>
        <form onSubmit={submit} className="flex gap-2">
          <input className={inputCls} value={name} maxLength={80} autoFocus onChange={(e) => setName(e.target.value)} aria-label={t('collection.name')} />
          <Button type="submit" variant="primary" loading={rename.isPending} disabled={!name.trim()}>
            {t('edit.save')}
          </Button>
        </form>
        <ErrorText>{rename.error && errorText(rename.error)}</ErrorText>
      </Modal>
    </div>
  )
}
