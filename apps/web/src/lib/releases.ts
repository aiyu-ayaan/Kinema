import { useQuery } from '@tanstack/react-query'
import { REPO } from './credits'
import { useServerInfo } from './queries'

export type Channel = 'stable' | 'beta' | 'alpha'

// The release workflow names versions 0.0.2-alpha.1, 0.0.2-beta.1 and 0.0.2 (scripts/release-version.mjs).
const VERSION = /^v?(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta)\.(\d+))?$/
const STAGE: Record<Channel, number> = { alpha: 0, beta: 1, stable: 2 }

/** `1.2.3-beta.4` as [major, minor, patch, stage, number]; null for any other shape. */
function versionKey(text: string): number[] | null {
  const m = VERSION.exec(text.trim())
  // A stable release has no pre-release number and still sorts above every one of its pre-releases.
  return m && [+m[1], +m[2], +m[3], STAGE[(m[4] as Channel | undefined) ?? 'stable'], m[5] ? +m[5] : Infinity]
}

const newer = (a: number[], b: number[]) => {
  const i = a.findIndex((part, n) => part !== b[n])
  return i !== -1 && a[i] > b[i]
}

/** The channel a build belongs to: whoever installed an alpha wants alphas, and stable never gets offered one. */
export const channelOf = (version: string): Channel =>
  (Object.keys(STAGE) as Channel[]).find((c) => STAGE[c] === versionKey(version)?.[3]) ?? 'stable'

interface GithubRelease {
  tag_name: string
  html_url: string
  draft?: boolean
  assets?: { name: string }[]
}

export interface ServerUpdate {
  version: string
  url: string
  channel: Channel
}

/** The newest release on the running version's channel (or a steadier one) that is newer and ships a server bundle. */
export function pickServerUpdate(releases: GithubRelease[], running: string): ServerUpdate | null {
  const current = versionKey(running)
  if (!current) return null
  const channel = channelOf(running)
  let best: { key: number[]; release: GithubRelease } | null = null
  for (const release of releases) {
    const key = versionKey(release.tag_name)
    if (release.draft || !key || key[3] < STAGE[channel] || !newer(key, best?.key ?? current)) continue
    if (!release.assets?.some((a) => a.name.startsWith('kinema-server-'))) continue
    best = { key, release }
  }
  return best && { version: best.release.tag_name.replace(/^v/, ''), url: best.release.html_url, channel }
}

/**
 * Admins only: a newer server release on GitHub. Asked by the browser (unauthenticated, 60 requests an hour per
 * address), so the server itself never phones home.
 */
export function useServerUpdate(enabled: boolean) {
  const running = useServerInfo().data?.version
  return useQuery({
    queryKey: ['server-update', running],
    enabled: enabled && !!running,
    staleTime: 6 * 60 * 60_000,
    retry: false,
    queryFn: async () => {
      const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=30`, { headers: { Accept: 'application/vnd.github+json' } })
      if (!res.ok) throw new Error(`GitHub answered ${res.status}`)
      return pickServerUpdate(await res.json(), running!)
    },
  })
}

const INSTALL = `curl -fsSL https://raw.githubusercontent.com/${REPO}/master/scripts/install-server.sh | sudo sh`

/** How to update each kind of server install; the same commands as the README's "Updating". */
export const updateCommands = (channel: Channel) => [
  { key: 'linux', command: channel === 'stable' ? INSTALL : `${INSTALL} -s -- --channel ${channel}` },
  { key: 'docker', command: 'docker compose pull && docker compose up -d' },
] as const
