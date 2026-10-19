import { useState } from 'react'
import { FilmSlate } from '@phosphor-icons/react'
import { imageUrl } from '@/lib/api'
import { cn } from '@/lib/cn'

/** Deterministic warm gradient so items without art still look distinct. */
function placeholderHue(seed: string) {
  let h = 0
  for (const c of seed) h = (h * 31 + c.charCodeAt(0)) % 360
  return h
}

export function Artwork({ image, title, className, aspect = 'poster', eager }: { image: string | null; title: string; className?: string; aspect?: 'poster' | 'wide'; eager?: boolean }) {
  const [failed, setFailed] = useState(false)
  const src = failed ? null : imageUrl(image)
  const hue = placeholderHue(title)
  return (
    <div
      className={cn('relative overflow-hidden rounded-lg bg-raised', aspect === 'poster' ? 'aspect-[2/3]' : 'aspect-video', className)}
      style={src ? undefined : { background: `linear-gradient(145deg, hsl(${hue} 35% 26%), hsl(${(hue + 40) % 360} 30% 14%))` }}
    >
      {src ? (
        <img src={src} alt="" loading={eager ? 'eager' : 'lazy'} decoding="async" onError={() => setFailed(true)} className="h-full w-full object-cover" />
      ) : (
        <div className="flex h-full w-full flex-col items-center justify-center gap-2 p-3 text-center text-white/80">
          <FilmSlate size={aspect === 'poster' ? 28 : 22} weight="duotone" aria-hidden />
          <span className="line-clamp-3 text-label font-medium">{title}</span>
        </div>
      )}
    </div>
  )
}
