export const isDev = import.meta.env.MODE === 'development' || import.meta.env.DEV

export const storageKeys = {
  token: isDev ? 'kinema.dev.token' : 'kinema.token',
  theme: isDev ? 'kinema.dev.theme' : 'kinema.theme',
  theater: isDev ? 'kinema.dev.theater' : 'kinema.theater',
  device: isDev ? 'kinema.dev.device' : 'kinema.device',
  volume: isDev ? 'kinema.dev.volume' : 'kinema.volume',
  playlist: isDev ? 'kinema.dev.playlist' : 'kinema.playlist',
  playlistQueue: isDev ? 'kinema.dev.playlistQueue' : 'kinema.playlistQueue',
}

export function initializeFlavor(): void {
  const favicon = document.querySelector<HTMLLinkElement>("link[rel*='icon']")
  if (favicon) {
    favicon.href = isDev ? '/favicon-dev.svg' : '/favicon.svg'
  }
}
