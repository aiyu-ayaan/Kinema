import { useState, type FormEvent } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowDown, ArrowUp, ListBullets, PencilSimple, Play, Shuffle, Trash, X } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { Button } from '@/components/ui/Button'
import { EmptyState, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { Switch } from '@/components/ui/Controls'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { activePlaylist } from '@/lib/activePlaylist'
import { api, frameUrl } from '@/lib/api'
import { runtime, subtitleOf } from '@/lib/format'
import { t } from '@/lib/i18n'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

function Rename({ id, name, onClose }: { id: number; name: string; onClose: () => void }) {
  const qc = useQueryClient()
  const [value, setValue] = useState(name)
  const save = useMutation({
    mutationFn: () => api.updatePlaylist(id, { name: value.trim() }),
    onSuccess: (d) => {
      qc.setQueryData(['playlist', id], d)
      qc.invalidateQueries({ queryKey: ['playlists'] })
      qc.invalidateQueries({ queryKey: ['home'] })
      onClose()
    },
  })
  function submit(e: FormEvent) {
    e.preventDefault()
    if (value.trim()) save.mutate()
  }
  return (
    <form onSubmit={submit} className="flex gap-2">
      <input className={inputCls} value={value} maxLength={80} autoFocus onChange={(e) => setValue(e.target.value)} aria-label={t('playlist.name')} />
      <Button type="submit" variant="primary" loading={save.isPending} disabled={!value.trim()}>
        {t('edit.save')}
      </Button>
      <div className="sr-only">{save.error && errorText(save.error)}</div>
    </form>
  )
}

/** One playlist: play it in order, and (for its owner) reorder, remove, rename, share and delete. */
export function PlaylistPage() {
  const id = Number(useParams().id)
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [renaming, setRenaming] = useState(false)
  const { data, isLoading, error } = useQuery({ queryKey: ['playlist', id], queryFn: () => api.playlist(id) })
  const refresh = () => {
    qc.invalidateQueries({ queryKey: ['playlists'] })
    qc.invalidateQueries({ queryKey: ['home'] })
  }
  const reorder = useMutation({
    mutationFn: (ids: number[]) => api.setPlaylistItems(id, ids),
    onSuccess: (d) => {
      qc.setQueryData(['playlist', id], d)
      refresh()
    },
  })
  const share = useMutation({
    mutationFn: (is_shared: boolean) => api.updatePlaylist(id, { is_shared }),
    onSuccess: (d) => {
      qc.setQueryData(['playlist', id], d)
      refresh()
    },
  })
  const remove = useMutation({
    mutationFn: () => api.deletePlaylist(id),
    onSuccess: () => {
      refresh()
      navigate('/playlists', { replace: true })
    },
  })

  if (isLoading) return <Skeleton className="h-64 w-full" />
  if (error || !data) return <EmptyState icon={<ListBullets size={40} weight="duotone" />} title={t('playlist.title')} message={errorText(error)} />

  const ids = data.items.map((i) => i.id)
  const move = (from: number, to: number) => {
    const next = [...ids]
    next.splice(to, 0, ...next.splice(from, 1))
    reorder.mutate(next)
  }
  const play = () => {
    if (ids.length === 0) return
    activePlaylist.set(id)
    navigate(`/play/${ids[0]}?t=0`)
  }
  const shuffle = () => {
    if (ids.length === 0) return
    const shuffled = [...ids]
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    activePlaylist.set(id, shuffled)
    navigate(`/play/${shuffled[0]}?t=0`)
  }
  return (
    <div className="anim-enter">
      <PageHeader
        title={data.name}
        description={data.mine ? t(data.item_count === 1 ? 'lists.itemCount.one' : 'lists.itemCount', { n: data.item_count }) : t('lists.sharedBy', { name: data.owner_name })}
        actions={
          <>
            <Button variant="primary" icon={<Play size={16} weight="fill" />} onClick={play} disabled={ids.length === 0}>
              {t('playlist.play')}
            </Button>
            <Button variant="secondary" icon={<Shuffle size={16} />} onClick={shuffle} disabled={ids.length === 0}>
              {t('playlist.shuffle')}
            </Button>
            {data.mine && (
              <>
                <Button icon={<PencilSimple size={16} />} onClick={() => setRenaming(true)} aria-label={t('playlist.rename')} title={t('playlist.rename')} />
                <Button
                  variant="danger"
                  icon={<Trash size={16} />}
                  aria-label={t('playlist.delete')}
                  title={t('playlist.delete')}
                  loading={remove.isPending}
                  onClick={() => window.confirm(t('playlist.deleteConfirm', { name: data.name })) && remove.mutate()}
                />
              </>
            )}
          </>
        }
      />
      {data.mine ? (
        <div className="mb-6 flex items-center justify-between gap-6 rounded-lg border border-border bg-panel px-4 py-3">
          <div className="min-w-0">
            <label htmlFor="share" className="text-body font-medium">
              {t('playlist.share')}
            </label>
            <p className="text-label text-muted">{t('playlist.shareHint')}</p>
          </div>
          <Switch id="share" checked={data.is_shared} onChange={(v) => share.mutate(v)} />
        </div>
      ) : (
        <p className="mb-6 text-label text-muted">{t('playlist.readOnly', { name: data.owner_name })}</p>
      )}
      <ErrorText>{(reorder.error && errorText(reorder.error)) || (share.error && errorText(share.error)) || (remove.error && errorText(remove.error))}</ErrorText>
      {data.items.length === 0 ? (
        <p className="py-10 text-center text-body text-muted">{t('playlist.emptyList')}</p>
      ) : (
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
              {data.mine && (
                <div className="flex shrink-0 gap-0.5">
                  <Button size="sm" variant="ghost" icon={<ArrowUp size={16} />} disabled={i === 0 || reorder.isPending} onClick={() => move(i, i - 1)} aria-label={t('playlist.moveUp')} title={t('playlist.moveUp')} />
                  <Button size="sm" variant="ghost" icon={<ArrowDown size={16} />} disabled={i === ids.length - 1 || reorder.isPending} onClick={() => move(i, i + 1)} aria-label={t('playlist.moveDown')} title={t('playlist.moveDown')} />
                  <Button size="sm" variant="ghost" icon={<X size={16} />} disabled={reorder.isPending} onClick={() => reorder.mutate(ids.filter((_, at) => at !== i))} aria-label={t('playlist.remove')} title={t('playlist.remove')} />
                </div>
              )}
            </li>
          ))}
        </ol>
      )}
      <Modal open={renaming} onClose={() => setRenaming(false)} title={t('playlist.rename')}>
        {renaming && <Rename id={id} name={data.name} onClose={() => setRenaming(false)} />}
      </Modal>
    </div>
  )
}
