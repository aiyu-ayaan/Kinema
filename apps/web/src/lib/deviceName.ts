/** Short, human name for this client: "Chrome on Windows", "Kinema desktop (linux)". */
export function describeDevice(userAgent: string, desktop: boolean, platform: string): string {
  if (desktop) return platform ? `Kinema desktop (${platform})` : 'Kinema desktop'
  // Order matters: Edge and Opera also say Chrome, Chrome also says Safari.
  const browser = /Edg\//.test(userAgent)
    ? 'Edge'
    : /OPR\/|Opera/.test(userAgent)
      ? 'Opera'
      : /Firefox\//.test(userAgent)
        ? 'Firefox'
        : /Chrome\/|CriOS\//.test(userAgent)
          ? 'Chrome'
          : /Safari\//.test(userAgent)
            ? 'Safari'
            : 'Browser'
  // iPhone UAs also say "Mac OS X", and Android ones say "Linux".
  const os = /Android/.test(userAgent)
    ? 'Android'
    : /iPhone|iPad|iPod/.test(userAgent)
      ? 'iOS'
      : /Windows/.test(userAgent)
        ? 'Windows'
        : /Mac OS X|Macintosh/.test(userAgent)
          ? 'macOS'
          : /CrOS/.test(userAgent)
            ? 'ChromeOS'
            : /Linux|X11/.test(userAgent)
              ? 'Linux'
              : ''
  return os ? `${browser} on ${os}` : browser
}
