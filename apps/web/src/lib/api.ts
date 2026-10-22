import { storageKeys } from './flavor'
import type {
  DirListing,
  MatchCandidate,
  Home,
  ItemCard,
  ItemDetail,
  Library,
  LibraryKind,
  ManagedUser,
  CreatedShare,
  ImportResult,
  Languages,
  MyProfile,
  Prefs,
  ProfileCard,
  ProfileInput,
  PlaybackPlan,
  Share,
  ShareView,
  ServerOptions,
  ServerStatus,
  SettingsView,
  User,
} from './types'

export interface ServerInfo {
  name: string
  version: string
  public_url: string
  /** SHA-256 of the server's own certificate; '' without TLS. */
  tls_fingerprint: string
  capabilities: string[]
}

/** The session token for API calls. Media URLs ride on the HttpOnly cookie set at sign-in. */
export const session = {
  get token(): string | null {
    try {
      return localStorage.getItem(storageKeys.token)
    } catch {
      return null
    }
  },
  set(token: string | null) {
    try {
      if (token) localStorage.setItem(storageKeys.token, token)
      else localStorage.removeItem(storageKeys.token)
    } catch {
      /* private mode: the session lasts for this tab only via the cookie */
    }
  },
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message)
  }
}

let onUnauthorized: () => void = () => {}
export function setUnauthorizedHandler(fn: () => void) {
  onUnauthorized = fn
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {}
  const token = session.token
  if (token) headers.Authorization = `Bearer ${token}`
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  const res = await fetch(`/api${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  if (!res.ok) {
    const err = await res.json().catch(() => null)
    if (res.status === 401 && token) onUnauthorized()
    throw new ApiError(res.status, err?.error?.code ?? 'unknown', err?.error?.message ?? res.statusText)
  }
  if (res.status === 204 || res.status === 202) return undefined as T
  return res.json()
}

interface SessionResponse {
  token: string
  user: User
}

export interface PlayCaps {
  hevc: boolean
  av1: boolean
}

function playQuery(audio: number | null, caps: PlayCaps, start?: number) {
  const q = new URLSearchParams()
  if (audio != null) q.set('audio', String(audio))
  if (caps.hevc) q.set('hevc', 'true')
  if (caps.av1) q.set('av1', 'true')
  if (start) q.set('start', start.toFixed(6))
  return q.toString()
}

export const api = {
  serverInfo: () => request<ServerInfo>('GET', '/server/info'),
  authStatus: () => request<{ needs_setup: boolean }>('GET', '/auth/status'),
  setup: (name: string, password: string) => request<SessionResponse>('POST', '/auth/setup', { name, password }),
  profiles: () => request<ProfileCard[]>('GET', '/auth/profiles'),
  myProfile: () => request<MyProfile>('GET', '/profile'),
  setLanguages: (l: Languages) => request<Prefs>('PUT', '/profile/preferences', l),
  exportProfile: () => request<unknown>('GET', '/profile/export'),
  importProfile: (data: unknown) => request<ImportResult>('POST', '/profile/import', data),
  exportUser: (id: number, includeLock: boolean) =>
    request<unknown>('GET', `/users/${id}/export${includeLock ? '?include_lock=true' : ''}`),
  importUser: (id: number, data: unknown, applyLock: boolean) =>
    request<ImportResult>('POST', `/users/${id}/import${applyLock ? '?apply_lock=true' : ''}`, data),
  setContentRating: (id: number, content_rating: string | null) =>
    request<void>('PUT', `/items/${id}/content-rating`, { content_rating }),
  shares: () => request<Share[]>('GET', '/shares'),
  createShare: (body: { item_id: number; expires_in_hours: number; label?: string }) =>
    request<CreatedShare>('POST', '/shares', body),
  revokeShare: (id: number) => request<void>('DELETE', `/shares/${id}`),
  login: (body: { user_id: number; password?: string; pin?: string }) => request<SessionResponse>('POST', '/auth/login', body),
  users: () => request<ManagedUser[]>('GET', '/users'),
  createUser: (p: ProfileInput) => request<ManagedUser>('POST', '/users', p),
  updateUser: (id: number, p: ProfileInput) => request<ManagedUser>('PATCH', `/users/${id}`, p),
  deleteUser: (id: number) => request<void>('DELETE', `/users/${id}`),
  logout: () => request<void>('POST', '/auth/logout'),
  me: () => request<User>('GET', '/auth/me'),

  libraries: () => request<Library[]>('GET', '/libraries'),
  createLibrary: (body: { name: string; kind: LibraryKind; paths: string[] }) => request<Library>('POST', '/libraries', body),
  deleteLibrary: (id: number) => request<void>('DELETE', `/libraries/${id}`),
  scanLibrary: (id: number) => request<void>('POST', `/libraries/${id}/scan`),
  libraryItems: (id: number) => request<ItemCard[]>('GET', `/libraries/${id}/items`),
  browse: (path: string) => request<DirListing>('GET', `/fs?path=${encodeURIComponent(path)}`),

  home: () => request<Home>('GET', '/home'),
  search: (q: string) => request<ItemCard[]>('GET', `/search?q=${encodeURIComponent(q)}`),
  item: (id: number) => request<ItemDetail>('GET', `/items/${id}`),
  progress: (id: number, position_s: number, watched = false) =>
    request<void>('PUT', `/items/${id}/progress`, { position_s, watched }),
  setWatched: (id: number, watched: boolean) => request<void>('PUT', `/items/${id}/watched`, { watched }),
  /** This profile's own audio / subtitle pick (positions; subtitle -1 is off), shared with its other devices. */
  setTrackChoice: (id: number, audio: number | null, subtitle: number | null) =>
    request<void>('PUT', `/items/${id}/track-choice`, { audio, subtitle }),
  setSeriesTracks: (id: number, audio: number | null, subtitle: number | null) =>
    request<void>('PUT', `/items/${id}/tracks`, { audio, subtitle }),

  matchSearch: (id: number, q: string, year: number | null) =>
    request<MatchCandidate[]>('GET', `/items/${id}/match?q=${encodeURIComponent(q)}${year ? `&year=${year}` : ''}`),
  matchApply: (id: number, tmdb_id: number) => request<void>('POST', `/items/${id}/match`, { tmdb_id }),

  plan: (fileId: number, audio: number | null, caps: PlayCaps, start: number) =>
    request<PlaybackPlan>('GET', `/files/${fileId}/plan?${playQuery(audio, caps, start)}`),
  /** What the server picks (audio, subtitle) for a player that takes the file as it is. */
  rawPlan: (fileId: number) => request<PlaybackPlan>('GET', `/files/${fileId}/plan?raw=true`),
  streamUrl: (fileId: number, audio: number | null, caps: PlayCaps, start: number) =>
    `/api/stream/${fileId}?${playQuery(audio, caps, start)}`,
  subtitleText: async (streamId: number) => {
    const res = await fetch(`/api/subtitles/${streamId}`)
    if (!res.ok) throw new ApiError(res.status, 'subtitle', res.statusText)
    return res.text()
  },

  settings: () => request<SettingsView>('GET', '/settings'),
  saveServerSettings: (s: ServerOptions) => request<SettingsView>('PUT', '/settings/server', s),
  saveTmdbKey: (tmdb_api_key: string) => request<SettingsView>('PUT', '/settings/tmdb', { tmdb_api_key }),
  status: () => request<ServerStatus>('GET', '/server/status'),
  clearCache: (kind: 'thumbnails' | 'subtitles' | 'images') => request<void>('POST', `/server/cache/${kind}/clear`),
  regenerateThumbnails: (id: number) => request<void>('POST', `/libraries/${id}/thumbnails`),
  matchMissing: (id: number) => request<void>('POST', `/libraries/${id}/match`),
}

/**
 * A share link's own endpoints. They carry the link token in the path and never the session,
 * so a viewer with no profile can watch exactly what the link covers.
 */
export const shareApi = (token: string) => {
  const base = `/api/share/${encodeURIComponent(token)}`
  const get = async <T,>(path: string): Promise<T> => {
    const res = await fetch(`${base}${path}`, { credentials: 'omit' })
    if (!res.ok) {
      const err = await res.json().catch(() => null)
      throw new ApiError(res.status, err?.error?.code ?? 'unknown', err?.error?.message ?? res.statusText)
    }
    return res.json()
  }
  return {
    view: (itemId?: number) => get<ShareView>(itemId ? `/items/${itemId}` : ''),
    plan: (fileId: number, audio: number | null, caps: PlayCaps) =>
      get<PlaybackPlan>(`/files/${fileId}/plan?${playQuery(audio, caps)}`),
    streamUrl: (fileId: number, audio: number | null, caps: PlayCaps) => `${base}/stream/${fileId}?${playQuery(audio, caps)}`,
    subtitleUrl: (streamId: number) => `${base}/subtitles/${streamId}`,
    imageUrl: (key: string | null | undefined) => (key ? `${base}/images/${key}` : null),
  }
}

/** Saves JSON as a file download, e.g. a profile export. */
export function downloadJson(data: unknown, filename: string) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }))
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}

/** Server-made video frame for an item without artwork (a show uses its first episode). */
export const frameUrl = (itemId: number) => `/api/items/${itemId}/thumb`

/** `/api/images/w500/abc.jpg` for an image key, or null. */
export const imageUrl = (key: string | null | undefined) => (key ? `/api/images/${key}` : null)

/**
 * The same TMDB art at a larger `width` (780 or 1280) for art shown bigger than its stored size, e.g. a 300 px
 * episode still as a full-width backdrop. Null when the key is already that big or is not TMDB art.
 */
export function sharperImageUrl(key: string | null | undefined, width: number) {
  const m = key?.match(/^w(\d+)\/([^/]+)$/)
  return m && Number(m[1]) < width ? `/api/images/w${width}/${m[2]}` : null
}
