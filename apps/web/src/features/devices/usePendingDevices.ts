import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'

/** Admin: the device list, polled so a new request shows up without a reload. */
export function useDevices(enabled = true) {
  return useQuery({ queryKey: ['devices'], queryFn: api.devices, refetchInterval: 10_000, enabled })
}

export function usePendingDevices(enabled = true) {
  return useDevices(enabled).data?.filter((d) => d.status === 'pending').length ?? 0
}
