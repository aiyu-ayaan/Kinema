import { useRef, useState } from 'react'
import { DownloadSimple, UploadSimple } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Select } from '@/components/ui/Controls'
import { Modal } from '@/components/ui/Modal'
import { downloadJson } from '@/lib/api'
import { languageName } from '@/lib/format'
import { t, type MessageKey } from '@/lib/i18n'
import type { ImportResult, Languages, ProfileOptions, SubtitleMode } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

/** Select value for "follow the server" (null); '' stays "file default". */
const SERVER = '__server__'

function LanguageSelect({ id, label, hint, value, codes, onChange }: { id: string; label: string; hint?: string; value: string | null; codes: string[]; onChange: (v: string | null) => void }) {
  return (
    <div className="flex flex-col gap-1.5">
      <label htmlFor={id} className="text-label font-medium">
        {label}
      </label>
      <Select id={id} value={value ?? SERVER} onChange={(e) => onChange(e.target.value === SERVER ? null : e.target.value)} aria-describedby={hint ? `${id}-hint` : undefined}>
        <option value={SERVER}>{t('profile.serverDefault')}</option>
        <option value="">{t('settings.languages.fileDefault')}</option>
        {codes.map((c) => (
          <option key={c} value={c}>
            {languageName(c) ?? c}
          </option>
        ))}
      </Select>
      {hint && (
        <p id={`${id}-hint`} className="text-caption text-muted">
          {hint}
        </p>
      )}
    </div>
  )
}

/** Audio / subtitle language and subtitle mode, each with "Server default". */
export function LanguageFields({ value, onChange, options, idPrefix }: { value: Languages; onChange: (v: Languages) => void; options: ProfileOptions; idPrefix: string }) {
  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <LanguageSelect id={`${idPrefix}-audio`} label={t('profile.audio')} value={value.audio_language} codes={options.track_languages} onChange={(v) => onChange({ ...value, audio_language: v })} />
      <LanguageSelect id={`${idPrefix}-sub`} label={t('profile.subtitle')} value={value.subtitle_language} codes={options.track_languages} onChange={(v) => onChange({ ...value, subtitle_language: v })} />
      <div className="flex flex-col gap-1.5">
        <label htmlFor={`${idPrefix}-mode`} className="text-label font-medium">
          {t('profile.subtitleMode')}
        </label>
        <Select id={`${idPrefix}-mode`} value={value.subtitle_mode ?? SERVER} onChange={(e) => onChange({ ...value, subtitle_mode: e.target.value === SERVER ? null : (e.target.value as SubtitleMode) })}>
          <option value={SERVER}>{t('profile.serverDefault')}</option>
          {options.subtitle_modes.map((m) => (
            <option key={m} value={m}>
              {t(`settings.languages.mode.${m}` as MessageKey)}
            </option>
          ))}
        </Select>
      </div>
    </div>
  )
}

/**
 * Export or import a profile file. `withLock` offers the explicit PIN / password options
 * (admins only); a profile importing into itself never changes its limits or lock.
 */
export function TransferModal({
  open,
  onClose,
  name,
  withLock,
  onExport,
  onImport,
}: {
  open: boolean
  onClose: () => void
  name: string
  withLock: boolean
  onExport: (includeLock: boolean) => Promise<unknown>
  onImport: (data: unknown, applyLock: boolean) => Promise<ImportResult>
}) {
  const fileInput = useRef<HTMLInputElement>(null)
  const [includeLock, setIncludeLock] = useState(false)
  const [applyLock, setApplyLock] = useState(false)
  const [busy, setBusy] = useState<'export' | 'import' | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<ImportResult | null>(null)

  async function doExport() {
    setBusy('export')
    setError(null)
    try {
      const safe = name.replace(/[^\w-]+/g, '_')
      downloadJson(await onExport(withLock && includeLock), `kinema-profile-${safe}.json`)
    } catch (e) {
      setError(errorText(e))
    } finally {
      setBusy(null)
    }
  }

  async function doImport(file: File) {
    setBusy('import')
    setError(null)
    setResult(null)
    try {
      let data: unknown
      try {
        data = JSON.parse(await file.text())
      } catch {
        throw new Error(t('transfer.badFile'))
      }
      setResult(await onImport(data, withLock && applyLock))
    } catch (e) {
      setError(errorText(e))
    } finally {
      setBusy(null)
      if (fileInput.current) fileInput.current.value = ''
    }
  }

  const checkbox = (id: string, checked: boolean, set: (v: boolean) => void, label: string, hint?: string) => (
    <label htmlFor={id} className="flex cursor-pointer items-start gap-2 text-label">
      <input id={id} type="checkbox" checked={checked} onChange={(e) => set(e.target.checked)} className="mt-0.5 h-4 w-4 accent-[rgb(var(--primary))]" />
      <span>
        {label}
        {hint && <span className="block text-caption text-muted">{hint}</span>}
      </span>
    </label>
  )

  return (
    <Modal open={open} onClose={onClose} title={t('transfer.open', { name })} footer={<Button variant="ghost" onClick={onClose}>{t('common.close')}</Button>}>
      <div className="flex flex-col gap-6">
        <section className="flex flex-col gap-3">
          <h3 className="text-body font-semibold">{t('transfer.export')}</h3>
          <p className="text-caption text-muted">{t('transfer.exportHint')}</p>
          {withLock && checkbox('transfer-include-lock', includeLock, setIncludeLock, t('transfer.includeLock'), t('transfer.includeLockHint'))}
          <Button icon={<DownloadSimple size={16} />} loading={busy === 'export'} onClick={doExport} className="self-start">
            {t('transfer.export')}
          </Button>
        </section>
        <section className="flex flex-col gap-3">
          <h3 className="text-body font-semibold">{t('transfer.import')}</h3>
          <p className="text-caption text-muted">{withLock ? t('transfer.importHint') : t('transfer.importSelfHint')}</p>
          {withLock && checkbox('transfer-apply-lock', applyLock, setApplyLock, t('transfer.applyLock'))}
          <input ref={fileInput} type="file" accept="application/json,.json" hidden onChange={(e) => e.target.files?.[0] && doImport(e.target.files[0])} />
          <Button icon={<UploadSimple size={16} />} loading={busy === 'import'} onClick={() => fileInput.current?.click()} className="self-start">
            {t('transfer.import')}
          </Button>
          <div aria-live="polite" className="text-caption text-muted">
            {result && (
              <>
                <p>{t('transfer.result', { imported: result.imported, kept: result.kept, unmatched: result.unmatched })}</p>
                {result.lock_applied && <p>{t('transfer.lockApplied')}</p>}
              </>
            )}
          </div>
          <ErrorText>{error}</ErrorText>
        </section>
      </div>
    </Modal>
  )
}
