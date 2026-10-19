export interface User {
  id: number
  name: string
  is_admin: boolean
}

export type LibraryKind = 'movie' | 'show' | 'other'

export interface Library {
  id: number
  name: string
  kind: LibraryKind
  item_count: number
  roots: string[]
  scanning: boolean
}

export type ItemKind = 'movie' | 'show' | 'season' | 'episode' | 'video'

export interface ItemCard {
  id: number
  library_id: number
  parent_id: number | null
  kind: ItemKind
  title: string
  year: number | null
  season_no: number | null
  episode_no: number | null
  summary: string | null
  rating: number | null
  poster: string | null
  backdrop: string | null
  show_title: string | null
  position_s: number | null
  watched: boolean
  duration_s: number | null
  child_count: number
  added_at: number
}

export interface Stream {
  id: number
  file_id: number
  idx: number
  type: 'video' | 'audio' | 'subtitle'
  codec: string | null
  profile: string | null
  language: string | null
  title: string | null
  channels: number | null
  width: number | null
  height: number | null
  is_default: boolean
}

export interface MediaFile {
  id: number
  path: string
  size: number
  container: string | null
  duration_s: number | null
  bitrate: number | null
  probe_status: 'pending' | 'ok' | 'failed'
  streams: Stream[]
}

export interface ItemDetail {
  item: ItemCard
  genres: string[]
  files: MediaFile[]
  children: ItemCard[]
  prev_id: number | null
  next_id: number | null
}

export interface Home {
  continue_watching: ItemCard[]
  recently_added: { library: Library; items: ItemCard[] }[]
}

export interface PlaybackPlan {
  mode: 'direct' | 'remux' | 'transcode'
  duration_s: number | null
  audio: number | null
  /** Where the stream really starts (a seek point before the requested time); the player's time offset. */
  start: number
}

export interface DirListing {
  path: string
  parent: string | null
  dirs: { name: string; path: string }[]
}

export interface ServerSettings {
  tmdb_configured: boolean
  ffmpeg_found: boolean
}
