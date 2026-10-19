import { NavLink, Navigate, Route, Routes } from 'react-router-dom'
import { CalendarCheck, Database, FilmSlate, FolderSimple, Gauge, SlidersHorizontal, Translate, WifiHigh, Wrench, Books, type Icon } from '@phosphor-icons/react'
import { cn } from '@/lib/cn'
import { t, type MessageKey } from '@/lib/i18n'
import { LibrariesSection } from './LibrariesSection'
import { GeneralSection, LanguagesSection, LibrarySection, MetadataSection, ScheduledSection, TranscoderSection } from './ServerSections'
import { DashboardSection, NetworkSection, TroubleshootingSection } from './StatusSections'

interface Section {
  path: string
  label: MessageKey
  icon: Icon
  element: JSX.Element
}

const groups: { label: MessageKey; sections: Section[] }[] = [
  { label: 'settings.group.status', sections: [{ path: 'dashboard', label: 'settings.nav.dashboard', icon: Gauge, element: <DashboardSection /> }] },
  {
    label: 'settings.group.settings',
    sections: [
      { path: 'general', label: 'settings.nav.general', icon: SlidersHorizontal, element: <GeneralSection /> },
      { path: 'library', label: 'settings.nav.library', icon: Books, element: <LibrarySection /> },
      { path: 'metadata', label: 'settings.nav.metadata', icon: Database, element: <MetadataSection /> },
      { path: 'languages', label: 'settings.nav.languages', icon: Translate, element: <LanguagesSection /> },
      { path: 'transcoder', label: 'settings.nav.transcoder', icon: FilmSlate, element: <TranscoderSection /> },
      { path: 'network', label: 'settings.nav.network', icon: WifiHigh, element: <NetworkSection /> },
      { path: 'scheduled', label: 'settings.nav.scheduled', icon: CalendarCheck, element: <ScheduledSection /> },
    ],
  },
  {
    label: 'settings.group.manage',
    sections: [
      { path: 'libraries', label: 'settings.nav.libraries', icon: FolderSimple, element: <LibrariesSection /> },
      { path: 'troubleshooting', label: 'settings.nav.troubleshooting', icon: Wrench, element: <TroubleshootingSection /> },
    ],
  },
]

const all = groups.flatMap((g) => g.sections)

function SectionLink({ s }: { s: Section }) {
  const I = s.icon
  return (
    <NavLink
      to={`/settings/${s.path}`}
      className={({ isActive }) =>
        cn(
          'flex h-9 shrink-0 items-center gap-2.5 whitespace-nowrap rounded-md px-3 text-body font-medium transition-colors duration-fast',
          isActive ? 'bg-raised text-fg' : 'text-muted hover:bg-raised/60 hover:text-fg',
        )
      }
    >
      {({ isActive }) => (
        <>
          <I size={17} weight={isActive ? 'fill' : 'regular'} className={isActive ? 'text-primary' : undefined} aria-hidden />
          {t(s.label)}
        </>
      )}
    </NavLink>
  )
}

/** Admin settings with grouped sections, like a server control panel. */
export function SettingsPage() {
  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-6 lg:flex-row lg:gap-10">
      <nav aria-label={t('settings.title')} className="lg:sticky lg:top-24 lg:w-56 lg:shrink-0 lg:self-start">
        <h1 className="mb-4 text-page font-semibold tracking-[-0.01em]">{t('settings.title')}</h1>
        {/* Phones: one scrolling row. Desktop: grouped column. */}
        <div className="scrollbar-none -mx-4 flex gap-1 overflow-x-auto px-4 lg:mx-0 lg:hidden lg:px-0">
          {all.map((s) => (
            <SectionLink key={s.path} s={s} />
          ))}
        </div>
        <div className="hidden flex-col gap-5 lg:flex">
          {groups.map((g) => (
            <div key={g.label}>
              <p className="mb-1 px-3 text-caption font-semibold uppercase tracking-wider text-subtle">{t(g.label)}</p>
              <div className="flex flex-col gap-0.5">
                {g.sections.map((s) => (
                  <SectionLink key={s.path} s={s} />
                ))}
              </div>
            </div>
          ))}
        </div>
      </nav>
      <div className="min-w-0 flex-1 lg:pt-12">
        <Routes>
          {all.map((s) => (
            <Route key={s.path} path={s.path} element={<div className="anim-enter">{s.element}</div>} />
          ))}
          <Route path="*" element={<Navigate to="dashboard" replace />} />
        </Routes>
      </div>
    </div>
  )
}
