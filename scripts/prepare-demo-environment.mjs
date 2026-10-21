import fs from 'node:fs'
import path from 'node:path'
import crypto from 'node:crypto'
import { execSync } from 'node:child_process'

const ROOT = process.cwd()
const DUMMY_DIR = path.join(ROOT, 'dummy_media')
const MOVIES_DIR = path.join(DUMMY_DIR, 'Movies')
const SHOWS_DIR = path.join(DUMMY_DIR, 'TV Shows')
const DATA_DIR = path.join(ROOT, 'data')
const DB_PATH = path.join(DATA_DIR, 'kinema.db')

console.log('--- Setting up pristine Demo Data (Creative Commons / Open Source) ---')

// 1. Ensure dirs
for (const p of [
  path.join(DATA_DIR, 'cache', 'images', 'w500'),
  path.join(DATA_DIR, 'cache', 'images', 'w780'),
  path.join(DATA_DIR, 'cache', 'images', 'w1280'),
  path.join(DATA_DIR, 'cache', 'images', 'original'),
  path.join(DATA_DIR, 'cache', 'thumbs'),
  path.join(DATA_DIR, 'cache', 'subtitles'),
  path.join(MOVIES_DIR, 'Big Buck Bunny (2008)'),
  path.join(MOVIES_DIR, 'Sintel (2010)'),
  path.join(MOVIES_DIR, 'Tears of Steel (2012)'),
  path.join(MOVIES_DIR, 'Cosmos Laundromat (2015)'),
  path.join(MOVIES_DIR, 'Elephants Dream (2006)'),
  path.join(SHOWS_DIR, 'Blender Open Stories', 'Season 01'),
]) {
  fs.mkdirSync(p, { recursive: true })
}

// 2. Movie metadata definitions
const MOVIES = [
  {
    id: 1,
    key: 'sintel',
    title: 'Sintel',
    year: 2010,
    rating: 8.5,
    runtime: 900,
    summary: 'A lonely young woman named Sintel searches across a perilous fantasy realm for a baby dragon she befriended and named Scales, facing fierce obstacles and heartbreak.',
    posterUrl: 'https://upload.wikimedia.org/wikipedia/commons/8/8f/Sintel_poster.jpg',
    backdropUrl: 'https://upload.wikimedia.org/wikipedia/commons/c/cf/Durian_-_Sintel-wallpaper-sintel.jpg',
    folder: 'Sintel (2010)',
    filename: 'Sintel (2010) [1080p].mp4',
    position: 215.0,
    watched: 0,
    color: '#8b4513'
  },
  {
    id: 2,
    key: 'bbb',
    title: 'Big Buck Bunny',
    year: 2008,
    rating: 8.2,
    runtime: 600,
    summary: 'A large, lovable rabbit with a gentle heart decides to stand up against a trio of mischievous forest bullies after they terrorize his fellow woodland creatures.',
    posterUrl: 'https://upload.wikimedia.org/wikipedia/commons/c/c5/Big_buck_bunny_poster_big.jpg',
    backdropUrl: 'https://peach.blender.org/wp-content/uploads/bbb-splash.png',
    folder: 'Big Buck Bunny (2008)',
    filename: 'Big Buck Bunny (2008) [1080p].mp4',
    position: 600.0,
    watched: 1,
    color: '#2e7d32'
  },
  {
    id: 3,
    key: 'tos',
    title: 'Tears of Steel',
    year: 2012,
    rating: 7.8,
    runtime: 720,
    summary: 'In a dystopian future Amsterdam, a ragtag team of scientists and warriors attempts to stage a key memory playback to save humanity from destructive rogue robotic sentinels.',
    posterUrl: 'https://upload.wikimedia.org/wikipedia/commons/7/70/Tos-poster.png',
    backdropUrl: 'https://upload.wikimedia.org/wikipedia/commons/1/12/Tears_of_Steel_frame_09_1a.jpg',
    folder: 'Tears of Steel (2012)',
    filename: 'Tears of Steel (2012) [4K].mp4',
    position: 340.0,
    watched: 0,
    color: '#1565c0'
  },
  {
    id: 4,
    key: 'cosmos',
    title: 'Cosmos Laundromat',
    year: 2015,
    rating: 8.0,
    runtime: 660,
    summary: 'On a desolate windswept island, a despondent sheep named Franck meets an enigmatic salesman named Victor who offers him the gift of infinite alternate lives.',
    posterUrl: 'https://upload.wikimedia.org/wikipedia/commons/c/c5/CosmosLaundromatPoster.jpg',
    backdropUrl: 'https://upload.wikimedia.org/wikipedia/commons/2/22/CosmosLaundromat-final-scene.jpg',
    folder: 'Cosmos Laundromat (2015)',
    filename: 'Cosmos Laundromat (2015) [1080p].mp4',
    position: 0,
    watched: 0,
    color: '#e65100'
  },
  {
    id: 5,
    key: 'ed',
    title: 'Elephants Dream',
    year: 2006,
    rating: 7.5,
    runtime: 660,
    summary: 'Two travellers, elder Proog and young Emo, navigate the labyrinthine corridors of an immense and surreal clockwork machine whose reality reflects their deepest perceptions.',
    posterUrl: 'https://upload.wikimedia.org/wikipedia/commons/0/0c/ElephantsDreamPoster.jpg',
    backdropUrl: 'https://upload.wikimedia.org/wikipedia/commons/d/db/Elephants_Dream_f11621.png',
    folder: 'Elephants Dream (2006)',
    filename: 'Elephants Dream (2006) [1080p].mp4',
    position: 0,
    watched: 0,
    color: '#4a148c'
  }
]

