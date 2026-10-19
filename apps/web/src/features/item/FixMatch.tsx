import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { MagnifyingGlass, Sparkle } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import type { ItemCard } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/** Admin search on TMDB to replace a wrong or missing match. */
export function FixMatch({ item }: { item: ItemCard }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [title, setTitle] = useState(item.title)
  const [year, setYear] = useState(item.year ? String(item.year) : '')
  const [query, setQuery] = useState<{ q: string; year: number | null } | null>(null)
  const results = useQuery({
    queryKey: ['match', item.id, query],
    queryFn: () => api.matchSearch(item.id, query?.q ?? '', query?.year ?? null),
    enabled: !!query,
  })
  const apply = useMutation({
    mutationFn: (tmdbId: number) => api.matchApply(item.id, tmdbId),
    onSuccess: () => {
      for (const key of ['item', 'library', 'home', 'libraries', 'search']) qc.invalidateQueries({ queryKey: [key] })
      setOpen(false)
    },
  })

  function search(e?: FormEvent) {
    e?.preventDefault()
    const y = Number(year)
    setQuery({ q: title.trim(), year: year && Number.isFinite(y) ? y : null })
  }

  return (
    <>
      <Button
        size="lg"
        icon={<Sparkle size={18} />}
        onClick={() => {
          setOpen(true)
          search()
        }}
      >
        {t('match.fix')}
      </Button>
      <Modal open={open} onClose={() => setOpen(false)} title={t(item.kind === 'show' ? 'match.titleShow' : 'match.titleMovie')}>
        <form onSubmit={search} className="flex gap-2">
          <input className={inputCls} value={title} onChange={(e) => setTitle(e.target.value)} aria-label={t('match.query')} placeholder={t('match.query')} />
          <input className={cn(inputCls, 'w-24 tabular')} value={year} onChange={(e) => setYear(e.target.value.replace(/\D/g, '').slice(0, 4))} inputMode="numeric" aria-label={t('match.year')} placeholder={t('match.year')} />
          <Button type="submit" variant="primary" loading={results.isFetching} icon={<MagnifyingGlass size={16} />} aria-label={t('match.search')} />
        </form>
        <div className="mt-4 flex flex-col gap-1">
          <ErrorText>{(results.error && errorText(results.error)) || (apply.error && errorText(apply.error))}</ErrorText>
          {results.data?.length === 0 && <p className="py-6 text-center text-body text-muted">{t('match.none')}</p>}
          {results.data?.map((c) => (
            <button
              key={c.tmdb_id}
              type="button"
              disabled={apply.isPending}
              onClick={() => apply.mutate(c.tmdb_id)}
              className="flex cursor-pointer gap-3 rounded-md p-2 text-start transition-colors hover:bg-raised disabled:opacity-50"
            >
              <Artwork image={c.poster ? `w300${c.poster}` : null} title={c.title} className="w-14 shrink-0" />
              <div className="min-w-0">
                <p className="text-body font-medium">
                  {c.title}
                  {c.year && <span className="ms-2 text-muted">{c.year}</span>}
                </p>
                {c.overview && <p className="mt-0.5 line-clamp-2 text-label text-muted">{c.overview}</p>}
              </div>
            </button>
          ))}
        </div>
      </Modal>
    </>
  )
}
