import React from 'react'
import { useParams } from 'react-router-dom'
import { Folder } from '@phosphor-icons/react'

export const LibraryPage: React.FC = () => {
  const { kind } = useParams<{ kind: string }>()

  return (
    <div className="p-8 space-y-6 max-w-7xl mx-auto">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold capitalize">{kind ?? 'Library'}</h1>
      </div>

      <div className="bg-panel border border-border border-dashed rounded-lg p-12 text-center text-subtle flex flex-col items-center justify-center">
        <Folder size={48} className="mb-3 opacity-40 text-primary" />
        <p className="text-base font-medium text-fg">No items found</p>
        <p className="text-sm text-muted mt-1 max-w-md">
          Configure a library root folder in <code className="text-xs bg-raised px-1 py-0.5 rounded">kinema.toml</code> to scan and index media files.
        </p>
      </div>
    </div>
  )
}
