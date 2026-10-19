export const isDesktop: boolean =
  typeof window !== 'undefined' && Boolean(window.kinema?.isDesktop || window.desktop?.isDesktop)

export const desktopPlatform: string =
  (typeof window !== 'undefined' && (window.kinema?.platform || window.desktop?.platform)) || ''

/**
 * On Windows and Linux, Electron's titleBarOverlay renders window controls
 * (minimize, maximize, close) in the top-right corner (~138px wide, 35px high).
 * We add right padding to top bars so controls never collide with UI actions.
 */
export const hasTopRightWindowControls: boolean = isDesktop && desktopPlatform !== 'darwin'

/**
 * On macOS, Electron's titleBarStyle: 'hidden' places traffic lights in the top-left corner (~80px wide).
 */
export const hasTopLeftWindowControls: boolean = isDesktop && desktopPlatform === 'darwin'

export function initializeDesktop(): void {
  if (typeof document === 'undefined') return
  if (isDesktop) {
    document.documentElement.dataset.desktop = 'true'
    if (desktopPlatform) {
      document.documentElement.dataset.platform = desktopPlatform
    }
  }
}
