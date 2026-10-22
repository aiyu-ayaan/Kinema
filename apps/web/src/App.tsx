import { lazy, Suspense } from 'react'
import { Navigate, Route, Routes } from 'react-router-dom'
import { CircleNotch } from '@phosphor-icons/react'
import { AppShell } from './components/AppShell'
import { AuthScreen } from './features/auth/AuthScreen'
import { HomePage } from './features/home/HomePage'
import { ItemPage } from './features/item/ItemPage'
import { LibraryPage } from './features/library/LibraryPage'
import { CollectionPage } from './features/playlists/CollectionPage'
import { PlaylistPage } from './features/playlists/PlaylistPage'
import { PlaylistsPage } from './features/playlists/PlaylistsPage'
import { SearchPage } from './features/search/SearchPage'
import { SettingsPage } from './features/settings/SettingsPage'
import { useAuth } from './lib/auth'
import { t } from './lib/i18n'

// The player is the heaviest screen; load it only when something plays.
const PlayerPage = lazy(() => import('./features/player/PlayerPage'))

function Splash() {
  return (
    <div className="flex min-h-dvh items-center justify-center">
      <CircleNotch size={32} className="animate-spin text-primary" aria-label={t('common.loading')} />
    </div>
  )
}

export function App() {
  const { user, loading } = useAuth()
  if (loading) return <Splash />
  if (!user) return <AuthScreen />
  return (
    <Routes>
      <Route
        path="/play/:id"
        element={
          <Suspense fallback={<Splash />}>
            <PlayerPage />
          </Suspense>
        }
      />
      <Route
        path="*"
        element={
          <AppShell>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/library/:id" element={<LibraryPage />} />
              <Route path="/item/:id" element={<ItemPage />} />
              <Route path="/search" element={<SearchPage />} />
              <Route path="/playlists" element={<PlaylistsPage />} />
              <Route path="/playlists/:id" element={<PlaylistPage />} />
              <Route path="/collections/:id" element={<CollectionPage />} />
              {user.is_admin && <Route path="/settings/*" element={<SettingsPage />} />}
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </AppShell>
        }
      />
    </Routes>
  )
}
