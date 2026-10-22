import { useState, type FormEvent } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Folders, PencilSimple, Trash } from '@phosphor-icons/react'
import { PosterGrid } from '@/components/media/PosterCard'
import { Button } from '@/components/ui/Button'
import { EmptyState, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { useIsAdmin } from '@/lib/auth'
import { t } from '@/lib/i18n'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/** A franchise collection as a grid of its titles, oldest first. Admins rename and delete it. */
export function CollectionPage() {
  const id = Number(useParams().id)
  const admin = useIsAdmin()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [renaming, setRenaming] = useState(false)
  const [name, setName] = useState('')
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
  if (isLoading) return <Skeleton className="h-64 w-full" />
  if (error || !data) return <EmptyState icon={<Folders size={40} weight="duotone" />} title={t('home.collections')} message={errorText(error)} />
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
          admin && (
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
          )
        }
      />
      <ErrorText>{remove.error && errorText(remove.error)}</ErrorText>
      {data.items.length === 0 ? <p className="py-10 text-center text-body text-muted">{t('collection.empty')}</p> : <PosterGrid items={data.items} />}
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
