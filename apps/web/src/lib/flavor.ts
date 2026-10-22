export const isDev = import.meta.env.MODE === 'development' || import.meta.env.DEV

export const storageKeys = {
  token: isDev ? 'kinema.dev.token' : 'kinema.token',
  theme: isDev ? 'kinema.dev.theme' : 'kinema.theme',
  theater: isDev ? 'kinema.dev.theater' : 'kinema.theater',
  playlist: isDev ? 'kinema.dev.playlist' : 'kinema.playlist',
}

export function initializeFlavor(): void {
  const favicon = document.querySelector<HTMLLinkElement>("link[rel*='icon']")
  if (favicon) {
    favicon.href = isDev ? '/favicon-dev.svg' : '/favicon.svg'
  }
}
