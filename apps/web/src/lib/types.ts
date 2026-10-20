export type AvatarColor = 'ember' | 'ocean' | 'forest' | 'plum' | 'sun' | 'rose' | 'slate' | 'teal'

export interface User {
  id: number
  name: string
  is_admin: boolean
  avatar: AvatarColor | null
  has_password: boolean
  has_pin: boolean
}

/** A profile as the admin edits it. Empty `library_ids` means every library. */
export interface ManagedUser extends User {
  library_ids: number[]
}

export interface ProfileInput {
  name: string
  is_admin: boolean
  avatar: AvatarColor
  /** undefined keeps the current one, '' removes it */
  password?: string
  pin?: string
  library_ids: number[]
}

export type LibraryKind = 'movie' | 'show' | 'other'

export interface Library {
  id: number
  name: string
  kind: LibraryKind
  item_count: number
  show_count: number
  roots: string[]
  scanning: boolean
  generating_thumbnails: boolean
}

export type ItemKind = 'movie' | 'show' | 'season' | 'episode' | 'video'

export interface ItemCard {
  id: number
  library_id: number
  parent_id: number | null
  /** Season or episode: the show it belongs to. */
  show_id: number | null
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
  /** Intro and end credits, found by the server or set by an admin. */
  markers?: Marker[]
}

export interface Marker {
  kind: 'intro' | 'credits'
  start_s: number
  end_s: number
  source: 'chapter' | 'audio' | 'black' | 'manual'
}

export interface CastMember {
  id: number
  name: string
  role: string
  image: string | null
}

export interface ItemDetail {
  item: ItemCard
  genres: string[]
  files: MediaFile[]
  children: ItemCard[]
  cast: CastMember[]
  prev_id: number | null
  next_id: number | null
  /** Show or season: the episode "Play" starts on. */
  next_up: ItemCard | null
  /** Show only. */
  tracks: SeriesTracks | null
}

/** A show's track choice: positions among the first episode's audio and subtitle streams. */
export interface SeriesTracks {
  /** null: server language settings */
  audio: number | null
  /** null: server language settings, -1: off */
  subtitle: number | null
  /** The first episode's streams. */
  streams: Stream[]
}

export interface Home {
  continue_watching: ItemCard[]
  shelves: { library: Library; kind: 'movies' | 'shows' | 'episodes'; items: ItemCard[] }[]
}

export interface PlaybackPlan {
  mode: 'direct' | 'remux' | 'transcode'
  duration_s: number | null
  audio: number | null
  /** Where the stream really starts (a seek point before the requested time); the player's time offset. */
  start: number
  /** Subtitle stream id the server's language settings pick; applied on first load. */
  subtitle: number | null
}

export interface MatchCandidate {
  tmdb_id: number
  title: string
  year: number | null
  overview: string | null
  /** TMDB path like "/abc.jpg" */
  poster: string | null
}

export interface DirListing {
  path: string
  parent: string | null
  dirs: { name: string; path: string }[]
}

/** Admin-editable server options (backend `server_settings.rs`). */
export interface ServerOptions {
  server_name: string
  /** Address the apps use away from home; '' when there is none. */
  public_url: string
  watch_changes: boolean
  thumbnails: boolean
  thumbnail_position: number
  metadata_language: string
  preferred_audio_language: string
  preferred_subtitle_language: string
  subtitle_mode: 'off' | 'always' | 'foreign'
  max_transcodes: number
  transcode_preset: string
  transcode_crf: number
  transcode_max_height: number
  rescan_hours: number
}

export interface SettingsView {
  server: ServerOptions
  tmdb_configured: boolean
  ffmpeg_found: boolean
  options: {
    presets: string[]
    max_heights: number[]
    subtitle_modes: ServerOptions['subtitle_mode'][]
    metadata_languages: string[]
    track_languages: string[]
  }
}

export interface ServerStatus {
  name: string
  version: string
  uptime_s: number
  ffmpeg: string | null
  ffprobe: string | null
  data_dir: string
  listen: string
  lan_urls: string[]
  /** SHA-256 of the HTTPS certificate; null when TLS is off. */
  tls_fingerprint: string | null
  stats: { movies: number; shows: number; episodes: number; files: number; bytes: number; hours: number }
  libraries: {
    id: number
    name: string
    kind: LibraryKind
    titles: number
    files: number
    thumbnails: number
    scanning: boolean
    generating_thumbnails: boolean
  }[]
  active_streams: number
  active_transcodes: number
  caches: { thumbnails: number; subtitles: number; images: number }
}
