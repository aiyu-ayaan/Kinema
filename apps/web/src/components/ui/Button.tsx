import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react'
import { CircleNotch } from '@phosphor-icons/react'
import { cn } from '@/lib/cn'

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger'
type Size = 'sm' | 'md' | 'lg'

const variants: Record<Variant, string> = {
  primary: 'bg-primary text-on-primary hover:bg-primary/90',
  secondary: 'border border-border bg-raised text-fg hover:border-border-strong',
  ghost: 'text-muted hover:bg-raised hover:text-fg',
  danger: 'border border-border text-danger hover:border-danger/40 hover:bg-danger/10',
}
const sizes: Record<Size, string> = {
  sm: 'h-8 gap-1.5 px-3 text-label',
  md: 'h-9 gap-2 px-3.5 text-body',
  lg: 'h-11 gap-2 px-5 text-title',
}
const iconOnly: Record<Size, string> = { sm: 'w-8 px-0', md: 'w-9 px-0', lg: 'w-11 px-0' }

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
  size?: Size
  loading?: boolean
  icon?: ReactNode
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = 'secondary', size = 'md', loading, icon, children, className, disabled, type = 'button', ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      className={cn(
        'inline-flex shrink-0 cursor-pointer select-none items-center justify-center rounded-md font-medium transition-colors duration-fast ease-std active:opacity-80 disabled:cursor-not-allowed disabled:opacity-40',
        variants[variant],
        sizes[size],
        !children && iconOnly[size],
        className,
      )}
      {...rest}
    >
      {loading ? <CircleNotch className="animate-spin" size={16} aria-hidden /> : icon}
      {children}
    </button>
  )
})
