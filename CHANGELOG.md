## v0.0.1-alpha.12 (2026-10-28)

- !alpha(all) : Release alpha build
- docs(Shares) : Viewer limit, rate limit and transcode cap
- web(Shares) : Choose how many viewers a share link admits, show the count
- backend(Shares) : Viewer limit, per-address rate limit and a transcode cap per share link
- backend(Security) : Artwork requests respect a profile's library and content limits
- db(Rating) : Fix match replaces the old title's certification
- docs(Packaging) : Reconcile TODO and packaging doc with what ships
- build(Packaging) : Windows server zip script, update-ffmpeg checks the published builds
- core(Platform) : Pause HLS runs on Windows with NtSuspendProcess, drop the unused kill stub
- backend(Perf) : Four ffprobe runs at once on an SSD, two on an HDD
- backend(Perf) : Hold background ffmpeg work while a re-encode runs or the machine is busy
- build(Perf) : Fat LTO and one codegen unit for release builds
- backend(Perf) : mimalloc as the global allocator
- docs(Owner) : Site owner role and upgrade behaviour
- web(Owner) : Site owner, administrator and member badges; android(Owner) : Role in Settings
- backend(Owner) : Site owner role that stays admin and cannot be deleted; upgrade test on real databases
- docs(Accounts) : Device approval, activity, requests and notifications; credit lettre
- backend(Tests) : Cover device approval, activity, requests and notifications; mask webhook addresses
- web(Accounts) : Device approval screen, activity, devices, requests and notification settings
- android(Accounts) : Device approval at sign-in, title requests and watch activity
- backend(Accounts) : Device approval, watch activity, title requests and admin notifications
- db(Accounts) : Add devices, watch time and title request tables
- android(Player) : Keep the episode list scroll position when another episode starts; docs(Player) : Kept list scroll
- web(Player) : Keep the episode list scroll position when another episode starts

## v0.0.1-alpha.11 (2026-10-27)

- !alpha(all) : Release alpha build
- docs(Claude) : Note HLS playback, network retries and saved volume
- player(Volume) : Keep one volume level for every title on web, desktop and Android; docs(Player) : Saved volume
- web(Player) : Play H.264 remux through HLS segments, retry dropped plan and stream requests; docs(Media) : HLS in the web player

## v0.0.1-alpha.10 (2026-10-26)

- !alpha(all) : Release alpha build
- android(Player) : Rapid taps keep skipping after a double tap; docs(Android) : Tap gestures on the video
- web(Player) : Seek bar lands every drag, also when released off the bar

## v0.0.1-alpha.9 (2026-10-24)

- !alpha(all) : Release alpha build
- docs(Todo) : Move Android frame time measurement to future scope
- android(Perf) : Record the profile signed in across tabs and a title; docs(Android) : Signed-in recording steps and results
- android(Perf) : Regenerate profile; docs(Android) : Record cold start result on a phone
- android(Perf) : Baseline and startup profile with generator module and startup benchmark; docs(Android) : Document profile workflow
- android(Fix) : Apply cutout and nav bar insets to player controls only in full screen
- perf(Db) : Title index for library pages, rebuild on new migrations; docs(Api) : Document library paging
- android(Library) : Page the library grid and use paged and show-only calls
- web(Library) : Load library grid in server-sorted pages as the end comes into view
- api(Library) : Page library items with type, sort, offset and limit; plain list kept for older apps
- perf(Web) : Gzip and long cache for hashed assets; docs(Perf) : Record measured results and open findings
- perf(Db) : Index library counts and recent order, limit card joins to result rows, bigger page cache
- perf(Scrub) : Single-thread keyframe decode, GPU try for HEVC/4K, content-hash cache key
- boot(Features) : Add startup check module and send server features to clients so they skip disabled requests
- android(Fix) : Stop polling scrub timeline on 404, retry only on 202
- test(Scrub) : Cover scrub thumbnails default and log e2e result in TODO
- settings(Scrub) : Add admin toggle for seek-bar preview thumbnails, off by default

## v0.0.1-alpha.8 (2026-10-23)

- !alpha(all) : Release alpha build
- android(Refactor) : Improve player animation and control sizes for better responsiveness
- android(Fix) : Fix scrub thumbnail rendering and auto-hide miniplayer controls
- Add comprehensive Material Design 3 references for theming, typography, shape, elevation, and motion
- docs(TODO) : Update completed UI features for web and android
- android(Feature) : Add playlists, collections, genre shelves, and scrub thumbnails
- web(Feature) : Add quick-add to playlist, watch details playlist button, and shuffle
- android(Fix) : Fix slide to pip animation and prevent black screen when minimized
- android(Fix) : Fix ForegroundServiceDidNotStartInTimeException in PlaybackService

## v0.0.1-alpha.7 (2026-10-22)

