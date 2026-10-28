import { useEffect, useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Check, MagnifyingGlass } from '@phosphor-icons/react'
import { Artwork } from '@/components/media/Artwork'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Field, inputCls } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import type { MatchCandidate, RequestKind } from '@/lib/types'

function useDebounced<T>(value: T, ms: number): T {
  const [v, setV] = useState(value)
  useEffect(() => {
    const id = setTimeout(() => setV(value), ms)
    return () => clearTimeout(id)
  }, [value, ms])
  return v
}

/** Ask for a movie or show: pick it on TMDB, or type the name when the server has no TMDB key. Mounted only while open. */
export function RequestDialog({ onClose }: { onClose: () => void }) {
  const qc = useQueryClient()
  const [kind, setKind] = useState<RequestKind>('movie')
  const [query, setQuery] = useState('')
  const [year, setYear] = useState('')
  const [note, setNote] = useState('')
  const [picked, setPicked] = useState<MatchCandidate | null>(null)
  const [typed, setTyped] = useState(false)
  const q = useDebounced(query.trim(), 350)
  const search = useQuery({ queryKey: ['request-search', kind, q], queryFn: () => api.requestSearch(q, kind), enabled: q.length > 0 && !picked })
  // Without TMDB there is nothing to pick from; the typed name is the request.
  const manual = typed || search.data?.tmdb === false
  const create = useMutation({
    mutationFn: () => {
      const y = Number(year)
      return api.createRequest({
        kind,
        title: picked?.title ?? query.trim(),
        year: picked ? picked.year : year && Number.isFinite(y) ? y : null,
        tmdb_id: picked?.tmdb_id ?? null,
        poster: picked?.poster ?? null,
        overview: picked?.overview ?? null,
        note: note.trim() || null,
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['requests'] })
      onClose()
    },
  })
  const ready = picked != null || (manual && query.trim().length > 0)

  function submit(e: FormEvent) {
    e.preventDefault()
    if (ready) create.mutate()
  }

  return (
    <Modal
      open
      onClose={onClose}
      title={t('requests.dialog.title')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" type="submit" form="request-form" loading={create.isPending} disabled={!ready}>
            {t('requests.dialog.submit')}
          </Button>
        </>
      }
    >
      <form id="request-form" onSubmit={submit} className="flex flex-col gap-4">
        <div role="radiogroup" aria-label={t('requests.dialog.kind')} className="flex gap-1">
          {(['movie', 'show'] as const).map((k) => (
            <button
              key={k}
              type="button"
              role="radio"
              aria-checked={kind === k}
              onClick={() => {
                setKind(k)
                setPicked(null)
              }}
              className={cn(
                'h-8 cursor-pointer rounded-md px-4 text-label font-medium transition-colors duration-fast',
                kind === k ? 'bg-primary-container text-on-primary-container' : 'text-muted hover:bg-raised hover:text-fg',
              )}
            >
              {t(k === 'movie' ? 'requests.kind.movie' : 'requests.kind.show')}
            </button>
          ))}
        </div>

        {picked ? (
          <div className="flex items-center gap-3 rounded-md border border-border bg-raised p-2">
            <Artwork image={picked.poster ? `w500${picked.poster}` : null} title={picked.title} className="w-12 shrink-0" />
            <p className="min-w-0 flex-1 text-body font-medium">
              {picked.title}
              {picked.year && <span className="ms-2 text-muted">{picked.year}</span>}
            </p>
            <Button size="sm" variant="ghost" onClick={() => setPicked(null)}>
              {t('requests.dialog.change')}
            </Button>
          </div>
        ) : (
          <>
            <div className="relative">
              <MagnifyingGlass size={16} className="pointer-events-none absolute start-3 top-1/2 -translate-y-1/2 text-subtle" aria-hidden />
              <input className={cn(inputCls, 'ps-9')} autoFocus value={query} maxLength={200} onChange={(e) => setQuery(e.target.value)} aria-label={t('requests.dialog.name')} placeholder={t('requests.dialog.name')} />
            </div>
            {manual ? (
              <>
                {search.data?.tmdb === false && <p className="text-caption text-muted">{t('requests.dialog.noTmdb')}</p>}
                <Field label={t('requests.dialog.year')} inputMode="numeric" className="w-32" value={year} onChange={(e) => setYear(e.target.value.replace(/\D/g, '').slice(0, 4))} />
              </>
            ) : (
              <div className="flex flex-col gap-1">
                {search.isFetching && <p className="py-2 text-center text-label text-muted">{t('requests.dialog.searching')}</p>}
                {search.data?.results.map((c) => (
                  <button key={c.tmdb_id} type="button" onClick={() => setPicked(c)} className="flex cursor-pointer gap-3 rounded-md p-2 text-start transition-colors hover:bg-raised">
                    <Artwork image={c.poster ? `w500${c.poster}` : null} title={c.title} className="w-14 shrink-0" />
                    <div className="min-w-0">
                      <p className="text-body font-medium">
                        {c.title}
                        {c.year && <span className="ms-2 text-muted">{c.year}</span>}
                      </p>
                      {c.overview && <p className="mt-0.5 line-clamp-2 text-label text-muted">{c.overview}</p>}
                    </div>
                  </button>
                ))}
                {search.data?.results.length === 0 && !search.isFetching && <p className="py-2 text-center text-body text-muted">{t('requests.dialog.none')}</p>}
                {q.length > 0 && !search.isFetching && (
                  <Button variant="ghost" size="sm" icon={<Check size={14} />} className="self-start" onClick={() => setTyped(true)}>
                    {t('requests.dialog.useTyped')}
                  </Button>
                )}
              </div>
            )}
          </>
        )}

        <Field label={t('requests.dialog.note')} value={note} maxLength={500} onChange={(e) => setNote(e.target.value)} hint={t('requests.dialog.noteHint')} />
        <ErrorText>{(search.error ?? create.error)?.message}</ErrorText>
      </form>
    </Modal>
  )
}
