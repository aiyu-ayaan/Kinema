import { describeDevice } from './deviceName'
import { desktopPlatform, isDesktop } from './desktop'
import { storageKeys } from './flavor'
import type { DeviceInfo } from './types'

/** `crypto.randomUUID` only exists on secure origins, and a server on the LAN is plain http. */
function randomId(): string {
  if (typeof crypto.randomUUID === 'function') return crypto.randomUUID()
  const bytes = crypto.getRandomValues(new Uint8Array(16))
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}

let fallback: string | undefined

/** Random id kept on this device, so the server can tell it apart from the profile's other devices. */
function deviceId(): string {
  try {
    const stored = localStorage.getItem(storageKeys.device)
    if (stored) return stored
    const id = randomId()
    localStorage.setItem(storageKeys.device, id)
    return id
  } catch {
    // Private mode: a new id each page load counts as a new device, so keep one for this load.
    return (fallback ??= randomId())
  }
}

/** What sign-in tells the server about this client. */
export function clientDevice(): DeviceInfo {
  return {
    id: deviceId(),
    kind: isDesktop ? 'desktop' : 'web',
    name: describeDevice(navigator.userAgent, isDesktop, desktopPlatform),
  }
}
