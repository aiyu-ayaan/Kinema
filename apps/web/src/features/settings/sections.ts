import { ArrowCircleUp, Bell, Books, CalendarCheck, Database, Devices, FilmSlate, FolderSimple, Gauge, Info, Pulse, Rows, ShareNetwork, SlidersHorizontal, Ticket, Translate, UsersThree, WifiHigh, Wrench, type Icon } from '@phosphor-icons/react'
import type { MessageKey } from '@/lib/i18n'

export type SectionPath =
  | 'dashboard'
  | 'activity'
  | 'general'
  | 'library'
  | 'metadata'
  | 'languages'
  | 'transcoder'
  | 'network'
  | 'scheduled'
  | 'notifications'
  | 'libraries'
  | 'shelves'
  | 'users'
  | 'shares'
  | 'devices'
  | 'requests'
  | 'troubleshooting'
  | 'updates'
  | 'about'

export const settingsGroups: { label: MessageKey; sections: { path: SectionPath; label: MessageKey; icon: Icon }[] }[] = [
  { label: 'settings.group.status', sections: [
      { path: 'dashboard', label: 'settings.nav.dashboard', icon: Gauge },
      { path: 'activity', label: 'settings.nav.activity', icon: Pulse },
    ],
  },
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
      { path: 'notifications', label: 'settings.nav.notifications', icon: Bell },
    ],
  },
  {
    label: 'settings.group.manage',
    sections: [
      { path: 'libraries', label: 'settings.nav.libraries', icon: FolderSimple },
      { path: 'shelves', label: 'settings.nav.shelves', icon: Rows },
      { path: 'users', label: 'settings.nav.users', icon: UsersThree },
      { path: 'shares', label: 'settings.nav.shares', icon: ShareNetwork },
      { path: 'devices', label: 'settings.nav.devices', icon: Devices },
      { path: 'requests', label: 'settings.nav.requests', icon: Ticket },
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

