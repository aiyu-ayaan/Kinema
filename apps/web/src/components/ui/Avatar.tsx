import { cn } from '@/lib/cn'
import type { AvatarColor } from '@/lib/types'
import { avatarColors } from './avatarColors'

export function Avatar({ name, color, size = 32, className }: { name: string; color: AvatarColor | null; size?: number; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn('inline-flex shrink-0 select-none items-center justify-center rounded-full font-semibold uppercase text-white', className)}
      style={{ width: size, height: size, fontSize: size * 0.42, background: avatarColors[color ?? 'ember'] }}
    >
      {name.trim().slice(0, 1)}
    </span>
  )
}