// 3. Download posters & backdrops
console.log('Checking artwork in cache...')
for (const m of MOVIES) {
  const posterFile = `${m.key}_poster.jpg`
  const backdropFile = `${m.key}_backdrop.jpg`

  for (const size of ['w500', 'w780', 'original']) {
    const dest = path.join(DATA_DIR, 'cache', 'images', size, posterFile)
    if (!fs.existsSync(dest) || fs.statSync(dest).size < 1000) {
      console.log(`Fetching ${size} poster for ${m.title}...`)
      try {
        execSync(`curl -s -L -A "Mozilla/5.0" "${m.posterUrl}" -o "${dest}"`)
      } catch (e) {
        console.error(`Failed poster for ${m.title}: ${e.message}`)
      }
    }
  }

  for (const size of ['w1280', 'original']) {
    const dest = path.join(DATA_DIR, 'cache', 'images', size, backdropFile)
    if (!fs.existsSync(dest) || fs.statSync(dest).size < 1000) {
      console.log(`Fetching ${size} backdrop for ${m.title}...`)
      try {
        execSync(`curl -s -L -A "Mozilla/5.0" "${m.backdropUrl}" -o "${dest}"`)
      } catch (e) {
        console.error(`Failed backdrop for ${m.title}: ${e.message}`)
      }
    }
  }
}

// 4. Generate playable video files with ffmpeg
console.log('Generating playable dummy video files with ffmpeg...')
for (const m of MOVIES) {
  const filePath = path.join(MOVIES_DIR, m.folder, m.filename)
  if (!fs.existsSync(filePath)) {
    console.log(`Generating ${m.filename}...`)
    const cmd = `ffmpeg -y -f lavfi -i "testsrc=size=1920x1080:rate=24:duration=180" \
      -f lavfi -i "sine=frequency=440:duration=180" \
      -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
      -c:a aac -b:a 192k \
      -movflags +faststart "${filePath}"`
    execSync(cmd, { stdio: 'ignore' })
  }
}

// TV Series Episodes
const EPISODES = [
  { ep: 1, title: 'The Awakening', file: 'Blender Open Stories - S01E01 - The Awakening.mp4' },
  { ep: 2, title: 'The Flight', file: 'Blender Open Stories - S01E02 - The Flight.mp4' },
  { ep: 3, title: 'The Final Run', file: 'Blender Open Stories - S01E03 - The Final Run.mp4' }
]

for (const ep of EPISODES) {
  const filePath = path.join(SHOWS_DIR, 'Blender Open Stories', 'Season 01', ep.file)
  if (!fs.existsSync(filePath)) {
    console.log(`Generating ${ep.file}...`)
    const cmd = `ffmpeg -y -f lavfi -i "testsrc=size=1920x1080:rate=24:duration=180" \
      -f lavfi -i "sine=frequency=520:duration=180" \
      -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
      -c:a aac -b:a 192k \
      -movflags +faststart "${filePath}"`
    execSync(cmd, { stdio: 'ignore' })
  }
}

// 5. Update SQLite Database
console.log('Updating SQLite database with dummy data...')
const now = Math.floor(Date.now() / 1000)

