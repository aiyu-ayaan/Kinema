import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Check, FolderPlus, ListPlus, Plus } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

interface Target {
  id: number
  name: string
}

/**
 * "Add to playlist" (any profile, playable items) or "Add to collection" (admin, movies and shows):
 * pick a list, or name a new one that starts with this item.
 */
export function AddToList({ itemId, kind, size = 'lg' }: { itemId: number; kind: 'playlist' | 'collection'; size?: 'md' | 'lg' }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [name, setName] = useState('')
  const [done, setDone] = useState<string | null>(null)
  const isPlaylist = kind === 'playlist'
  const lists = useQuery({
    queryKey: [kind === 'playlist' ? 'playlists' : 'collections'],
    queryFn: async (): Promise<Target[]> => (isPlaylist ? (await api.playlists()).filter((p) => p.mine) : await api.collections()),
    enabled: open,
  })
  const finish = (target: string) => {
    for (const key of ['playlists', 'playlist', 'collections', 'collection', 'home', 'item']) qc.invalidateQueries({ queryKey: [key] })
    setDone(target)
    setName('')
  }
  const add = useMutation({
    mutationFn: async (list: Target) => {
      if (isPlaylist) await api.addToPlaylist(list.id, [itemId])
      else {
        // A collection is replaced as a whole, so append to what it holds now.
        const members = (await api.collection(list.id)).items.map((i) => i.id)
        await api.setCollectionItems(list.id, members.includes(itemId) ? members : [...members, itemId])
      }
      return list.name
    },
    onSuccess: finish,
  })
  const create = useMutation({
    mutationFn: async (n: string) => (isPlaylist ? await api.createPlaylist(n, [itemId]) : await api.createCollection(n, [itemId])).name,
    onSuccess: finish,
  })

  function submit(e: FormEvent) {
    e.preventDefault()
    if (name.trim()) create.mutate(name.trim())
  }
  const busy = add.isPending || create.isPending
  const label = t(isPlaylist ? 'playlist.add' : 'collection.add')
  return (
    <>
      <Button
        size={size}
        aria-label={label}
        title={label}
        icon={isPlaylist ? <ListPlus size={20} /> : <FolderPlus size={20} />}
        onClick={() => {
          setDone(null)
          setOpen(true)
        }}
      />
      <Modal open={open} onClose={() => setOpen(false)} title={label}>
        <ul className="flex flex-col gap-1">
          {lists.data?.map((l) => (
            <li key={l.id}>
              <button
                type="button"
                disabled={busy}
                onClick={() => add.mutate(l)}
                className="flex w-full cursor-pointer items-center justify-between gap-3 rounded-md px-3 py-2 text-start text-body transition-colors hover:bg-raised disabled:opacity-50"
              >
                <span className="truncate">{l.name}</span>
                <Plus size={16} className="shrink-0 text-subtle" aria-hidden />
              </button>
            </li>
          ))}
        </ul>
        {lists.data?.length === 0 && <p className="py-3 text-label text-muted">{t(isPlaylist ? 'playlist.none' : 'collection.none')}</p>}
        <form onSubmit={submit} className="mt-4 flex gap-2">
          <input
            className={inputCls}
            value={name}
            maxLength={80}
            onChange={(e) => setName(e.target.value)}
            placeholder={t(isPlaylist ? 'playlist.name' : 'collection.name')}
            aria-label={t(isPlaylist ? 'playlist.name' : 'collection.name')}
          />
          <Button type="submit" variant="primary" loading={create.isPending} disabled={!name.trim()}>
            {t('playlist.create')}
          </Button>
        </form>
        <div className="mt-3 min-h-5" role="status">
          {done && (
            <p className="flex items-center gap-1.5 text-label text-primary">
              <Check size={14} aria-hidden />
              {t('playlist.added', { name: done })}
            </p>
          )}
          <ErrorText>{(add.error && errorText(add.error)) || (create.error && errorText(create.error)) || (lists.error && errorText(lists.error))}</ErrorText>
        </div>
      </Modal>
    </>
  )
}
