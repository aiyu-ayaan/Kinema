import { useState } from 'react'
import { FilmSlate } from '@phosphor-icons/react'
import { imageUrl, sharperImageUrl } from '@/lib/api'
import { cn } from '@/lib/cn'

/** Deterministic warm gradient so items without art still look distinct. */
function placeholderHue(seed: string) {
  let h = 0
  for (const c of seed) h = (h * 31 + c.charCodeAt(0)) % 360
  return h
}

/**
 * TMDB art first, then `frame` (a video frame from the server's thumbnail cache),
 * then a gradient with the title. A frame carries no title, so posters built from
 * one get the title printed over it.
 */
export function Artwork({
  image,
  frame,
  title,
  className,
  aspect = 'poster',
  eager,
}: {
  image: string | null
  frame?: string | null
  title: string
  className?: string
  aspect?: 'poster' | 'wide'
  eager?: boolean
}) {
  // Wide cards show episode stills, stored at 300 px: ask for a bigger copy first.
  const sources = [aspect === 'wide' ? sharperImageUrl(image, 780) : null, imageUrl(image), frame].filter((s): s is string => !!s)
  const [failed, setFailed] = useState(0)
  const src = sources[failed] ?? null
  const isFrame = src != null && src === frame
  const hue = placeholderHue(title)
  return (
    <div
      className={cn('relative overflow-hidden rounded-lg bg-raised', aspect === 'poster' ? 'aspect-[2/3]' : 'aspect-video', className)}
      style={src ? undefined : { background: `linear-gradient(145deg, hsl(${hue} 35% 26%), hsl(${(hue + 40) % 360} 30% 14%))` }}
    >
      {src ? (
        <>
          <img key={src} src={src} alt="" loading={eager ? 'eager' : 'lazy'} decoding="async" onError={() => setFailed((f) => f + 1)} className="h-full w-full object-cover" />
          {isFrame && aspect === 'poster' && (
            <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3 pt-10">
              <span className="text-over-media line-clamp-3 text-label font-semibold text-white">{title}</span>
            </div>
          )}
        </>
      ) : (
        <div className="flex h-full w-full flex-col items-center justify-center gap-2 p-3 text-center text-white/80">
          <FilmSlate size={aspect === 'poster' ? 28 : 22} weight="duotone" aria-hidden />
          <span className="line-clamp-3 text-label font-medium">{title}</span>
        </div>
      )}
    </div>
  )
}
