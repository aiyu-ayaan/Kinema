const { test } = require('node:test')
const assert = require('node:assert/strict')
const fs = require('fs')
const os = require('os')
const path = require('path')
const { pathToFileURL } = require('url')

const script = path.join(__dirname, '..', '..', '..', 'scripts', 'fetch-mpv.mjs')
const load = () => import(pathToFileURL(script).href)
const HASH = 'a'.repeat(64)

test('parseLock and pinFor read a pinned platform and skip an empty one', async () => {
  const { parseLock, pinFor } = await load()
  const lock = parseLock(
    ['# comment', 'version=0.40', `windows-x64.format=7z`, 'windows-x64.url=https://x/mpv.7z', `windows-x64.sha256=${HASH.toUpperCase()}`, 'linux-x86_64.format=appimage', 'linux-x86_64.url=', 'linux-x86_64.sha256='].join('\n'),
  )
  assert.deepEqual(pinFor(lock, 'windows-x64'), { url: 'https://x/mpv.7z', sha256: HASH, format: '7z', version: '0.40' })
  assert.equal(pinFor(lock, 'linux-x86_64'), null)
  assert.equal(pinFor(lock, 'linux-aarch64'), null)
  assert.equal(pinFor(parseLock('a.url=u\na.format=7z\na.sha256=abc'), 'a'), null, 'a short hash is not a pin')
})

test('the shipped lock parses and no platform carries a half-written pin', async () => {
  const { parseLock, pinFor } = await load()
  const lock = parseLock(fs.readFileSync(path.join(path.dirname(script), 'mpv.lock'), 'utf8'))
  for (const platform of ['windows-x64', 'linux-x86_64', 'linux-aarch64']) {
    assert.ok(lock[`${platform}.format`], `${platform} has a format`)
    const filled = [lock[`${platform}.url`], lock[`${platform}.sha256`]].filter(Boolean).length
    assert.notEqual(filled, 1, `${platform} sets both url and sha256 or neither`)
    if (filled === 2) assert.ok(pinFor(lock, platform), `${platform} has a valid pin`)
  }
})

test('verify accepts the right SHA-256 and rejects any other', async () => {
  const { verify, sha256File } = await load()
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'kinema-fetch-'))
  const file = path.join(dir, 'f')
  fs.writeFileSync(file, 'abc')
  const sha = 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'
  assert.equal(await sha256File(file), sha)
  await verify(file, sha)
  await assert.rejects(verify(file, HASH), /mismatch/)
  fs.rmSync(dir, { recursive: true })
})

test('detectPlatform maps the supported machines only', async () => {
  const { detectPlatform } = await load()
  assert.equal(detectPlatform('win32', 'x64'), 'windows-x64')
  assert.equal(detectPlatform('linux', 'x64'), 'linux-x86_64')
  assert.equal(detectPlatform('linux', 'arm64'), 'linux-aarch64')
  assert.equal(detectPlatform('darwin', 'arm64'), null)
})

test('noticeText names the license, the source and the build', async () => {
  const { noticeText } = await load()
  const text = noticeText({ version: '0.40', url: 'https://x/mpv.7z' })
  for (const part of ['GPL', 'LGPL', 'https://github.com/mpv-player/mpv', 'https://x/mpv.7z', '0.40']) assert.ok(text.includes(part), part)
})
