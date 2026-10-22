const { test } = require('node:test')
const assert = require('node:assert/strict')
const fs = require('fs')
const http = require('http')
const os = require('os')
const path = require('path')
const { findMpv, mpvArgs, mpvEnv, startProxy } = require('../src/mpv')

test('findMpv takes KINEMA_MPV, then PATH, else null', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'kinema-mpv-'))
  const bin = path.join(dir, 'mpv')
  fs.writeFileSync(bin, '')
  assert.equal(findMpv({ PATH: dir }, 'linux'), bin)
  assert.equal(findMpv({ KINEMA_MPV: bin, PATH: '' }, 'linux'), bin)
  assert.equal(findMpv({ KINEMA_MPV: path.join(dir, 'nope'), PATH: '' }, 'linux'), null)
  assert.equal(findMpv({ PATH: dir }, 'win32'), null)
  fs.rmSync(dir, { recursive: true })
})

test('findMpv order: KINEMA_MPV, bundled, then PATH', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'kinema-mpv-'))
  const make = (...parts) => {
    const file = path.join(root, ...parts)
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, '')
    return file
  }
  const onPath = make('path', 'mpv')
  const resourcesCopy = make('res', 'mpv', 'mpv')
  const appDirCopy = make('appdir', 'usr', 'bin', 'mpv')
  const override = make('own', 'mpv')
  const base = { PATH: path.dirname(onPath) }
  const res = path.join(root, 'res')

  assert.equal(findMpv({ ...base, KINEMA_MPV: override, APPDIR: path.join(root, 'appdir') }, 'linux', res), override)
  assert.equal(findMpv({ ...base, APPDIR: path.join(root, 'appdir') }, 'linux', res), appDirCopy)
  assert.equal(findMpv(base, 'linux', res), resourcesCopy)
  assert.equal(findMpv(base, 'linux', path.join(root, 'none')), onPath)
  // A broken KINEMA_MPV falls through to the bundled copy instead of hiding it.
  assert.equal(findMpv({ ...base, KINEMA_MPV: path.join(root, 'nope') }, 'linux', res), resourcesCopy)

  // Windows: resources/mpv/mpv.exe, or mpv.exe next to the app, before PATH.
  const winRes = make('winres', 'mpv', 'mpv.exe')
  const winPath = make('winpath', 'mpv.exe')
  const winEnv = { PATH: path.dirname(winPath) }
  assert.equal(findMpv(winEnv, 'win32', path.join(root, 'winres')), winRes)
  const beside = make('app', 'mpv.exe')
  assert.equal(findMpv(winEnv, 'win32', undefined, path.join(root, 'app', 'Kinema.exe')), beside)
  assert.equal(findMpv(winEnv, 'win32', undefined, path.join(root, 'elsewhere', 'Kinema.exe')), winPath)
  fs.rmSync(root, { recursive: true })
})

test('mpvEnv drops the AppImage variables and keeps the rest', () => {
  const env = mpvEnv({ APPDIR: '/a', APPIMAGE: '/b', ARGV0: 'c', OWD: '/d', HOME: '/h', DISPLAY: ':0' })
  assert.deepEqual(env, { HOME: '/h', DISPLAY: ':0' })
})

test('mpvArgs keeps the URL after -- and turns subtitles off without one', () => {
  const args = mpvArgs({ url: 'http://127.0.0.1:1/n/api/stream/7?raw=true', start: 12.5, title: '--fs', aid: 2, subUrl: null, ipc: '/tmp/s' })
  assert.deepEqual(args.slice(-2), ['--', 'http://127.0.0.1:1/n/api/stream/7?raw=true'])
  assert.ok(args.includes('--start=12.5'))
  assert.ok(args.includes('--aid=2'))
  assert.ok(args.includes('--sid=no'))
  assert.ok(args.includes('--force-media-title=--fs'))
  assert.ok(args.includes('--ytdl=no'))
  const withSub = mpvArgs({ url: 'u', start: 0, title: 't', aid: null, subUrl: 'http://x/s', ipc: '/tmp/s' })
  assert.ok(withSub.includes('--sub-file=http://x/s'))
  assert.ok(!withSub.some((a) => a.startsWith('--start') || a.startsWith('--aid')))
})

test('proxy forwards only nonce paths under /api, with the range', async () => {
  const seen = []
  const upstream = http.createServer((req, res) => {
    seen.push({ url: req.url, range: req.headers.range })
    res.writeHead(206, { 'content-type': 'video/mp4', 'content-range': 'bytes 0-1/10', 'set-cookie': 'x=1' })
    res.end('ab')
  })
  await new Promise((r) => upstream.listen(0, '127.0.0.1', r))
  const server = `http://127.0.0.1:${upstream.address().port}`
  const fetchUpstream = (url, method, range) => http.request(url, { method, headers: range ? { range } : {} })
  const proxy = await startProxy(server, fetchUpstream)
  const get = (url, headers = {}) =>
    new Promise((resolve) => http.get(url, { headers }, (res) => {
      let body = ''
      res.on('data', (c) => (body += c))
      res.on('end', () => resolve({ status: res.statusCode, headers: res.headers, body }))
    }))

  const ok = await get(`${proxy.base}/api/stream/7?raw=true`, { range: 'bytes=0-1' })
  assert.equal(ok.status, 206)
  assert.equal(ok.body, 'ab')
  assert.equal(ok.headers['content-range'], 'bytes 0-1/10')
  assert.equal(ok.headers['set-cookie'], undefined)
  assert.deepEqual(seen, [{ url: '/api/stream/7?raw=true', range: 'bytes=0-1' }])

  const origin = new URL(proxy.base).origin
  assert.equal((await get(`${origin}/api/stream/7`)).status, 404)
  assert.equal((await get(`${proxy.base}/other`)).status, 404)
  assert.equal(seen.length, 1)

  proxy.close()
  upstream.close()
})