// Predefined fixed token
const SESSION_TOKEN = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'
const SESSION_HASH = crypto.createHash('sha256').update(SESSION_TOKEN).digest('hex')

// Read existing password hash for user 1 if present
let passwordHash = '$argon2id$v=19$m=19456,t=2,p=1$LvXl2DhzdMnkLijY2rj6ZA$0gAVA3WaMOnoJQK4sYNc5b+rOQMNXPo+yMiZw5L9CGs'
try {
  const out = execSync(`sqlite3 "${DB_PATH}" "SELECT password_hash FROM users WHERE id=1;"`, { encoding: 'utf8' }).trim()
  if (out) passwordHash = out
} catch (e) {}

// Clear tables
const sqlScript = `
DELETE FROM watch_state;
DELETE FROM track_choice;
DELETE FROM markers;
DELETE FROM streams;
DELETE FROM files;
DELETE FROM playlist_items;
DELETE FROM playlists;
DELETE FROM item_genres;
DELETE FROM items;
DELETE FROM library_roots;
DELETE FROM libraries;
DELETE FROM sessions;

-- Ensure user 1
INSERT OR REPLACE INTO users (id, name, is_admin, password_hash, created_at)
VALUES (1, 'Ayaan', 1, '${passwordHash}', ${now - 86400 * 30});

-- Add active session
INSERT INTO sessions (id, user_id, created_at, expires_at)
VALUES ('${SESSION_HASH}', 1, ${now}, ${now + 86400 * 30});

-- Libraries
INSERT INTO libraries (id, name, kind, language, scan_interval_min)
VALUES (1, 'Movies', 'movie', 'en-US', 0);
INSERT INTO libraries (id, name, kind, language, scan_interval_min)
VALUES (2, 'TV Shows', 'show', 'en-US', 0);

INSERT INTO library_roots (library_id, path) VALUES (1, '${MOVIES_DIR}');
INSERT INTO library_roots (library_id, path) VALUES (2, '${SHOWS_DIR}');
`

execSync(`sqlite3 "${DB_PATH}" "${sqlScript}"`)

// Insert Movies, files, streams, markers, thumbs
let fileId = 1
for (const m of MOVIES) {
  const filePath = path.join(MOVIES_DIR, m.folder, m.filename)
  const st = fs.statSync(filePath)
  const mtime = Math.floor(st.mtimeMs / 1000)
  const fingerprint = `${st.size}-${mtime}`

  const posterKey = `w500/${m.key}_poster.jpg`
  const backdropKey = `w1280/${m.key}_backdrop.jpg`

  const movieSql = `
  INSERT INTO items (id, library_id, kind, title, sort_title, year, summary, runtime_s, rating, poster, backdrop, added_at, updated_at)
  VALUES (${m.id}, 1, 'movie', '${m.title.replace(/'/g, "''")}', '${m.title.replace(/'/g, "''")}', ${m.year}, '${m.summary.replace(/'/g, "''")}', ${m.runtime}, ${m.rating}, '${posterKey}', '${backdropKey}', ${now - m.id * 3600}, ${now});

  INSERT INTO files (id, item_id, path, size, mtime, fingerprint, container, duration_s, bitrate, probe_status)
  VALUES (${fileId}, ${m.id}, '${filePath.replace(/'/g, "''")}', ${st.size}, ${mtime}, '${fingerprint}', 'mp4', 180.0, 480000, 'ok');

  INSERT INTO streams (id, file_id, idx, type, codec, width, height, is_default)
  VALUES (${fileId * 10 + 1}, ${fileId}, 0, 'video', 'h264', 1920, 1080, 1);

  INSERT INTO streams (id, file_id, idx, type, codec, language, channels, is_default, title)
  VALUES (${fileId * 10 + 2}, ${fileId}, 1, 'audio', 'aac', 'eng', 2, 1, 'English (Stereo AAC)');

  INSERT INTO streams (id, file_id, idx, type, codec, language, is_default, title)
  VALUES (${fileId * 10 + 3}, ${fileId}, 2, 'subtitle', 'subrip', 'eng', 0, 'English SDH');

  INSERT INTO markers (file_id, kind, start_s, end_s, source)
  VALUES (${fileId}, 'intro', 5.0, 18.0, 'chapter');

  INSERT INTO markers (file_id, kind, start_s, end_s, source)
  VALUES (${fileId}, 'credits', 155.0, 175.0, 'black');
  `
  execSync(`sqlite3 "${DB_PATH}" "${movieSql}"`)

  if (m.position > 0 || m.watched > 0) {
    const watchSql = `
    INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at)
    VALUES (1, ${m.id}, ${m.position}, ${m.watched}, 1, ${now - m.id * 1800});
    `
    execSync(`sqlite3 "${DB_PATH}" "${watchSql}"`)
  }

  // Generate thumbnail for file
  const thumbPath = path.join(DATA_DIR, 'cache', 'thumbs', `${fileId}-${mtime}.jpg`)
  if (!fs.existsSync(thumbPath)) {
    try {
      execSync(`ffmpeg -y -ss 15 -i "${filePath}" -vframes 1 -q:v 2 "${thumbPath}"`, { stdio: 'ignore' })
    } catch (e) {}
  }

  fileId++
}

