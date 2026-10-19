import { forwardRef, useId, useState, type InputHTMLAttributes, type ReactNode } from 'react'
import { Eye, EyeSlash } from '@phosphor-icons/react'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'

export const inputCls =
  'h-9 w-full rounded-md border border-border bg-raised px-3 text-body text-fg placeholder:text-subtle transition-colors duration-fast focus:border-border-strong focus:outline-none focus:ring-2 focus:ring-primary/25 disabled:opacity-50'

interface FieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string
  hint?: ReactNode
  error?: string
}

export const Field = forwardRef<HTMLInputElement, FieldProps>(function Field({ label, hint, error, className, type, ...rest }, ref) {
  const id = useId()
  const [reveal, setReveal] = useState(false)
  const isPassword = type === 'password'
  return (
    <div className={cn('flex flex-col gap-1.5', className)}>
      <label htmlFor={id} className="text-label font-medium text-fg">
        {label}
      </label>
      <div className="relative">
        <input
          ref={ref}
          id={id}
          type={isPassword && reveal ? 'text' : type}
          aria-invalid={!!error || undefined}
          aria-describedby={error ? `${id}-err` : hint ? `${id}-hint` : undefined}
          className={cn(inputCls, isPassword && 'pe-10', error && 'border-danger/60')}
          {...rest}
        />
        {isPassword && (
          <button
            type="button"
            onClick={() => setReveal((r) => !r)}
            aria-label={t(reveal ? 'field.hidePassword' : 'field.showPassword')}
            className="absolute inset-y-0 end-0 flex w-9 cursor-pointer items-center justify-center text-subtle hover:text-fg"
          >
            {reveal ? <EyeSlash size={16} /> : <Eye size={16} />}
          </button>
        )}
      </div>
      {error ? (
        <p id={`${id}-err`} role="alert" className="text-caption text-danger">
          {error}
        </p>
      ) : hint ? (
        <p id={`${id}-hint`} className="text-caption text-subtle">
          {hint}
        </p>
      ) : null}
    </div>
  )
})
