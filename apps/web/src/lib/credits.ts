import { t } from '@/lib/i18n'

export const REPO = 'aiyu-ayaan/Kinema'
export const REPO_URL = `https://github.com/${REPO}`
export const DEVELOPER = { name: 'aiyu-ayaan', github: 'https://github.com/aiyu-ayaan', site: 'https://aiyu.co.in' }

export interface Credit {
  name: string
  url: string
  license: string
}

/** Keep in sync with apps/web/package.json and the Cargo.toml files. The Android app keeps its own list (ui/AboutScreens.kt). */
export const credits: { group: string; items: Credit[] }[] = [
  {
    group: t('credits.group.interface'),
    items: [
      { name: 'React', url: 'https://github.com/facebook/react', license: 'MIT' },
      { name: 'React Router', url: 'https://github.com/remix-run/react-router', license: 'MIT' },
      { name: 'TanStack Query', url: 'https://github.com/TanStack/query', license: 'MIT' },
      { name: 'Tailwind CSS', url: 'https://github.com/tailwindlabs/tailwindcss', license: 'MIT' },
      { name: 'hls.js', url: 'https://github.com/video-dev/hls.js', license: 'Apache-2.0' },
      { name: 'Phosphor Icons', url: 'https://github.com/phosphor-icons/react', license: 'MIT' },
      { name: 'Lucide', url: 'https://github.com/lucide-icons/lucide', license: 'ISC' },
      { name: 'Inter', url: 'https://github.com/rsms/inter', license: 'OFL-1.1' },
      { name: 'TypeScript', url: 'https://github.com/microsoft/TypeScript', license: 'Apache-2.0' },
      { name: 'Vite', url: 'https://github.com/vitejs/vite', license: 'MIT' },
      { name: 'Electron', url: 'https://github.com/electron/electron', license: 'MIT' },
      { name: 'electron-builder', url: 'https://github.com/electron-userland/electron-builder', license: 'MIT' },
    ],
  },
  {
    group: t('credits.group.server'),
    items: [
      { name: 'Rust', url: 'https://github.com/rust-lang/rust', license: 'MIT / Apache-2.0' },
      { name: 'Tokio', url: 'https://github.com/tokio-rs/tokio', license: 'MIT' },
      { name: 'axum and tower-http', url: 'https://github.com/tokio-rs/axum', license: 'MIT' },
      { name: 'axum-server', url: 'https://github.com/programatik29/axum-server', license: 'MIT' },
      { name: 'SQLx', url: 'https://github.com/launchbadge/sqlx', license: 'MIT / Apache-2.0' },
      { name: 'SQLite', url: 'https://www.sqlite.org', license: 'Public domain' },
      { name: 'rustls', url: 'https://github.com/rustls/rustls', license: 'Apache-2.0 / ISC / MIT' },
      { name: 'rcgen', url: 'https://github.com/rustls/rcgen', license: 'MIT / Apache-2.0' },
      { name: 'reqwest', url: 'https://github.com/seanmonstar/reqwest', license: 'MIT / Apache-2.0' },
      { name: 'lettre', url: 'https://github.com/lettre/lettre', license: 'MIT / Apache-2.0' },
      { name: 'notify', url: 'https://github.com/notify-rs/notify', license: 'CC0-1.0' },
      { name: 'walkdir', url: 'https://github.com/BurntSushi/walkdir', license: 'MIT / Unlicense' },
      { name: 'argon2 and sha2 (RustCrypto)', url: 'https://github.com/RustCrypto', license: 'MIT / Apache-2.0' },
      { name: 'serde', url: 'https://github.com/serde-rs/serde', license: 'MIT / Apache-2.0' },
      { name: 'chrono', url: 'https://github.com/chronotope/chrono', license: 'MIT / Apache-2.0' },
      { name: 'libc', url: 'https://github.com/rust-lang/libc', license: 'MIT / Apache-2.0' },
      { name: 'tracing', url: 'https://github.com/tokio-rs/tracing', license: 'MIT' },
      { name: 'FFmpeg', url: 'https://ffmpeg.org', license: 'GPL (bundled build, run as a separate program)' },
      { name: 'mpv', url: 'https://github.com/mpv-player/mpv', license: 'GPL-2.0+ / LGPL-2.1+ (bundled with the desktop app, run as a separate program; notice in resources/mpv/NOTICE-mpv.txt)' },
    ],
  },
  {
    group: t('credits.group.data'),
    items: [
      { name: 'TMDB', url: 'https://www.themoviedb.org', license: t('credits.tmdb') },
      { name: 'GitHub Releases', url: 'https://docs.github.com/rest/releases', license: t('credits.github') },
    ],
  },
  {
    group: t('credits.group.inspiration'),
    items: [{ name: 'Plex Media Server', url: 'https://www.plex.tv', license: t('credits.plex') }],
  },
]
