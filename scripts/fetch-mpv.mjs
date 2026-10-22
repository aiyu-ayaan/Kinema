#!/usr/bin/env node
// Fetches the pinned mpv for one platform into apps/desktop/build/mpv/, which electron-builder copies to
// resources/mpv/ (extraResources). Binaries never go into git. Pins live in scripts/mpv.lock.
//
//   node scripts/fetch-mpv.mjs [platform]            fetch (default: this machine)
//   node scripts/fetch-mpv.mjs --pin <platform> <url> download, print the SHA-256 and write it to the lock
//
// KINEMA_SKIP_MPV=1 builds without mpv. A platform that is not pinned is skipped with a warning; a download
// whose SHA-256 differs from the pin fails the build.
import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { createReadStream, createWriteStream, existsSync } from 'node:fs'
import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { Readable } from 'node:stream'
import { pipeline } from 'node:stream/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const lockPath = join(root, 'scripts', 'mpv.lock')
const outDir = join(root, 'apps', 'desktop', 'build', 'mpv')

export function parseLock(text) {
  const out = {}
  for (const line of text.split(/\r?\n/)) {
    const m = /^([A-Za-z0-9._-]+)=(.*)$/.exec(line.trim())
    if (m) out[m[1]] = m[2].trim()
  }
  return out
}

/** The pin for `platform`, or `null` while its url or sha256 is empty. */
export function pinFor(lock, platform) {
  const url = lock[`${platform}.url`]
  const sha256 = (lock[`${platform}.sha256`] ?? '').toLowerCase()
  const format = lock[`${platform}.format`]
  if (!url || !/^[0-9a-f]{64}$/.test(sha256) || !format) return null
  return { url, sha256, format, version: lock.version || 'unknown' }
}

export function detectPlatform(platform = process.platform, arch = process.arch) {
  if (platform === 'win32' && arch === 'x64') return 'windows-x64'
  if (platform === 'linux' && arch === 'x64') return 'linux-x86_64'
  if (platform === 'linux' && arch === 'arm64') return 'linux-aarch64'
  return null
}

export async function sha256File(file) {
  const hash = createHash('sha256')
  await pipeline(createReadStream(file), hash)
  return hash.digest('hex')
}

/** Throws unless `file` has the pinned hash. */
export async function verify(file, sha256) {
  const actual = await sha256File(file)
  if (actual !== sha256.toLowerCase()) throw new Error(`SHA-256 mismatch for ${file}: expected ${sha256}, got ${actual}`)
}

async function download(url, file) {
  if (!url.startsWith('https://')) throw new Error(`refusing a non-HTTPS download: ${url}`)
  const res = await fetch(url, { redirect: 'follow' })
  if (!res.ok || !res.body) throw new Error(`${url} answered ${res.status}`)
  await pipeline(Readable.fromWeb(res.body), createWriteStream(file))
}

function run(cmd, args, cwd) {
  const r = spawnSync(cmd, args, { cwd, stdio: 'inherit' })
  if (r.error || r.status !== 0) throw new Error(`${cmd} failed${r.error ? `: ${r.error.message}` : ''}`)
}

function sevenZip() {
  try {
    // electron-builder depends on 7zip-bin, so it is there wherever the desktop app builds.
    return createRequire(join(root, 'apps', 'desktop', 'package.json'))('7zip-bin').path7za
  } catch {
    return process.platform === 'win32' ? '7z.exe' : '7z'
  }
}

async function findFile(dir, name) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name)
    if (entry.isDirectory()) {
      const hit = await findFile(p, name)
      if (hit) return hit
    } else if (entry.name === name) return p
  }
  return null
}

export function noticeText({ version, url }) {
  return `mpv ${version}
mpv is bundled with Kinema Desktop as a separate program. Kinema starts it as its own process and never links it.
License: GPL-2.0-or-later (this build; the mpv project also offers LGPL-2.1-or-later builds). The full text is in
the license files shipped in this folder, or at https://www.gnu.org/licenses/.
Project and source code: https://github.com/mpv-player/mpv
This build: ${url}
`
}

/** The linux wrapper: the extracted AppImage tree starts through its AppRun, which finds its libraries beside itself. */
const LINUX_WRAPPER = '#!/bin/sh\nexec "$(dirname "$(readlink -f "$0")")/AppRun" "$@"\n'

