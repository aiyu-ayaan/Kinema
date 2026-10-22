// Plays a title in mpv: every codec decodes on this computer, so the server only serves the file's bytes
// (no remux, no transcode). mpv runs in its own window; the page waits and saves the progress mpv reports.
const { spawn } = require('child_process')
const crypto = require('crypto')
const fs = require('fs')
const http = require('http')
const net = require('net')
const os = require('os')
const path = require('path')

const isFile = (p) => {
  try {
    return fs.statSync(p).isFile()
  } catch {
    return false
  }
}

/**
 * Where a bundled mpv lives: Linux AppImage `$APPDIR/usr/bin/mpv`, then the app's resources (`resources/mpv/`,
 * filled at build time by scripts/fetch-mpv.mjs; same layout in the .deb and the Windows install), and on Windows
 * an `mpv.exe` next to the app.
 */
function bundledCandidates(env, platform, resources, execPath) {
  const name = platform === 'win32' ? 'mpv.exe' : 'mpv'
  const list = []
  if (platform !== 'win32' && env.APPDIR) list.push(path.join(env.APPDIR, 'usr', 'bin', name))
  if (resources) list.push(path.join(resources, 'mpv', name))
  if (platform === 'win32' && execPath) list.push(path.join(path.dirname(execPath), name))
  return list
}

/** mpv's binary: KINEMA_MPV, the copy bundled with the app, then the first `mpv` on PATH. */
function findMpv(env = process.env, platform = process.platform, resources = process.resourcesPath, execPath = process.execPath) {
  if (env.KINEMA_MPV && isFile(env.KINEMA_MPV)) return env.KINEMA_MPV
  for (const bundled of bundledCandidates(env, platform, resources, execPath)) if (isFile(bundled)) return bundled
  const name = platform === 'win32' ? 'mpv.exe' : 'mpv'
  for (const dir of (env.PATH || env.Path || '').split(path.delimiter)) {
    if (dir && isFile(path.join(dir, name))) return path.join(dir, name)
  }
  return null
}

/**
 * The environment mpv starts with. The bundled Linux copy is an AppImage tree with its own AppRun, which must not
 * see this app's AppImage variables (it would take them for its own mount point).
 */
function mpvEnv(env = process.env) {
  const clean = { ...env }
  for (const key of ['APPDIR', 'APPIMAGE', 'ARGV0', 'OWD']) delete clean[key]
  return clean
}

/** Response headers mpv needs for ranged reads; the rest (cookies, caching) stays behind. */
const PASSED = ['content-type', 'content-length', 'content-range', 'accept-ranges']

/**
 * A loopback server that forwards `GET /<nonce>/api/...` to `server` with `fetchUpstream`, the app's own
 * network stack: it sends the session cookie and checks the pinned certificate. So the token never sits on
 * mpv's command line, and mpv never has to skip certificate checks. The nonce keeps other local programs out.
 */
function startProxy(server, fetchUpstream) {
  const nonce = crypto.randomBytes(16).toString('hex')
  const prefix = `/${nonce}/api/`
  const proxy = http.createServer((req, res) => {
    if ((req.method !== 'GET' && req.method !== 'HEAD') || !req.url.startsWith(prefix)) {
      res.writeHead(404).end()
      return
    }
    const upstream = fetchUpstream(new URL(req.url.slice(nonce.length + 1), server).href, req.method, req.headers.range)
    upstream.on('response', (r) => {
      const headers = {}
      for (const name of PASSED) if (r.headers[name] != null) headers[name] = r.headers[name]
      res.writeHead(r.statusCode, headers)
      r.pipe(res)
    })
    upstream.on('error', () => res.destroy())
    // mpv seeks by dropping the connection and asking for a new range.
    res.on('close', () => upstream.abort())
    upstream.end()
  })
  return new Promise((resolve, reject) => {
    proxy.once('error', reject)
    proxy.listen(0, '127.0.0.1', () => {
      resolve({ base: `http://127.0.0.1:${proxy.address().port}/${nonce}`, close: () => proxy.close() })
    })
  })
}

