import { useEffect, useRef, type ReactNode } from 'react'
import { X } from '@phosphor-icons/react'
import { Button } from './Button'
import { t } from '@/lib/i18n'

/** Centred dialog on the native <dialog>, so focus trapping and Escape come free. */
export function Modal({ open, onClose, title, children, footer }: { open: boolean; onClose: () => void; title: string; children: ReactNode; footer?: ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null)
  useEffect(() => {
    const d = ref.current
    if (!d) return
    if (open && !d.open) d.showModal()
    if (!open && d.open) d.close()
  }, [open])
  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onClick={(e) => e.target === ref.current && onClose()}
      className="w-[calc(100%-2rem)] max-w-lg rounded-lg border border-border bg-panel p-0 text-fg shadow-overlay backdrop:bg-black/60 open:anim-enter"
    >
      <div className="flex min-h-14 items-center justify-between gap-3 border-b border-border px-5">
        <h2 className="text-title font-semibold">{title}</h2>
        <Button variant="ghost" size="sm" onClick={onClose} aria-label={t('common.close')} icon={<X size={16} />} />
      </div>
      <div className="max-h-[70dvh] overflow-y-auto p-5">{children}</div>
      {footer && <div className="flex justify-end gap-2 border-t border-border px-5 py-3">{footer}</div>}
    </dialog>
  )
}
