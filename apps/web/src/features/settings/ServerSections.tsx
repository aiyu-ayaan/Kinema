import { useEffect, useState, type ReactNode } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CheckCircle, WarningCircle } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, ErrorText, Notice, Skeleton } from '@/components/ui/Card'
import { SettingRow, Select, Switch } from '@/components/ui/Controls'
import { Field, inputCls } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { languageName } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ServerOptions, SettingsView } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/**
 * Edits a copy of the server options and saves the whole object, so each section only
 * renders its own rows. Save is enabled once something changed.
 */
function ServerForm({ title, children }: { title: string; children: (s: ServerOptions, set: (patch: Partial<ServerOptions>) => void, view: SettingsView) => ReactNode }) {
  const qc = useQueryClient()
  const { data } = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const [draft, setDraft] = useState<ServerOptions | null>(null)
  useEffect(() => {
    if (data) setDraft(data.server)
  }, [data])
  const save = useMutation({
    mutationFn: api.saveServerSettings,
    onSuccess: (v) => {
      qc.setQueryData(['settings'], v)
      qc.invalidateQueries({ queryKey: ['server-info'] })
      qc.invalidateQueries({ queryKey: ['libraries'] })
    },
  })
  if (!data || !draft) return <Skeleton className="h-48" />
  const dirty = JSON.stringify(draft) !== JSON.stringify(data.server)
  return (
    <Card>
      <CardHeader title={title} />
      <div className="divide-y divide-border">{children(draft, (patch) => setDraft({ ...draft, ...patch }), data)}</div>
      <div className="flex items-center justify-end gap-2 border-t border-border px-4 py-3">
        <ErrorText>{save.error && errorText(save.error)}</ErrorText>
        {save.isSuccess && !dirty && <span className="text-label text-muted">{t('settings.saved')}</span>}
        <Button variant="ghost" disabled={!dirty} onClick={() => setDraft(data.server)}>
          {t('common.reset')}
        </Button>
        <Button variant="primary" disabled={!dirty} loading={save.isPending} onClick={() => save.mutate(draft)}>
          {t('common.save')}
        </Button>
      </div>
    </Card>
  )
}

export function GeneralSection() {
  return (
    <ServerForm title={t('settings.nav.general')}>
      {(s, set) => (
        <>
          <SettingRow label={t('settings.general.name')} description={t('settings.general.nameHint')} htmlFor="server-name">
            <input id="server-name" className={cn(inputCls, 'w-64')} value={s.server_name} maxLength={60} onChange={(e) => set({ server_name: e.target.value })} />
          </SettingRow>
          <SettingRow label={t('settings.general.publicUrl')} description={t('settings.general.publicUrlHint')} htmlFor="public-url">
            <input
              id="public-url"
              type="url"
              inputMode="url"
              className={cn(inputCls, 'w-64')}
              value={s.public_url}
              maxLength={200}
              placeholder="https://media.example.com"
              onChange={(e) => set({ public_url: e.target.value })}
            />
          </SettingRow>
          {typeof window !== 'undefined' && (window.kinema?.chooseServer || window.desktop?.chooseServer) && (
            <SettingRow label={t('auth.changeServer')} description={t('connect.subtitle')}>
              <Button
                variant="ghost"
                onClick={() => (window.kinema?.chooseServer || window.desktop?.chooseServer)?.()}
              >
                {t('auth.changeServer')}
              </Button>
            </SettingRow>
          )}
        </>
      )}
    </ServerForm>
  )
}

export function LibrarySection() {
  return (
    <ServerForm title={t('settings.nav.library')}>
      {(s, set, view) => (
        <>
          <SettingRow label={t('settings.library.watch')} description={t('settings.library.watchHint')}>
            <Switch checked={s.watch_changes} onChange={(v) => set({ watch_changes: v })} label={t('settings.library.watch')} />
          </SettingRow>
          <SettingRow label={t('settings.library.thumbnails')} description={view.ffmpeg_found ? t('settings.library.thumbnailsHint') : t('settings.ffmpegMissing')}>
            <Switch checked={s.thumbnails} onChange={(v) => set({ thumbnails: v })} label={t('settings.library.thumbnails')} />
          </SettingRow>
          <SettingRow label={t('settings.library.position')} description={t('settings.library.positionHint')} htmlFor="thumb-pos">
            <div className="flex items-center gap-3">
              <input
                id="thumb-pos"
                type="range"
                className="range w-40"
                min={5}
                max={90}
                step={5}
                value={s.thumbnail_position}
                disabled={!s.thumbnails}
                style={{ ['--fill' as string]: `${((s.thumbnail_position - 5) / 85) * 100}%` }}
                onChange={(e) => set({ thumbnail_position: Number(e.target.value) })}
              />
              <span className="w-10 text-end text-label tabular text-muted">{s.thumbnail_position}%</span>
            </div>
          </SettingRow>
        </>
      )}
    </ServerForm>
  )
}