const ipcPath = (id, platform = process.platform) =>
  platform === 'win32' ? `\\\\.\\pipe\\kinema-mpv-${id}` : path.join(os.tmpdir(), `kinema-mpv-${id}.sock`)

/** mpv's arguments for one title. Numbers only from the page; the title is passed as one argument, never parsed. */
function mpvArgs({ url, start, title, aid, subUrl, ipc }) {
  const args = [
    '--no-terminal',
    '--force-window=immediate',
    '--keep-open=no',
    // A URL from the server must never reach youtube-dl.
    '--ytdl=no',
    `--input-ipc-server=${ipc}`,
    `--force-media-title=${title}`,
  ]
  if (start > 0) args.push(`--start=${start}`)
  if (aid) args.push(`--aid=${aid}`)
  args.push(subUrl ? `--sub-file=${subUrl}` : '--sid=no')
  args.push('--', url)
  return args
}

/** Watches mpv's JSON IPC: the position as it plays, and whether the file played to its end. */
function watchIpc(ipc, onPosition, state) {
  let tries = 0
  const connect = () => {
    const socket = net.connect(ipc)
    let buffer = ''
    let connected = false
    socket.on('connect', () => {
      connected = true
      socket.write('{"command":["observe_property",1,"time-pos"]}\n')
    })
    socket.on('data', (chunk) => {
      buffer += chunk
      let nl
      while ((nl = buffer.indexOf('\n')) >= 0) {
        const line = buffer.slice(0, nl)
        buffer = buffer.slice(nl + 1)
        try {
          const msg = JSON.parse(line)
          if (msg.event === 'property-change' && msg.id === 1 && typeof msg.data === 'number') {
            state.position = msg.data
            onPosition(msg.data)
          } else if (msg.event === 'end-file' && msg.reason === 'eof') {
            state.ended = true
          }
        } catch {
          // A partial or non-JSON line: mpv only writes JSON, so nothing to do.
        }
      }
    })
    // The socket appears a moment after mpv starts.
    socket.on('error', () => {
      if (!connected && !state.exited && ++tries < 50) setTimeout(connect, 100)
    })
  }
  connect()
}

let current = null

/** Stops the running mpv, if any; its play() then resolves with where it was. */
function stop() {
  if (current) current.kill()
}

/**
 * Plays `fileId` from `server` and resolves once mpv closes: `{ position, ended }`, or `{ error }`.
 * `onPosition` gets the time as it plays. A new play() closes the previous mpv first.
 */
async function play({ mpv, server, fetchUpstream, fileId, start, title, aid, subtitleId, onPosition }) {
  stop()
  const proxy = await startProxy(server, fetchUpstream)
  const id = crypto.randomBytes(8).toString('hex')
  const ipc = ipcPath(id)
  const args = mpvArgs({
    url: `${proxy.base}/api/stream/${fileId}?raw=true`,
    start,
    title,
    aid,
    subUrl: subtitleId ? `${proxy.base}/api/subtitles/${subtitleId}` : null,
    ipc,
  })
  const state = { position: start, ended: false, exited: false }
  return new Promise((resolve) => {
    const child = spawn(mpv, args, { stdio: 'ignore', env: mpvEnv() })
    current = child
    let lastSent = 0
    watchIpc(
      ipc,
      (pos) => {
        // Every 10 s, as the in-page player does.
        if (Math.abs(pos - lastSent) >= 10) {
          lastSent = pos
          onPosition(pos)
        }
      },
      state,
    )
    const done = (result) => {
      if (state.exited) return
      state.exited = true
      if (current === child) current = null
      proxy.close()
      if (process.platform !== 'win32') fs.rm(ipc, { force: true }, () => {})
      resolve(result)
    }
    child.on('error', (error) => done({ error: error.message }))
    child.on('exit', () => done({ position: state.position, ended: state.ended }))
  })
}

module.exports = { findMpv, mpvEnv, startProxy, mpvArgs, ipcPath, play, stop }
