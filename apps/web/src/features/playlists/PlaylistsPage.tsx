import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ListBullets } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { EmptyState, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import { ListCard } from './ListCard'

/** All playlists: the profile's own, then the ones other profiles shared. */
export function PlaylistsPage() {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [name, setName] = useState('')
  const { data, isLoading } = useQuery({ queryKey: ['playlists'], queryFn: api.playlists })
  const create = useMutation({
    mutationFn: () => api.createPlaylist(name.trim()),
    onSuccess: (p) => {
      qc.invalidateQueries({ queryKey: ['playlists'] })
      qc.invalidateQueries({ queryKey: ['home'] })
      navigate(`/playlists/${p.id}`)
    },
  })
  function submit(e: FormEvent) {
    e.preventDefault()
    if (name.trim()) create.mutate()
  }
  const mine = data?.filter((p) => p.mine) ?? []
  const shared = data?.filter((p) => !p.mine) ?? []
  const grid = 'grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-x-4 gap-y-6'
  return (
    <div className="anim-enter">
      <PageHeader title={t('playlist.title')} />
      <form onSubmit={submit} className="mb-8 flex max-w-md gap-2">
        <input className={inputCls} value={name} maxLength={80} onChange={(e) => setName(e.target.value)} placeholder={t('playlist.name')} aria-label={t('playlist.name')} />
        <Button type="submit" variant="primary" loading={create.isPending} disabled={!name.trim()}>
          {t('playlist.create')}
        </Button>
      </form>
      <ErrorText>{create.error instanceof Error && create.error.message}</ErrorText>
      {isLoading ? (
        <Skeleton className="aspect-[2/3] w-36" />
      ) : data?.length === 0 ? (
        <EmptyState icon={<ListBullets size={40} weight="duotone" />} title={t('playlist.empty.title')} message={t('playlist.empty.message')} />
      ) : (
        <div className="space-y-10">
          {mine.length > 0 && (
            <section>
              <h2 className="mb-3 text-title font-semibold">{t('playlist.mine')}</h2>
              <div className={grid}>
                {mine.map((p) => (
                  <ListCard key={p.id} to={`/playlists/${p.id}`} title={p.name} count={p.item_count} poster={p.poster} />
                ))}
              </div>
            </section>
          )}
          {shared.length > 0 && (
            <section>
              <h2 className="mb-3 text-title font-semibold">{t('playlist.shared')}</h2>
              <div className={grid}>
                {shared.map((p) => (
                  <ListCard key={p.id} to={`/playlists/${p.id}`} title={p.name} count={p.item_count} poster={p.poster} note={t('lists.sharedBy', { name: p.owner_name })} />
                ))}
              </div>
            </section>
          )}
        </div>
      )}
    </div>
  )
}
