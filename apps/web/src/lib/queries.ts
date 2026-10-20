import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from './api'

/** Libraries poll every 2 s while a scan or thumbnail job runs, so progress shows as it happens. */
export function useLibraries() {
  return useQuery({
    queryKey: ['libraries'],
    queryFn: api.libraries,
    refetchInterval: (q) => (q.state.data?.some((l) => l.scanning || l.generating_thumbnails) ? 2000 : false),
  })
}

export function useScanning() {
  return useLibraries().data?.some((l) => l.scanning) ?? false
}

/** Public server info; also names the browser tab after the server. Re-read on focus so a rename made elsewhere shows up. */
export function useServerInfo() {
  const q = useQuery({ queryKey: ['server-info'], queryFn: api.serverInfo, staleTime: 5 * 60_000, refetchOnWindowFocus: 'always' })
  useEffect(() => {
    if (q.data?.name) document.title = q.data.name
  }, [q.data?.name])
  return q
}