function TmdbKey() {
  const qc = useQueryClient()
  const { data } = useQuery({ queryKey: ['settings'], queryFn: api.settings })
  const [key, setKey] = useState('')
  const save = useMutation({
    mutationFn: api.saveTmdbKey,
    onSuccess: (v) => {
      qc.setQueryData(['settings'], v)
      qc.invalidateQueries({ queryKey: ['libraries'] })
      setKey('')
    },
  })
  return (
    <Card>
      <CardHeader
        title={t('settings.tmdb.title')}
        action={
          data && (
            <Badge tone={data.tmdb_configured ? 'primary' : 'warn'}>
              {data.tmdb_configured ? <CheckCircle size={12} weight="fill" /> : <WarningCircle size={12} weight="fill" />}
              {t(data.tmdb_configured ? 'settings.tmdb.on' : 'settings.tmdb.off')}
            </Badge>
          )
        }
      />
      <form
        className="flex flex-col gap-3 p-4"
        onSubmit={(e) => {
          e.preventDefault()
          if (key.trim()) save.mutate(key.trim())
        }}
      >
        <p className="text-body text-muted">{t('settings.tmdb.description')}</p>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-end">
          <Field className="flex-1" label={t('settings.tmdb.label')} type="password" autoComplete="off" placeholder={data?.tmdb_configured ? t('settings.tmdb.replace') : ''} value={key} onChange={(e) => setKey(e.target.value)} />
          <Button type="submit" variant="primary" loading={save.isPending} disabled={!key.trim()}>
            {t('common.save')}
          </Button>
          {data?.tmdb_configured && (
            <Button variant="danger" onClick={() => save.mutate('')}>
              {t('common.remove')}
            </Button>
          )}
        </div>
        <ErrorText>{save.error && errorText(save.error)}</ErrorText>
      </form>
    </Card>
  )
}

export function MetadataSection() {
  return (
    <div className="space-y-6">
      <TmdbKey />
      <ServerForm title={t('settings.metadata.title')}>
        {(s, set, view) => (
          <SettingRow label={t('settings.metadata.language')} description={t('settings.metadata.languageHint')} htmlFor="meta-lang">
            <Select id="meta-lang" value={s.metadata_language} onChange={(e) => set({ metadata_language: e.target.value })}>
              {view.options.metadata_languages.map((l) => (
                <option key={l} value={l}>
                  {languageName(l)} ({l})
                </option>
              ))}
            </Select>
          </SettingRow>
        )}
      </ServerForm>
    </div>
  )
}

function TrackLanguage({ id, value, onChange, codes }: { id: string; value: string; onChange: (v: string) => void; codes: string[] }) {
  return (
    <Select id={id} value={value} onChange={(e) => onChange(e.target.value)}>
      <option value="">{t('settings.languages.fileDefault')}</option>
      {codes
        .map((c) => [c, languageName(c) ?? c] as const)
        .sort((a, b) => a[1].localeCompare(b[1]))
        .map(([c, name]) => (
          <option key={c} value={c}>
            {name}
          </option>
        ))}
    </Select>
  )
}

