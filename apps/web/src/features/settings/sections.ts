import { ArrowCircleUp, Books, CalendarCheck, Database, FilmSlate, FolderSimple, Gauge, Info, SlidersHorizontal, Translate, UsersThree, WifiHigh, Wrench, type Icon } from '@phosphor-icons/react'
import type { MessageKey } from '@/lib/i18n'

export type SectionPath =
  | 'dashboard'
  | 'general'
  | 'library'
  | 'metadata'
  | 'languages'
  | 'transcoder'
  | 'network'
  | 'scheduled'
  | 'libraries'
  | 'users'
  | 'troubleshooting'
  | 'updates'
  | 'about'

export const settingsGroups: { label: MessageKey; sections: { path: SectionPath; label: MessageKey; icon: Icon }[] }[] = [
  { label: 'settings.group.status', sections: [{ path: 'dashboard', label: 'settings.nav.dashboard', icon: Gauge }] },
  {
    label: 'settings.group.settings',
    sections: [
      { path: 'general', label: 'settings.nav.general', icon: SlidersHorizontal },
      { path: 'library', label: 'settings.nav.library', icon: Books },
      { path: 'metadata', label: 'settings.nav.metadata', icon: Database },
      { path: 'languages', label: 'settings.nav.languages', icon: Translate },
      { path: 'transcoder', label: 'settings.nav.transcoder', icon: FilmSlate },
      { path: 'network', label: 'settings.nav.network', icon: WifiHigh },
      { path: 'scheduled', label: 'settings.nav.scheduled', icon: CalendarCheck },
    ],
  },
  {
    label: 'settings.group.manage',
    sections: [
      { path: 'libraries', label: 'settings.nav.libraries', icon: FolderSimple },
      { path: 'users', label: 'settings.nav.users', icon: UsersThree },
      { path: 'troubleshooting', label: 'settings.nav.troubleshooting', icon: Wrench },
    ],
  },
  {
    label: 'settings.group.kinema',
    sections: [
      { path: 'updates', label: 'settings.nav.updates', icon: ArrowCircleUp },
      { path: 'about', label: 'settings.nav.about', icon: Info },
    ],
  },
]

