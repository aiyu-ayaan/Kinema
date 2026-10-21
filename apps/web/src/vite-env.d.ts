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
  /** Plays in mpv (apps/desktop/src/mpv.js). Absent in older desktop builds. */
  mpv?: {
    /** Whether mpv is installed on this computer. */
    available(): Promise<boolean>
    /** Resolves when mpv closes. `aid` is 1-based among audio tracks; `subtitleId` a text subtitle stream id. */
    play(options: { fileId: number; start: number; title: string; aid: number | null; subtitleId: number | null }): Promise<{ position?: number; ended?: boolean; error?: string }>
    stop(): void
    /** The position every 10 s while playing. Returns the function that stops listening. */
    onProgress(listener: (position: number) => void): () => void
  }
  /** The desktop app's own releases (apps/desktop/src/updates.js). Absent in older desktop builds. */
  updates?: {
    info(): Promise<{ version: string; channel: string; selfInstall: boolean; skipped: string | null } | null>
    check(): Promise<{ version: string; page: string } | null>
    /** The reason it failed, or '' (the app restarts, or the installer was opened). */
    install(): Promise<string>
    /** Don't offer the found version at start again. */
    skip(): Promise<void>
    /** The download's progress, 0..1, then null. Returns the function that stops listening. */
    onProgress(listener: (fraction: number | null) => void): () => void
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