- !alpha : Release alpha build
- android(Fix) : Clamp spring progress to prevent negative corner radius
- android(Feature) : Use M3 expressive spring motion and single persistent texture frame for PiP
- android(Feature) : Add YouTube-style watch screen, swipe gestures, and smooth PiP for mobile
- android(Feature) : Add YouTube-style in-app PiP miniplayer and controls on tablet
- android(Feature) : Add swipe down and miniplayer gestures with localized strings
- web(Fix) : Authenticate timeline and sprite requests with session token
- backend(Fix) : Support query token auth and fix sprite filename parsing
- android(Fix) : Remove unused nativeKeyEvent import in PlayerScreen
- media(Fix) : Allow chunks_exact_to_as_chunks in decode_times
- fix(merge) : Apply content-rating limits to playlists, collections and shelves; pass profile languages to HLS
- docs(profiles) : Document profiles, household rules, LAN-only unlock, share links and export/import
- feat(web) : Household rules, profile languages, export/import, share links and the public share page
- feat(profiles) : Offer track languages and rating levels with the profile
- feat(web) : Show scrub previews on the seek bar and let admins set or clear skip markers in the player
- docs(Change) : Document the keyframe index and HLS sessions
- backend(Change) : Index keyframes after scans, snap remux seeks to them, and serve HLS that reuses cut segments and idles 60 s
- media(Change) : Build keyframe indexes from Cues, stss or a bounded packet scan, and cut HLS segments that line up across ffmpeg runs
- docs(subtitles) : Describe scan-time extraction, sidecar sync and OCR
- db(Change) : Store a keyframe index per file version
- feat(player) : Offer OCR subtitles, keep fetched tracks and add the OCR setting
- feat(subtitles) : Cache WebVTT after each scan and add the optional OCR setting
- feat(scanner) : Pick up sidecar subtitles added, edited or removed next to an unchanged video
- feat(media) : Extract all text subtitles of a file in one run and read PGS/VobSub with tesseract
- feat(db) : Store sidecar fingerprints and OCR state on subtitle streams
- feat(profiles) : LAN-only unlocked profiles, content limits, per-profile languages, export/import and scoped share links
- feat(metadata) : Keep TMDB certifications on movies and shows, backfilled once for matched titles
- feat(profiles) : Store profile preferences, content ratings, LAN-only sessions and share links
- docs(Playlists) : Locked fields, folder polling, playlists, collections and shelves
- feat(markers) : Find end credits over lit footage by stable picture with lettering on it
- feat(Web) : Edit details with field locks, folder polling and home shelves settings
- feat(Web) : Playlists, collections and genre shelves on Home, playlist autoplay
- feat(DB) : List genre names for shelves
- feat(API) : Playlists, collections, genre shelves, metadata edit and locks
- feat(Library) : Per-folder polling task, TMDB collections on match
- feat(Scanner) : Poll folders by directory listing diff for network shares
- feat(scrub) : Make scrub thumbnail sheets and a WebVTT index on the first request, behind auth, at low priority
- feat(DB) : Locked metadata fields, root polling options, playlists, collections and genre shelves
- feat(Android) : Move the last hard-coded update errors to strings.xml and add es, fr, de, hi drafts for the core strings
- feat(Android) : Tablet keyboard shortcuts in the player, space, k, f, t and the arrows like the web player
- feat(Android) : Seek inside the buffered part of a remux stream without restarting it when the player can seek the stream
- docs(desktop) : Document the bundled mpv, its lock and what is still unpinned
- feat(desktop) : Offer the in-app player when mpv is missing or fails to start
- feat(Android) : Picture-in-picture window gets back and forward 10 s buttons, and attaching a keyboard no longer recreates the player
- build(desktop) : Ship the fetched mpv in the installers as resources/mpv
- feat(desktop) : Fetch a pinned, checksum-verified mpv at build time
- feat(desktop) : Find a bundled mpv before PATH and start it without the AppImage environment

## v0.0.1-alpha.6 (2026-10-22)

- !alpha:$Release
- feat(transcode) : Encode on the GPU when a startup test encode finds NVENC, Quick Sync or VA-API
- fix(transcode) : Keep at least 4 threads per re-encode so a Raspberry Pi 5 still converts 1080p in real time
- perf(transcode) : Cap the x264 peak bitrate by output height and the lookahead of slow presets
- perf(transcode) : Give each re-encode a share of the cores instead of every core
- perf(transcode) : Shorten the re-encode lead to 30 s so each play or seek spikes the CPU for half as long
- docs(showcase) : Add Kinema motion showcase video, fix mobile drawer containment and polish multi-device holders
- docs(install) : Prioritize one-line script installers over docker in quick start and homepage
- style(docs) : Switch to full app-icon tile logo and add standalone semantic CSS styling
- ci(docs) : Update pnpm lockfile and relax frozen-lockfile check in deploy workflow
- ci(docs) : Use corepack for pnpm 12.4.2 in deploy-docs workflow
- ci(docs) : Use packageManager version resolution in deploy-docs workflow
- ci(docs) : Specify pnpm version in deploy workflow and package.json
- ci(docs) : Add GitHub Pages deployment workflow and automated build verification
- docs(portal) : Add comprehensive documentation guides, API reference, and interactive architecture diagrams
- feat(docs) : Initialize Docusaurus documentation portal with custom Kinema cinematic theme and animations

