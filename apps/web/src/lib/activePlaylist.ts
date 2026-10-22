import { storageKeys } from './flavor'

/**
 * The playlist being played, kept for this tab so the player can continue with the next entry
 * instead of the next episode. Playing anything outside the playlist drops it.
 */
export const activePlaylist = {
  get id(): number | null {
    try {
      const v = Number(sessionStorage.getItem(storageKeys.playlist))
      return Number.isFinite(v) && v > 0 ? v : null
    } catch {
      return null
    }
  },
  set(id: number | null) {
    try {
      if (id) sessionStorage.setItem(storageKeys.playlist, String(id))
      else sessionStorage.removeItem(storageKeys.playlist)
    } catch {
      /* storage blocked: the player simply falls back to the next episode */
    }
  },
}

/** The entry after `current`, or null at the end or when `current` is not in the list. */
export function nextInList(ids: number[], current: number): number | null {
  const at = ids.indexOf(current)
  return at >= 0 && at + 1 < ids.length ? ids[at + 1] : null
}
