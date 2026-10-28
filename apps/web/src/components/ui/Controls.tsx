import { useId, type ReactNode, type SelectHTMLAttributes } from 'react'
import { CaretDown } from '@phosphor-icons/react'
import { cn } from '@/lib/cn'
import { inputCls } from './Field'

/** One setting: label and help on the left, control on the right (stacked on phones). */
export function SettingRow({ label, description, children, htmlFor }: { label: string; description?: ReactNode; children: ReactNode; htmlFor?: string }) {
  return (
    <div className="flex flex-col gap-3 px-4 py-4 sm:flex-row sm:items-center sm:justify-between sm:gap-8">
      <div className="min-w-0">
        <label htmlFor={htmlFor} className="text-body font-medium text-fg">
          {label}
        </label>
        {description && <p className="mt-0.5 text-label text-muted">{description}</p>}
      </div>
      <div className="shrink-0 sm:max-w-[50%]">{children}</div>
    </div>
  )
}

export function Switch({ checked, onChange, id, label, disabled }: { checked: boolean; onChange: (v: boolean) => void; id?: string; label?: string; disabled?: boolean }) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        'relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border transition-colors duration-fast disabled:cursor-not-allowed disabled:opacity-50',
        checked ? 'border-primary bg-primary' : 'border-border-strong bg-raised',
      )}
    >
      <span className={cn('inline-block h-4 w-4 rounded-full shadow transition-transform duration-fast', checked ? 'translate-x-6 bg-on-primary' : 'translate-x-1 bg-border-strong')} />
    </button>
  )
}

export function Select({ className, children, ...rest }: SelectHTMLAttributes<HTMLSelectElement>) {
  const id = useId()
  return (
    <div className={cn('relative', className)}>
      <select id={rest.id ?? id} className={cn(inputCls, 'min-w-48 cursor-pointer appearance-none pe-8')} {...rest}>
        {children}
      </select>
      <CaretDown size={14} className="pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-subtle" aria-hidden />
    </div>
  )
}
