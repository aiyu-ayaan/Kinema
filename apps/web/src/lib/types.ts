export type AvatarColor = 'ember' | 'ocean' | 'forest' | 'plum' | 'sun' | 'rose' | 'slate' | 'teal'

export interface User {
  id: number
  name: string
  is_admin: boolean
  avatar: AvatarColor | null
  has_password: boolean
  has_pin: boolean
}

/** A profile on the picker, as seen from this device's network. */
export interface ProfileCard extends User {
  /** Opens with one tap from here (home network, and no lock or a PIN skipped at home). */
  opens_without_lock: boolean
  /** No lock, so it cannot be opened away from home. */
  home_only: boolean
}

export type SubtitleMode = 'off' | 'always' | 'foreign'

/** Track languages: ISO 639-2, '' for the file's default, null to follow the server. */
export interface Languages {
  audio_language: string | null
  subtitle_language: string | null
  subtitle_mode: SubtitleMode | null
}

/** A profile's preferences; limits and LAN unlock are set by an admin. */
export interface Prefs extends Languages {
  /** Minimum-age scale 0..18; null = no limit. */
  max_rating: number | null
  allow_unrated: boolean
  /** A PIN profile opens without it on the home network. */
  lan_unlock: boolean
}

export interface ProfileOptions {
  track_languages: string[]
  subtitle_modes: SubtitleMode[]
  rating_levels: number[]
}

export interface MyProfile {
  user: User
  preferences: Prefs
  options: ProfileOptions
}

/** A profile as the admin edits it. Empty `library_ids` means every library. */
export interface ManagedUser extends User {
  library_ids: number[]
  preferences: Prefs
}

export interface ProfileInput {
  name: string
  is_admin: boolean
  avatar: AvatarColor
  /** undefined keeps the current one, '' removes it */
  password?: string
  pin?: string
  library_ids: number[]
  /** undefined keeps them */
  preferences?: Prefs
}

export interface ImportResult {
  imported: number
  unmatched: number
  kept: number
  lock_applied: boolean
}

export interface Share {
  id: number
  created_by: number
  creator_name: string | null
  item_id: number
  item_title: string | null
  item_kind: ItemKind | null
  label: string | null
  created_at: number
  expires_at: number
  revoked_at: number | null
}

export interface CreatedShare {
  share: Share
  /** Shown once; the server keeps only a hash. */
  token: string
  /** Client route, `/s/<token>`. */
  path: string
}

export interface ShareFile {
  id: number
  container: string | null
  duration_s: number | null
  streams: Stream[]
}

/** What a share-link viewer sees: one title and what is under it. */
export interface ShareView {
  share: { label: string | null; expires_at: number; target_id: number }
  item: ItemCard
  children: ItemCard[]
  files: ShareFile[]
}

export type LibraryKind = 'movie' | 'show' | 'other'

export interface Library {
  id: number
  name: string
  kind: LibraryKind
  item_count: number
  show_count: number
  roots: string[]
  /** Per-folder scan mode: filesystem events, or polling for network shares. */
  root_options: RootOption[]
  scanning: boolean
  generating_thumbnails: boolean
}

export interface RootOption {
  path: string
  poll: boolean
  poll_interval_min: number
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
  /** Certification ("PG-13"), inherited from the show; '' looked up and unrated. */
  content_rating: string | null
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
  /** A picture subtitle read by OCR on the server: it has a WebVTT like a text track. */
  ocr?: boolean
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
  source: 'chapter' | 'audio' | 'black' | 'text' | 'manual'
}

export interface CastMember {
  id: number
  name: string
  role: string
  image: string | null
}

export type LibrarySort = 'title' | 'added' | 'year' | 'rating'

/** One page of a library; `next` is the offset to ask for after it, null at the end. */
export interface LibraryPage {
  items: ItemCard[]
  /** null while a rating limit hides an unknown share of the titles. */
  total: number | null
  next: number | null
}

export interface ItemDetail {
  item: ItemCard
  genres: string[]
  /** Fields edited by hand; scans and refreshes never overwrite them. */
  locked_fields: LockableField[]
  /** Franchise collections the title belongs to. */
  collections: { id: number; name: string; tmdb_id: number | null }[]
  files: MediaFile[]
  /** What the server serves right now; skip requests for a feature that is off. */
  features: { scrub_thumbnails: boolean }
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
  playlists: PlaylistSummary[]
  collections: CollectionSummary[]
  genre_shelves: { id: number; title: string; genre: string; items: ItemCard[] }[]
}

export const lockableFields = ['title', 'year', 'summary', 'rating', 'poster', 'backdrop', 'genres'] as const
export type LockableField = (typeof lockableFields)[number]

export interface PlaylistSummary {
  id: number
  owner_id: number
  owner_name: string
  name: string
  is_shared: boolean
  mine: boolean
  /** Items this profile may see. */
  item_count: number
  poster: string | null
}

export interface PlaylistDetail extends PlaylistSummary {
  items: ItemCard[]
}

export interface CollectionSummary {
  id: number
  name: string
  tmdb_id: number | null
  item_count: number
  poster: string | null
}

export interface CollectionDetail extends CollectionSummary {
  items: ItemCard[]
}

export interface ShelfDef {
  id: number
  title: string
  genre: string
  position: number
  /** This profile hid the shelf. */
  hidden: boolean
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
  transcode_threads: number
  transcode_hardware: boolean
  rescan_hours: number
  subtitle_ocr: boolean
  scrub_thumbnails: boolean
}

export interface SettingsView {
  server: ServerOptions
  tmdb_configured: boolean
  ffmpeg_found: boolean
  hardware_encoder: string | null
  /** OCR language models when tesseract was found at startup. */
  ocr_languages: string[] | null
  options: {
    presets: string[]
    max_heights: number[]
    threads: number[]
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
    extracting_subtitles: boolean
  }[]
  active_streams: number
  active_transcodes: number
  caches: { thumbnails: number; subtitles: number; images: number }
}