// Insert TV Show: Blender Open Stories
const showId = 10
const seasonId = 11
const showSql = `
INSERT INTO items (id, library_id, kind, title, sort_title, year, summary, rating, poster, backdrop, added_at, updated_at)
VALUES (${showId}, 2, 'show', 'Blender Open Stories', 'Blender Open Stories', 2024, 'An anthology of groundbreaking digital cinema showcasing open source animated narratives and visual exploration.', 8.8, 'w500/bbb_poster.jpg', 'w1280/sintel_backdrop.jpg', ${now - 7200}, ${now});

INSERT INTO items (id, library_id, parent_id, kind, title, sort_title, season_no, summary, added_at, updated_at)
VALUES (${seasonId}, 2, ${showId}, 'season', 'Season 1', 'Season 1', 1, 'The premier season introducing short-form narrative animation.', ${now - 7200}, ${now});
`
execSync(`sqlite3 "${DB_PATH}" "${showSql}"`)

let epId = 12
for (const ep of EPISODES) {
  const filePath = path.join(SHOWS_DIR, 'Blender Open Stories', 'Season 01', ep.file)
  const st = fs.statSync(filePath)
  const mtime = Math.floor(st.mtimeMs / 1000)
  const fingerprint = `${st.size}-${mtime}`

  const epSql = `
  INSERT INTO items (id, library_id, parent_id, kind, title, sort_title, season_no, episode_no, runtime_s, added_at, updated_at)
  VALUES (${epId}, 2, ${seasonId}, 'episode', '${ep.title}', '${ep.title}', 1, ${ep.ep}, 180, ${now - (4 - ep.ep) * 1200}, ${now});

  INSERT INTO files (id, item_id, path, size, mtime, fingerprint, container, duration_s, bitrate, probe_status)
  VALUES (${fileId}, ${epId}, '${filePath.replace(/'/g, "''")}', ${st.size}, ${mtime}, '${fingerprint}', 'mp4', 180.0, 480000, 'ok');

  INSERT INTO streams (id, file_id, idx, type, codec, width, height, is_default)
  VALUES (${fileId * 10 + 1}, ${fileId}, 0, 'video', 'h264', 1920, 1080, 1);

  INSERT INTO streams (id, file_id, idx, type, codec, language, channels, is_default, title)
  VALUES (${fileId * 10 + 2}, ${fileId}, 1, 'audio', 'aac', 'eng', 2, 1, 'English (Stereo AAC)');

  INSERT INTO markers (file_id, kind, start_s, end_s, source)
  VALUES (${fileId}, 'intro', 4.0, 16.0, 'chapter');
  `
  execSync(`sqlite3 "${DB_PATH}" "${epSql}"`)

  if (ep.ep === 1) {
    execSync(`sqlite3 "${DB_PATH}" "INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at) VALUES (1, ${epId}, 180.0, 1, 1, ${now - 3600});"`)
  } else if (ep.ep === 2) {
    execSync(`sqlite3 "${DB_PATH}" "INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at) VALUES (1, ${epId}, 65.0, 0, 1, ${now - 600});"`)
  }

  const thumbPath = path.join(DATA_DIR, 'cache', 'thumbs', `${fileId}-${mtime}.jpg`)
  if (!fs.existsSync(thumbPath)) {
    try {
      execSync(`ffmpeg -y -ss 15 -i "${filePath}" -vframes 1 -q:v 2 "${thumbPath}"`, { stdio: 'ignore' })
    } catch (e) {}
  }

  epId++
  fileId++
}

console.log('✅ Demo environment ready!')
console.log(`SESSION_TOKEN: ${SESSION_TOKEN}`)
