import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from 'react'
import { Link, NavLink, useLocation, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { GearSix, House, List, MagnifyingGlass, Moon, Sun, UserSwitch, X, type Icon } from '@phosphor-icons/react'
import { Avatar } from './ui/Avatar'
import { SettingsNav } from '@/features/settings/SettingsNav'
import { libraryIcon } from './media/libraryIcon'
import { Button } from './ui/Button'
import { api } from '@/lib/api'
import { useAuth, useIsAdmin } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { isDev } from '@/lib/flavor'
import { t } from '@/lib/i18n'
import { useLibraries, useServerInfo } from '@/lib/queries'
import { setTheme, useTheme } from '@/lib/theme'
import type { Library } from '@/lib/types'

function NavItem({ to, icon: I, label, end, onClick }: { to: string; icon: Icon; label: string; end?: boolean; onClick?: () => void }) {
  return (
    <NavLink
      to={to}
      end={end}
      onClick={onClick}
      className={({ isActive }) =>
        cn(
          'flex h-9 items-center gap-3 rounded-md px-3 text-body font-medium transition-colors duration-fast',
          isActive ? 'bg-primary-container text-on-primary-container' : 'text-muted hover:bg-raised hover:text-fg',
        )
      }
    >
      {({ isActive }) => (
        <>
          <I size={18} weight={isActive ? 'fill' : 'regular'} aria-hidden />
          <span className="truncate">{label}</span>
        </>
      )}
    </NavLink>
  )
}

const navCls = (active: boolean) =>
  cn(
    'flex h-9 items-center gap-3 rounded-md px-3 text-body font-medium transition-colors duration-fast',
    active ? 'bg-primary-container text-on-primary-container' : 'text-muted hover:bg-raised hover:text-fg',
  )

/** A library, with Movies as the library and actual TV shows listed directly. */
function LibraryNav({ library: l, onNavigate }: { library: Library; onNavigate?: () => void }) {
  const location = useLocation()
  const here = location.pathname === `/library/${l.id}`
  const I = libraryIcon[l.kind]

  const { data: items } = useQuery({
    queryKey: ['library', l.id],
    queryFn: () => api.libraryItems(l.id),
    enabled: l.show_count > 0,
  })

  const shows = useMemo(() => {
    if (!items) return []
    return items.filter((i) => i.kind === 'show')
  }, [items])

  const isGenericShowLib =
    l.kind === 'show' &&
    (l.name.toLowerCase() === 'tv shows' || l.name.toLowerCase() === t('library.kind.show').toLowerCase())

  return (
    <>
      {!isGenericShowLib && (
        <Link
          to={`/library/${l.id}`}
          onClick={onNavigate}
          aria-current={here ? 'page' : undefined}
          className={navCls(here)}
        >
          <I size={18} weight={here ? 'fill' : 'regular'} aria-hidden className="shrink-0" />
          <span className="truncate">{l.name}</span>
        </Link>
      )}
      {shows.map((show) => {
        const active = location.pathname === `/item/${show.id}`
        const ShowIcon = libraryIcon.show
        return (
          <Link
            key={show.id}
            to={`/item/${show.id}`}
            onClick={onNavigate}
            aria-current={active ? 'page' : undefined}
            className={navCls(active)}
            title={show.title}
          >
            <ShowIcon size={18} weight={active ? 'fill' : 'regular'} aria-hidden className="shrink-0" />
            <span className="truncate">{show.title}</span>
          </Link>
        )
      })}
    </>
  )
}

function Brand() {
  return (
    <div className="flex h-16 items-center gap-2.5 px-5">
      <img src={isDev ? '/favicon-dev.svg' : '/favicon.svg'} alt="" className="h-7 w-7" />
      <span className="text-title font-semibold tracking-[-0.01em]">{t('app.name')}</span>
      {isDev && <span className="rounded border border-info/30 bg-info/10 px-1.5 text-[11px] font-bold tracking-wide text-info">{t('app.dev')}</span>}
    </div>
  )
}

function Nav({ onNavigate }: { onNavigate?: () => void }) {
  const { data: libraries } = useLibraries()
  const admin = useIsAdmin()
  return (
    <nav className="flex flex-1 flex-col gap-1 overflow-y-auto px-3 pb-3">
      <NavItem to="/" end icon={House} label={t('nav.home')} onClick={onNavigate} />
      {libraries && libraries.length > 0 && (
        <p className="mt-4 px-3 pb-1 text-caption font-semibold uppercase tracking-wider text-subtle">{t('nav.libraries')}</p>
      )}
      {libraries?.map((l) => (
        <LibraryNav key={l.id} library={l} onNavigate={onNavigate} />
      ))}
      {admin && (
        <div className="mt-auto pt-4">
          <NavItem to="/settings" icon={GearSix} label={t('nav.settings')} onClick={onNavigate} />
        </div>
      )}
    </nav>
  )
}

function SearchBox() {
  const navigate = useNavigate()
  const location = useLocation()
  const [q, setQ] = useState(() => new URLSearchParams(location.search).get('q') ?? '')
  function submit(e: FormEvent) {
    e.preventDefault()
    if (q.trim()) navigate(`/search?q=${encodeURIComponent(q.trim())}`)
  }
  return (
    <form onSubmit={submit} role="search" className="relative w-full max-w-md">
      <MagnifyingGlass size={16} className="pointer-events-none absolute start-3 top-1/2 -translate-y-1/2 text-subtle" aria-hidden />
      <input
        type="search"
        value={q}
        onChange={(e) => setQ(e.target.value)}
        placeholder={t('search.placeholder')}
        aria-label={t('search.placeholder')}
        className="h-9 w-full rounded-full border border-border bg-raised ps-9 pe-4 text-body placeholder:text-subtle focus:border-border-strong focus:outline-none focus:ring-2 focus:ring-primary/25"
      />
    </form>
  )
}

function UserMenu() {
  const { user, signOut } = useAuth()
  const theme = useTheme()
  const dark = theme === 'dark' || (theme === 'system' && !window.matchMedia('(prefers-color-scheme: light)').matches)
  return (
    <div className="flex items-center gap-1">
      <Button
        variant="ghost"
        onClick={() => setTheme(dark ? 'light' : 'dark')}
        aria-label={t(dark ? 'theme.light' : 'theme.dark')}
        title={t(dark ? 'theme.light' : 'theme.dark')}
        icon={dark ? <Sun size={18} /> : <Moon size={18} />}
      />
      <span className="hidden h-8 items-center gap-2 rounded-full border border-border bg-raised ps-1 pe-3 text-label font-medium sm:flex">
        {user && <Avatar name={user.name} color={user.avatar} size={24} />}
        {user?.name}
      </span>
      {/* Signing out returns to the profile picker, which is how profiles switch. */}
      <Button variant="ghost" onClick={signOut} aria-label={t('auth.switchProfile')} title={t('auth.switchProfile')} icon={<UserSwitch size={18} />} />
    </div>
  )
}

export function AppShell({ children }: { children: ReactNode }) {
  useServerInfo()
  const [drawer, setDrawer] = useState(false)
  const location = useLocation()
  useEffect(() => setDrawer(false), [location.pathname])
  // Settings takes over the sidebar, so the page itself is all content.
  const inSettings = location.pathname.startsWith('/settings')
  return (
    <div className="flex min-h-dvh">
      <aside className="sticky top-0 hidden h-dvh w-60 shrink-0 flex-col border-e border-border bg-panel md:flex">
        <Brand />
        {inSettings ? <SettingsNav /> : <Nav />}
      </aside>
      {drawer && (
        <div className="fixed inset-0 z-sheet md:hidden" role="dialog" aria-modal>
          <div className="anim-fade absolute inset-0 bg-black/60" onClick={() => setDrawer(false)} />
          <aside className="anim-enter absolute inset-y-0 start-0 flex w-72 flex-col border-e border-border bg-panel">
            <div className="flex items-center justify-between pe-3">
              <Brand />
              <Button variant="ghost" onClick={() => setDrawer(false)} aria-label={t('common.close')} icon={<X size={18} />} />
            </div>
            {inSettings ? <SettingsNav onNavigate={() => setDrawer(false)} /> : <Nav onNavigate={() => setDrawer(false)} />}
          </aside>
        </div>
      )}
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="chrome sticky top-0 z-nav flex h-16 items-center gap-3 border-b border-border px-4 md:px-8">
          <Button variant="ghost" className="md:hidden" onClick={() => setDrawer(true)} aria-label={t('nav.menu')} icon={<List size={20} />} />
          <SearchBox />
          <div className="ms-auto">
            <UserMenu />
          </div>
        </header>
        <main className="flex-1 px-4 py-6 md:px-8 md:py-8">{children}</main>
      </div>
    </div>
  )
}
