import { storageKeys } from './flavor'
import type {
  DirListing,
  Home,
  ItemCard,
  ItemDetail,
  Library,
  LibraryKind,
  PlaybackPlan,
  ServerSettings,
  User,
} from './types'

export interface ServerInfo {
  name: string
  version: string
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
  login: (name: string, password: string) => request<SessionResponse>('POST', '/auth/login', { name, password }),
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

  plan: (fileId: number, audio: number | null, caps: PlayCaps, start: number) =>
    request<PlaybackPlan>('GET', `/files/${fileId}/plan?${playQuery(audio, caps, start)}`),
  streamUrl: (fileId: number, audio: number | null, caps: PlayCaps, start: number) =>
    `/api/stream/${fileId}?${playQuery(audio, caps, start)}`,
  subtitleText: async (streamId: number) => {
    const res = await fetch(`/api/subtitles/${streamId}`)
    if (!res.ok) throw new ApiError(res.status, 'subtitle', res.statusText)
    return res.text()
  },

  settings: () => request<ServerSettings>('GET', '/settings'),
  saveSettings: (tmdb_api_key: string) => request<ServerSettings>('PUT', '/settings', { tmdb_api_key }),
}

/** `/api/images/w500/abc.jpg` for an image key, or null. */
export const imageUrl = (key: string | null | undefined) => (key ? `/api/images/${key}` : null)
