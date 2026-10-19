import { FilmStrip, FolderSimple, Television, type Icon } from '@phosphor-icons/react'
import type { LibraryKind } from '@/lib/types'

export const libraryIcon: Record<LibraryKind, Icon> = { movie: FilmStrip, show: Television, other: FolderSimple }
