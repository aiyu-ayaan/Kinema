import React from 'react'
import { MagnifyingGlass, User } from '@phosphor-icons/react'
import { isDev } from '../lib/flavor'

export const Header: React.FC = () => {
  return (
    <header className="h-16 px-6 bg-panel/75 backdrop-blur-md border-b border-border flex items-center justify-between sticky top-0 z-20">
      <div className="flex items-center gap-4 flex-1 max-w-md">
        <div className="relative w-full">
          <MagnifyingGlass
            size={18}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-subtle"
          />
          <input
            type="text"
            placeholder="Search movies, TV shows, music..."
            className="w-full bg-raised border border-border rounded-md pl-10 pr-4 py-1.5 text-sm placeholder:text-subtle focus:outline-none focus:border-primary transition-colors"
          />
        </div>
        {isDev && (
          <span className="bg-amber-500/10 text-amber-500 border border-amber-500/20 text-[11px] font-bold px-2 py-0.5 rounded tracking-wide">
            DEV
          </span>
        )}
      </div>

      <div className="flex items-center gap-3">
        <button
          type="button"
          className="w-9 h-9 rounded-full bg-raised border border-border flex items-center justify-center text-muted hover:text-fg hover:border-border-strong transition-colors"
          title="Profile"
        >
          <User size={18} />
        </button>
      </div>
    </header>
  )
}
