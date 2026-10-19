import React from 'react'
import { Link, useLocation } from 'react-router-dom'
import { House, FilmStrip, Television, MusicNotes, Folder, Plus, Icon } from '@phosphor-icons/react'
import clsx from 'clsx'

interface NavItem {
  label: string
  to: string
  icon: Icon
}

const navItems: NavItem[] = [
  { label: 'Home', to: '/', icon: House },
  { label: 'Movies', to: '/library/movies', icon: FilmStrip },
  { label: 'TV Shows', to: '/library/shows', icon: Television },
  { label: 'Music', to: '/library/music', icon: MusicNotes },
  { label: 'Other Videos', to: '/library/other', icon: Folder },
]

export const Sidebar: React.FC = () => {
  const location = useLocation()

  return (
    <aside className="w-60 bg-panel border-r border-border flex flex-col justify-between shrink-0 h-screen sticky top-0">
      <div>
        <div className="h-16 px-6 flex items-center gap-3 border-b border-border">
          <div className="w-8 h-8 rounded-md bg-primary flex items-center justify-center text-on-primary font-bold text-lg">
            K
          </div>
          <span className="font-semibold text-lg tracking-tight">Kinema</span>
        </div>

        <nav className="p-3 space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-subtle uppercase tracking-wider">
            Libraries
          </div>
          {navItems.map((item) => {
            const Icon = item.icon
            const active = location.pathname === item.to
            return (
              <Link
                key={item.to}
                to={item.to}
                className={clsx(
                  'flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  active
                    ? 'bg-raised text-primary'
                    : 'text-muted hover:text-fg hover:bg-raised/50'
                )}
              >
                <Icon size={20} />
                <span>{item.label}</span>
              </Link>
            )
          })}
        </nav>
      </div>

      <div className="p-3 border-t border-border">
        <button
          type="button"
          className="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-muted hover:text-fg rounded-md hover:bg-raised transition-colors"
        >
          <Plus size={16} />
          <span>Add Library</span>
        </button>
      </div>
    </aside>
  )
}
