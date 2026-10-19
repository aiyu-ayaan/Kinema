import { storageKeys } from './flavor'

export interface ServerInfo {
  name: string
  version: string
  capabilities: string[]
}

export async function fetchServerInfo(): Promise<ServerInfo> {
  const token = localStorage.getItem(storageKeys.token)
  const headers: HeadersInit = {}
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch('/api/server/info', { headers })
  if (!res.ok) {
    throw new Error(`Failed to fetch server info: ${res.statusText}`)
  }
  return res.json()
}