export function LanguagesSection() {
  return (
    <ServerForm title={t('settings.nav.languages')}>
      {(s, set, view) => (
        <>
          <SettingRow label={t('settings.languages.audio')} description={t('settings.languages.audioHint')} htmlFor="audio-lang">
            <TrackLanguage id="audio-lang" value={s.preferred_audio_language} onChange={(v) => set({ preferred_audio_language: v })} codes={view.options.track_languages} />
          </SettingRow>
          <SettingRow label={t('settings.languages.subtitleMode')} description={t(`settings.languages.mode.${s.subtitle_mode}.hint`)} htmlFor="sub-mode">
            <Select id="sub-mode" value={s.subtitle_mode} onChange={(e) => set({ subtitle_mode: e.target.value as ServerOptions['subtitle_mode'] })}>
              {view.options.subtitle_modes.map((m) => (
                <option key={m} value={m}>
                  {t(`settings.languages.mode.${m}`)}
                </option>
              ))}
            </Select>
          </SettingRow>
          <SettingRow label={t('settings.languages.subtitle')} description={t('settings.languages.subtitleHint')} htmlFor="sub-lang">
            <TrackLanguage id="sub-lang" value={s.preferred_subtitle_language} onChange={(v) => set({ preferred_subtitle_language: v })} codes={view.options.track_languages} />
          </SettingRow>
        </>
      )}
    </ServerForm>
  )
}

export function TranscoderSection() {
  return (
    <div className="space-y-6">
      <Notice>{t('settings.transcoder.intro')}</Notice>
      <ServerForm title={t('settings.nav.transcoder')}>
        {(s, set, view) => (
          <>
            <SettingRow label={t('settings.transcoder.max')} description={t('settings.transcoder.maxHint')} htmlFor="max-tc">
              <Select id="max-tc" value={s.max_transcodes} onChange={(e) => set({ max_transcodes: Number(e.target.value) })}>
                {[1, 2, 3, 4, 6, 8].map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </Select>
            </SettingRow>
            <SettingRow label={t('settings.transcoder.preset')} description={t('settings.transcoder.presetHint')} htmlFor="preset">
              <Select id="preset" value={s.transcode_preset} onChange={(e) => set({ transcode_preset: e.target.value })}>
                {view.options.presets.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </Select>
            </SettingRow>
            <SettingRow label={t('settings.transcoder.quality')} description={t('settings.transcoder.qualityHint')} htmlFor="crf">
              <div className="flex items-center gap-3">
                <span className="text-caption text-subtle">{t('settings.transcoder.smaller')}</span>
                {/* The slider reads as quality, so it runs from CRF 30 (left) to 16 (right). */}
                <input
                  id="crf"
                  type="range"
                  className="range w-40"
                  min={16}
                  max={30}
                  value={46 - s.transcode_crf}
                  style={{ ['--fill' as string]: `${((30 - s.transcode_crf) / 14) * 100}%` }}
                  onChange={(e) => set({ transcode_crf: 46 - Number(e.target.value) })}
                />
                <span className="text-caption text-subtle">{t('settings.transcoder.better')}</span>
                <span className="w-14 text-end text-label tabular text-muted">CRF {s.transcode_crf}</span>
              </div>
            </SettingRow>
            <SettingRow label={t('settings.transcoder.height')} description={t('settings.transcoder.heightHint')} htmlFor="height">
              <Select id="height" value={s.transcode_max_height} onChange={(e) => set({ transcode_max_height: Number(e.target.value) })}>
                {view.options.max_heights.map((h) => (
                  <option key={h} value={h}>
                    {h === 0 ? t('settings.transcoder.original') : `${h}p`}
                  </option>
                ))}
              </Select>
            </SettingRow>
          </>
        )}
      </ServerForm>
    </div>
  )
}

export function ScheduledSection() {
  return (
    <ServerForm title={t('settings.nav.scheduled')}>
      {(s, set) => (
        <SettingRow label={t('settings.scheduled.rescan')} description={t('settings.scheduled.rescanHint')} htmlFor="rescan">
          <Select id="rescan" value={s.rescan_hours} onChange={(e) => set({ rescan_hours: Number(e.target.value) })}>
            {[0, 1, 3, 6, 12, 24, 72, 168].map((h) => (
              <option key={h} value={h}>
                {h === 0 ? t('settings.scheduled.never') : h === 1 ? t('settings.scheduled.hour') : h === 24 ? t('settings.scheduled.day') : h % 24 === 0 ? t('settings.scheduled.days', { n: h / 24 }) : t('settings.scheduled.hours', { n: h })}
              </option>
            ))}
          </Select>
        </SettingRow>
      )}
    </ServerForm>
  )
}