async function extract(pin, archive, platform) {
  const work = await mkdtemp(join(tmpdir(), 'kinema-mpv-x-'))
  try {
    if (pin.format === '7z') {
      run(sevenZip(), ['x', '-y', `-o${work}`, archive])
      const exe = await findFile(work, 'mpv.exe')
      if (!exe) throw new Error('the archive has no mpv.exe')
      // The folder holding mpv.exe is the program: its DLLs and license files sit beside it.
      await copyDir(dirname(exe), outDir)
    } else if (pin.format === 'appimage') {
      await chmod(archive, 0o755)
      run(archive, ['--appimage-extract'], work)
      const tree = join(work, 'squashfs-root')
      if (!existsSync(join(tree, 'AppRun'))) throw new Error('the AppImage has no AppRun')
      await copyDir(tree, outDir)
      await writeFile(join(outDir, 'mpv'), LINUX_WRAPPER)
      await chmod(join(outDir, 'mpv'), 0o755)
    } else {
      throw new Error(`unknown format ${pin.format} for ${platform}`)
    }
  } finally {
    await rm(work, { recursive: true, force: true })
  }
}

async function copyDir(from, to) {
  await mkdir(to, { recursive: true })
  for (const entry of await readdir(from, { withFileTypes: true })) {
    const src = join(from, entry.name)
    const dst = join(to, entry.name)
    if (entry.isDirectory()) await copyDir(src, dst)
    else if (entry.isFile()) await copyFile(src, dst)
    // Symlinks are skipped: extraResources would copy them as links that break outside the AppImage tree.
  }
}

async function fetchMpv(platform) {
  // electron-builder takes the folder as is, so it must exist even when there is nothing in it.
  await mkdir(outDir, { recursive: true })
  if (process.env.KINEMA_SKIP_MPV === '1') return console.log('mpv: KINEMA_SKIP_MPV=1, building without it')
  const pin = pinFor(parseLock(await readFile(lockPath, 'utf8')), platform ?? '')
  if (!pin) {
    return console.warn(`mpv: ${platform ?? 'this platform'} is not pinned in scripts/mpv.lock; the app will use mpv from PATH`)
  }
  const stamp = join(outDir, '.pinned')
  if (existsSync(stamp) && (await readFile(stamp, 'utf8')).trim() === pin.sha256) return console.log('mpv: up to date')

  await rm(outDir, { recursive: true, force: true })
  await mkdir(outDir, { recursive: true })
  const tmp = await mkdtemp(join(tmpdir(), 'kinema-mpv-dl-'))
  try {
    const archive = join(tmp, 'mpv-download')
    console.log(`mpv: downloading ${pin.url}`)
    await download(pin.url, archive)
    await verify(archive, pin.sha256)
    await extract(pin, archive, platform)
    await writeFile(join(outDir, 'NOTICE-mpv.txt'), noticeText(pin))
    await writeFile(stamp, `${pin.sha256}\n`)
    console.log(`mpv ${pin.version} ready in ${outDir}`)
  } catch (e) {
    await rm(outDir, { recursive: true, force: true })
    await mkdir(outDir, { recursive: true })
    throw e
  } finally {
    await rm(tmp, { recursive: true, force: true })
  }
}

async function pin(platform, url) {
  if (!platform || !url) throw new Error('usage: fetch-mpv.mjs --pin <platform> <url>')
  const tmp = await mkdtemp(join(tmpdir(), 'kinema-mpv-pin-'))
  try {
    const file = join(tmp, 'download')
    await download(url, file)
    const sha = await sha256File(file)
    let lock = await readFile(lockPath, 'utf8')
    const set = (key, value) => {
      const re = new RegExp(`^${key.replace('.', '\\.')}=.*$`, 'm')
      if (!re.test(lock)) throw new Error(`${key} is not in the lock`)
      lock = lock.replace(re, `${key}=${value}`)
    }
    set(`${platform}.url`, url)
    set(`${platform}.sha256`, sha)
    await writeFile(lockPath, lock)
    console.log(`${platform}: ${sha}\nCompare it with the publisher's checksum before committing.`)
  } finally {
    await rm(tmp, { recursive: true, force: true })
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2)
  const job = args[0] === '--pin' ? pin(args[1], args[2]) : fetchMpv(args[0] ?? detectPlatform())
  job.catch((e) => {
    console.error(`mpv: ${e.message}`)
    process.exit(1)
  })
}
