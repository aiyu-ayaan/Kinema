import { useEffect, useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { t } from '@/lib/i18n'
import { Button } from './ui/Button'
import { ErrorText } from './ui/Card'
import { Modal } from './ui/Modal'

const bridge = typeof window !== 'undefined' ? (window.kinema ?? window.desktop)?.updates : undefined

/** The desktop app's own update: what it found, the install and the download's progress (0..1, null when idle). */
// eslint-disable-next-line react-refresh/only-export-components
export function useDesktopUpdate() {
  const { data: info } = useQuery({ queryKey: ['desktop-update-info'], queryFn: () => bridge!.info(), enabled: !!bridge })
  const check = useQuery({ queryKey: ['desktop-update'], queryFn: () => bridge!.check(), enabled: !!info, retry: false, staleTime: 6 * 60 * 60_000 })
  const [progress, setProgress] = useState<number | null>(null)
  // The app reports to every listener, so About and the prompt show the same download.
  useEffect(() => bridge?.onProgress(setProgress), [])
  const install = useMutation({ mutationFn: () => bridge!.install() })
  return { bridge, info, check, install, progress, busy: install.isPending || progress !== null }
}

export function UpdateProgress({ value }: { value: number }) {
  const percent = Math.round(value * 100)
  return (
    <div className="flex items-center gap-3">
      <div className="h-1.5 flex-1 rounded-full bg-raised" role="progressbar" aria-label={t('desktopUpdate.downloading')} aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
        <div className="h-full rounded-full bg-primary transition-[width] duration-fast" style={{ width: `${percent}%` }} />
      </div>
      <span className="w-9 text-end text-label tabular-nums text-muted">{percent}%</span>
    </div>
  )
}

/** Desktop only: offers a new release of the app once at start. "Later" is remembered for that version. */
export function UpdatePrompt() {
  const { bridge, info, check, install, progress, busy } = useDesktopUpdate()
  const [closed, setClosed] = useState(false)
  const found = check.data
  const open = !!found && !!info && !closed && found.version !== info.skipped
  if (!found || !info) return null
  const later = () => {
    if (busy) return
    bridge?.skip()
    setClosed(true)
  }
  return (
    <Modal
      open={open}
      onClose={later}
      title={t('desktopUpdate.available', { version: found.version })}
      footer={
        <>
          <Button variant="ghost" disabled={busy} onClick={later}>
            {t('desktopUpdate.later')}
          </Button>
          <Button variant="primary" autoFocus loading={busy} onClick={() => install.mutate()}>
            {info.selfInstall ? t('desktopUpdate.restart') : t('desktopUpdate.download')}
          </Button>
        </>
      }
    >
      <div className="grid gap-4">
        <p className="text-body text-muted">
          {t('desktopUpdate.youHave', { version: info.version })} {info.selfInstall ? t('desktopUpdate.restartsWhenDone') : t('desktopUpdate.installerOpens')}
        </p>
        <a href={found.page} target="_blank" rel="noreferrer" className="text-label text-primary hover:underline">
          {t('update.release')}
        </a>
        {busy && <UpdateProgress value={progress ?? 0} />}
        <ErrorText>{install.data}</ErrorText>
      </div>
    </Modal>
  )
}
