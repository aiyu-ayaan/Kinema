import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowsLeftRight } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { StatsView } from '@/features/activity/StatsView'
import { Card, CardHeader, ErrorText, PageHeader, Skeleton } from '@/components/ui/Card'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { Languages } from '@/lib/types'
import { LanguageFields, TransferModal } from './ProfileFields'
import { ratingLabel } from './rating'

/** The signed-in profile's own settings: track languages, and export / import. */
export function ProfilePage() {
  const qc = useQueryClient()
  const { data, isLoading } = useQuery({ queryKey: ['my-profile'], queryFn: api.myProfile })
  const stats = useQuery({ queryKey: ['my-stats'], queryFn: api.myStats })
  const [langs, setLangs] = useState<Languages | null>(null)
  const [transfer, setTransfer] = useState(false)
  useEffect(() => {
    if (data) setLangs({ audio_language: data.preferences.audio_language, subtitle_language: data.preferences.subtitle_language, subtitle_mode: data.preferences.subtitle_mode })
  }, [data])
  const save = useMutation({
    mutationFn: api.setLanguages,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['my-profile'] }),
  })

  if (isLoading || !data || !langs) return <Skeleton className="h-64" />
  const limit = data.preferences.max_rating

  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-6">
      <PageHeader
        title={t('profile.title')}
        description={t('profile.description', { name: data.user.name })}
        actions={
          <Button icon={<ArrowsLeftRight size={16} />} onClick={() => setTransfer(true)}>
            {t('transfer.title')}
          </Button>
        }
      />
      <Card>
        <CardHeader title={t('profile.languages')} />
        <form
          className="flex flex-col gap-4 p-4"
          onSubmit={(e) => {
            e.preventDefault()
            save.mutate(langs)
          }}
        >
          <LanguageFields idPrefix="me" value={langs} onChange={setLangs} options={data.options} />
          <div className="flex items-center gap-3">
            <Button type="submit" variant="primary" loading={save.isPending}>
              {t('common.save')}
            </Button>
            <span aria-live="polite" className="text-caption text-muted">
              {save.isSuccess && t('profile.saved')}
            </span>
          </div>
          <ErrorText>{save.error?.message}</ErrorText>
          {!data.user.is_admin && limit != null && <p className="text-caption text-muted">{t('profile.limitNote', { limit: ratingLabel(limit) })}</p>}
        </form>
      </Card>
      <section aria-labelledby="watch-activity" className="space-y-3">
        <h2 id="watch-activity" className="text-title font-semibold">
          {t('profile.activity')}
        </h2>
        <ErrorText>{stats.error?.message}</ErrorText>
        {stats.data ? <StatsView stats={stats.data} /> : !stats.error && <Skeleton className="h-48" />}
      </section>
      {transfer && (
        <TransferModal
          open
          onClose={() => setTransfer(false)}
          name={data.user.name}
          withLock={false}
          onExport={() => api.exportProfile()}
          onImport={async (file) => {
            const r = await api.importProfile(file)
            qc.invalidateQueries()
            return r
          }}
        />
      )}
    </div>
  )
}