## v0.0.1-alpha.5 (2026-10-21)

- !alpha : Dedicated PiP button, swipe-down PiP gesture, and media notification seekbar
- fix(player) : Re-encode audio with transcoded video so seeks no longer freeze the picture
- fix(android) : Match the system predictive back animation and stop the leaving page ghosting
- docs(readme) : Add multi-device screenshots with open-source demo library
- perf(player) : Start transcodes in about 2.5 s after play or seek and hold the picture meanwhile
- fix(player) : Revert desktop HEVC switches, fall back to transcode on decode errors, steady repeated skips

## v0.0.1-alpha.4 (2026-10-21)

- !alpha(all) : HEVC GPU decoding, throttled transcodes, and desktop mpv playback
- docs(all) : Add the HEVC playback work to the README and credits (libc, mpv)
- docs(all) : Record the HEVC playback work in CLAUDE.md and TODO
- feat(desktop) : Play titles in mpv with no remux or transcode on the server
- feat(server) : Serve the file as it is on raw=true for players that decode anything
- perf(server) : Decode HEVC on the Raspberry Pi 5 GPU for transcodes
- perf(server) : Throttle live transcodes to playback pace and lower ffmpeg priority
- feat(desktop) : Turn on GPU HEVC decoding on Linux so HEVC files remux

## v0.0.1-alpha.3 (2026-10-20)

- !alpha(all) : YouTube watch page, skip intro and credits, and in-app updates
- feat(player) : Skip intro and skip credits controls for Android and Web
- feat(server) : Find intro and end credits for Skip intro and Up next
- feat(web) : Add theater mode to the watch page on the t key
- feat(web) : Add a YouTube-style watch page with an up-next list
- feat(android) : Add a YouTube-style watch page with episodes and suggestions
- feat(android) : Offer the next episode only in the last minute
- fix(android) : Size the home hero and title page for tablets and foldables
- feat(android): implement animated splash screen and update app icons
- feat(android) : Add Updates and About Kinema screens with channel, release notes and acknowledgements
- fix(web) : Keep only web, desktop and server credits in Acknowledgements
- refactor(web) : Move About Kinema into Settings and add an Updates section
- feat(android) : Auto-update the Android app from GitHub releases
- feat(desktop) : Auto-update the desktop app from GitHub releases
- feat(web) : Check GitHub for a newer server release and show the update commands
- feat(web) : Add About Kinema page with developer and acknowledgements
- refactor(android) : Remove server-wide Languages section from Settings
- fix(android) : Show up to 4 lines per track label so subtitle titles are readable
- feat(android) : Add dev gateway script that forwards ports and probes server from devices
- feat(ui) : Network tab owns public address, series tracks on Android, S/E links, sharper hero art

## v0.0.1-alpha.2 (2026-10-20)

- !alpha: Fix Android release build (SwipeLevels JVM signature clash) and rustfmt
- fix(android) : Rename SwipeLevels setters to avoid JVM signature clash with private property setters
- !alpha: Media service, PiP, swipe gestures, volume boost, home-address roaming and updater scripts
- feat(installers) : Install scripts double as safe updaters
- feat(android) : Media service, picture-in-picture, swipe gestures, stop button and server languages
- feat(web) : Volume boost up to 200% in the player
- fix(android) : Sharper hero carousel and wide card artwork
- feat(server) : Sync server name into the web page and prefer the home address

## v0.0.1-alpha.2 (2026-10-20)

- !alpha: Media service, PiP, swipe gestures, volume boost, home-address roaming and updater scripts
- feat(installers) : Install scripts double as safe updaters
- feat(android) : Media service, picture-in-picture, swipe gestures, stop button and server languages
- feat(web) : Volume boost up to 200% in the player
- fix(android) : Sharper hero carousel and wide card artwork
- feat(server) : Sync server name into the web page and prefer the home address

## v0.0.1-alpha.1 (2026-10-19)

