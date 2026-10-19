import { useSyncExternalStore } from 'react'
import { storageKeys } from './flavor'

export type Theme = 'system' | 'dark' | 'light'

function read(): Theme {
  try {
    return (localStorage.getItem(storageKeys.theme) as Theme) || 'system'
  } catch {
    return 'system'
  }
}

let current: Theme = read()
const listeners = new Set<() => void>()

function sync(theme: Theme) {
  if (theme === 'system') delete document.documentElement.dataset.theme
  else document.documentElement.dataset.theme = theme
}

export function applyStoredTheme() {
  sync(current)
}

export function setTheme(theme: Theme) {
  current = theme
  try {
    localStorage.setItem(storageKeys.theme, theme)
  } catch {
    /* not persisted in private mode */
  }
  sync(theme)
  listeners.forEach((l) => l())
}

export function useTheme(): Theme {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => current,
  )
}
