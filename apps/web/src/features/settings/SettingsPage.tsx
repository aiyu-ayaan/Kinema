import { Navigate, Route, Routes, useParams } from 'react-router-dom'
import { PageHeader } from '@/components/ui/Card'
import { t } from '@/lib/i18n'
import { AboutSection, AcknowledgementsSection, UpdatesSection } from '@/features/about/AboutPage'
import { LibrariesSection } from './LibrariesSection'
import { GeneralSection, LanguagesSection, LibrarySection, MetadataSection, ScheduledSection, TranscoderSection } from './ServerSections'
import { settingsGroups, type SectionPath } from './sections'
import { DashboardSection, NetworkSection, TroubleshootingSection } from './StatusSections'
import { ShelvesSection } from './ShelvesSection'
import { UsersSection } from './UsersSection'
import { SharesSection } from '@/features/shares/SharesSection'

const elements: Record<SectionPath, JSX.Element> = {
  dashboard: <DashboardSection />,
  general: <GeneralSection />,
  library: <LibrarySection />,
  metadata: <MetadataSection />,
  languages: <LanguagesSection />,
  transcoder: <TranscoderSection />,
  network: <NetworkSection />,
  scheduled: <ScheduledSection />,
  libraries: <LibrariesSection />,
  shelves: <ShelvesSection />,
  users: <UsersSection />,
  shares: <SharesSection />,
  troubleshooting: <TroubleshootingSection />,
  updates: <UpdatesSection />,
  about: <AboutSection />,
}

const sections = settingsGroups.flatMap((g) => g.sections)

function Section() {
  const path = useParams().section as SectionPath
  const section = sections.find((s) => s.path === path)
  if (!section) return <Navigate to="/settings/dashboard" replace />
  return (
    <div key={path} className="anim-enter">
      <PageHeader title={t(section.label)} />
      {elements[path]}
    </div>
  )
}

/** Admin settings; the section list lives in the app sidebar (`SettingsNav`). */
export function SettingsPage() {
  return (
    <div className="mx-auto max-w-5xl">
      <Routes>
        <Route path="about/acknowledgements" element={<AcknowledgementsSection />} />
        <Route path=":section" element={<Section />} />
        <Route path="*" element={<Navigate to="dashboard" replace />} />
      </Routes>
    </div>
  )
}
