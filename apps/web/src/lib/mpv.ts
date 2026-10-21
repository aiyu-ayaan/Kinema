import { useQuery } from '@tanstack/react-query'

/** The desktop app's mpv bridge; absent in a browser or an older desktop build. */
export const mpvBridge = typeof window !== 'undefined' ? (window.kinema ?? window.desktop)?.mpv : undefined

/** Whether titles can play in mpv on this computer. */
export function useMpv(): boolean {
  const { data } = useQuery({ queryKey: ['mpv-available'], queryFn: () => mpvBridge!.available(), enabled: !!mpvBridge, staleTime: Infinity })
  return !!data
}
