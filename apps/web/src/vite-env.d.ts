/// <reference types="vite/client" />

export interface DesktopBridge {
  isDesktop: boolean
  platform: string
  flavor: 'dev' | 'prod'
  chooseServer: () => void
  setTheme: (theme: string) => void
  token?: {
    get: () => string | null
    set: (token: string | null) => void
  }
}

declare global {
  interface Window {
    kinema?: DesktopBridge
    desktop?: DesktopBridge
  }
}
