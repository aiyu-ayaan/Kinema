import { useLayoutEffect, useRef, type UIEvent } from 'react'

// The player remounts for every title (key={id}); the list's scroll position lives here so it survives that.
const keptScroll = { show: null as number | null, top: 0 }

/** Scroll position of the side list, carried from one episode of a show to the next. */
export function useKeptScroll(showId: number | null) {
  const ref = useRef<HTMLElement>(null)
  useLayoutEffect(() => {
    if (ref.current && showId != null && keptScroll.show === showId) ref.current.scrollTop = keptScroll.top
  }, [showId])
  return {
    ref,
    onScroll: (e: UIEvent<HTMLElement>) => {
      keptScroll.show = showId
      keptScroll.top = e.currentTarget.scrollTop
    },
  }
}