- !alpha : Initial alpha release with home server bundle, desktop app, and web player
- feat(installers) : Auto pick highest available release channel (stable -> beta -> alpha)
- docs(readme) : Add quick install one-liners at the top of README
- ci(workflow) : Add release and CI GitHub Actions workflows
- feat(installers) : Add home server and desktop installation scripts and quick install instructions
- build(server) : Add Dockerfile, compose file and the bundled server tarball with ffmpeg
- build(android) : Sign release builds from keystore.properties and take versionCode from the root build number
- feat(backend) : Fall back to the next free port when the configured one is taken
- fix(desktop) : Add homepage and author email so the deb package builds
- fix(web) : Add right padding for desktop window controls overlay
- docs(README) : Add architecture diagrams, data model and project overview
- fix(desktop) : Robustly locate web dist and handle load failures in desktop app
- feat(desktop) : Add desktop app with server switching and remove default menu items
- feat(android) : Redesign the library switcher sheet and keep shows collapsed between opens
- feat(android) : Open a collapsible library switcher sheet when Library is tapped again
- feat(web) : Indent shows under their library in the sidebar
- feat(android) : Show a mixed library's movies with its shows as chips, like the web sidebar
- feat(sync) : Save each profile's audio and subtitle pick across devices
- fix(android) : Fall back to H.264 when the phone fails to decode a stream
- fix(android) : Keep the leaving page on top during predictive back
- feat(android) : Add the Android app with Material 3 Expressive design
- feat(server) : Serve HTTPS with a self-signed certificate, LAN discovery and a public URL
- web(Player) : Add stop button matching Plex playback controls
- feat(web) : Pick audio and subtitle tracks for all episodes of a show
- feat(media) : Store a per-show audio and subtitle track choice
- feat(item) : Add play, watched and copy-link actions to show and season pages
- feat(tvmaze) : Integrate TVmaze metadata support for shows without TMDB matches
- docs(media) : Note -noaccurate_seek for copied video with re-encoded audio
- fix(media) : Keep re-encoded audio in sync with copied video after a seek
- web(Nav) : List individual TV show titles directly and remove TV Shows option
- web(Library) : Segregate TV serials and movies into distinct top-level titles
- feat(codegraph) : Add CodeGraph configuration and documentation files
- docs(meta) : Document cast and crew metadata and profile image caching
- feat(web) : Display cast and crew avatars and roles on item details
- feat(meta) : Fetch and persist movie and show cast and crew from TMDB
- feat(web) : Add Plex-style split click targets to play video or open details
- fix(player) : Fix seek click dropping, slider snapback and out-of-order responses
- style(player) : Expand range slider hit area and add hover styling
- docs(users) : Document profiles, PIN lockout, library access and new endpoints
- feat(web) : Add profile picker with PIN pad, users manager, and settings nav in the app sidebar
- feat(users) : Add family profiles with PIN lock, lockout and per-profile library access
- docs(settings) : Document settings sections, background thumbnails and new endpoints
- feat(settings) : Add sectioned settings with dashboard, transcoder, languages and library actions
- feat(backend) : Generate thumbnails in a background job and add server settings, status and cache APIs
- docs(mvp) : Document thumbnails cache and new home layout
- feat(web) : Redesign home with hero carousel, show episodes as a grid, use frame thumbnails
- feat(media) : Add cached video-frame thumbnails and kind-split home shelves
- docs(mvp) : Document episode grouping, fix match and library sub-entries
- feat(web) : Add Movies and TV Shows sub-entries for mixed libraries
- feat(match) : Add admin fix-match search and apply against TMDB
- feat(scanner) : Group episodes found in movie libraries into shows and regroup old items
- chore(dev) : Restart backend on source changes in pnpm dev without cargo-watch
- fix(scanner) : Reject whole-drive and system folders as library roots and skip /proc, /sys, /dev, /run
- docs(mvp) : Document MVP architecture, playback flow, API, Ember palette and TODO
- feat(web) : Add login, home, library, detail, settings and player with audio and subtitle switching
- fix(media) : Align remux time offset with ffmpeg's real seek point for subtitle sync
- feat(backend) : Add admin setup, login sessions, library scan, metadata, direct/remux streaming and subtitles API
- feat(meta) : Add TMDB client for movies, shows, seasons and images
- style(scanner) : Fix clippy lints
- feat(media) : Add ffprobe parsing, direct/remux/transcode plan, fMP4 stream command and WebVTT extraction
- feat(scanner) : Add release-name parser, video walker, sidecar subtitles and fs watcher
- feat(db) : Add MVP migration and queries for users, libraries, items, files and watch state
- docs(m0) : Update roadmap, playback pipeline, and mark M0 complete in TODO
- feat(web) : Initialize React TypeScript web app with Vite and Tailwind
- style(design-system) : Add Kinema design tokens and UI principles
- chore(packaging) : Add build and packaging scripts for Linux, Windows, Docker, and ffmpeg pinning
- chore(agents) : Add agent workspace skills and configuration
- feat(backend) : Initialize Cargo workspace and crate skeleton
- docs(init) : Add design documentation and repository baseline

