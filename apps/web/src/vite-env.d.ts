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
  /** From the root package.json (vite.config.ts). */
  const __APP_VERSION__: string
  const __APP_BUILD__: number
  interface Window {
    kinema?: DesktopBridge
    desktop?: DesktopBridge
  }
}
