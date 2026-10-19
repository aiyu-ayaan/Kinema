import React from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchServerInfo } from '../../lib/api'
import { FilmStrip, PlayCircle, Television } from '@phosphor-icons/react'

export const HomePage: React.FC = () => {
  const { data: serverInfo, isLoading, error } = useQuery({
    queryKey: ['serverInfo'],
    queryFn: fetchServerInfo,
    retry: 1,
  })

  return (
    <div className="p-8 space-y-10 max-w-7xl mx-auto">
      {/* Server banner status */}
      <div className="bg-raised/60 border border-border rounded-lg p-6 flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Kinema Media Server</h1>
          <p className="text-sm text-muted mt-1">
            {isLoading
              ? 'Connecting to backend...'
              : error
              ? 'Backend is offline or starting up'
              : `${serverInfo?.name} v${serverInfo?.version}`}
          </p>
        </div>
        {serverInfo && (
          <div className="flex flex-wrap gap-2">
            {serverInfo.capabilities.map((cap) => (
              <span
                key={cap}
                className="bg-panel border border-border-strong text-xs font-mono text-muted px-2.5 py-1 rounded"
              >
                {cap}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Continue Watching Section */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <PlayCircle size={20} className="text-primary" />
            Continue Watching
          </h2>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
          <div className="aspect-[2/3] bg-panel border border-border border-dashed rounded-lg flex flex-col items-center justify-center p-4 text-center text-subtle">
            <FilmStrip size={32} className="mb-2 opacity-50" />
            <span className="text-xs">No media currently in progress</span>
          </div>
        </div>
      </section>

      {/* Recently Added Section */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Television size={20} className="text-primary" />
            Recently Added
          </h2>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
          <div className="aspect-[2/3] bg-panel border border-border border-dashed rounded-lg flex flex-col items-center justify-center p-4 text-center text-subtle">
            <Television size={32} className="mb-2 opacity-50" />
            <span className="text-xs">Add a library to start scanning files</span>
          </div>
        </div>
      </section>
    </div>
  )
}
