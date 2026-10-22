import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Plus, Trash } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Card, CardHeader, ErrorText } from '@/components/ui/Card'
import { Select } from '@/components/ui/Controls'
import { inputCls } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { ShelfDef } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

function GenreSelect({ value, genres, onChange, label }: { value: string; genres: string[]; onChange: (g: string) => void; label: string }) {
  // A shelf may name a genre no title has (yet): keep it selectable.
  const options = value && !genres.includes(value) ? [value, ...genres] : genres
  return (
    <Select value={value} onChange={(e) => onChange(e.target.value)} aria-label={label}>
      <option value="">{t('shelves.pickGenre')}</option>
      {options.map((g) => (
        <option key={g} value={g}>
          {g}
        </option>
      ))}
    </Select>
  )
}

function ShelfRow({ shelf, genres, onChanged }: { shelf: ShelfDef; genres: string[]; onChanged: () => void }) {
  const [title, setTitle] = useState(shelf.title)
  const [genre, setGenre] = useState(shelf.genre)
  const save = useMutation({ mutationFn: () => api.updateShelf(shelf.id, { title: title.trim(), genre }), onSuccess: onChanged })
  const remove = useMutation({ mutationFn: () => api.deleteShelf(shelf.id), onSuccess: onChanged })
  const dirty = title.trim() !== shelf.title || genre !== shelf.genre
  return (
    <li className="flex flex-wrap items-center gap-2 px-4 py-3">
      <input className={`${inputCls} min-w-40 flex-1`} value={title} maxLength={80} onChange={(e) => setTitle(e.target.value)} aria-label={t('shelves.titleLabel')} />
      <GenreSelect value={genre} genres={genres} onChange={setGenre} label={t('shelves.genreLabel')} />
      <Button variant="primary" size="md" disabled={!dirty || !title.trim() || !genre} loading={save.isPending} onClick={() => save.mutate()}>
        {t('shelves.save')}
      </Button>
      <Button variant="ghost" icon={<Trash size={16} />} loading={remove.isPending} onClick={() => remove.mutate()} aria-label={t('shelves.delete')} title={t('shelves.delete')} />
      <ErrorText>{(save.error && errorText(save.error)) || (remove.error && errorText(remove.error))}</ErrorText>
    </li>
  )
}

/** Manage → Home shelves: titled genre rows on Home. */
export function ShelvesSection() {
  const qc = useQueryClient()
  const shelves = useQuery({ queryKey: ['shelves'], queryFn: api.shelves })
  const genres = useQuery({ queryKey: ['genres'], queryFn: api.genres })
  const [title, setTitle] = useState('')
  const [genre, setGenre] = useState('')
  const changed = () => {
    qc.invalidateQueries({ queryKey: ['shelves'] })
    qc.invalidateQueries({ queryKey: ['home'] })
  }
  const add = useMutation({
    mutationFn: () => api.createShelf(title.trim(), genre),
    onSuccess: () => {
      setTitle('')
      setGenre('')
      changed()
    },
  })
  function submit(e: FormEvent) {
    e.preventDefault()
    if (title.trim() && genre) add.mutate()
  }
  const names = genres.data ?? []
  return (
    <Card>
      <CardHeader title={t('settings.nav.shelves')} />
      <p className="px-4 pt-3 text-label text-muted">{t('shelves.intro')}</p>
      <ul className="divide-y divide-border">
        {shelves.data?.map((s) => (
          <ShelfRow key={`${s.id}-${s.title}-${s.genre}`} shelf={s} genres={names} onChanged={changed} />
        ))}
      </ul>
      {shelves.data?.length === 0 && <p className="px-4 py-3 text-label text-muted">{t('shelves.empty')}</p>}
      <form onSubmit={submit} className="flex flex-wrap items-center gap-2 border-t border-border px-4 py-3">
        <input className={`${inputCls} min-w-40 flex-1`} value={title} maxLength={80} onChange={(e) => setTitle(e.target.value)} placeholder={t('shelves.titleLabel')} aria-label={t('shelves.titleLabel')} />
        <GenreSelect value={genre} genres={names} onChange={setGenre} label={t('shelves.genreLabel')} />
        <Button type="submit" variant="primary" icon={<Plus size={16} />} loading={add.isPending} disabled={!title.trim() || !genre}>
          {t('shelves.add')}
        </Button>
        <ErrorText>{add.error && errorText(add.error)}</ErrorText>
      </form>
    </Card>
  )
}
