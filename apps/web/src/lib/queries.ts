import { useQuery } from '@tanstack/react-query'
import { api } from './api'

/** Libraries poll every 2 s while any scan runs, so new items appear as they are found. */
export function useLibraries() {
  return useQuery({
    queryKey: ['libraries'],
    queryFn: api.libraries,
    refetchInterval: (q) => (q.state.data?.some((l) => l.scanning) ? 2000 : false),
  })
}

export function useScanning() {
  return useLibraries().data?.some((l) => l.scanning) ?? false
}
