// Runs the backend and restarts it when Rust sources, migrations or locales change.
// Plain Node (fs.watch recursive), so dev needs no cargo-watch install.
import { spawn, spawnSync } from 'node:child_process'
import { watch } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const watched = ['backend', 'crates', 'Cargo.toml']
const relevant = /\.(rs|toml|sql|json)$/

let child = null
let timer = null
let restarting = false

function start() {
  // detached: cargo and the server get their own process group so a restart stops both.
  child = spawn('cargo', ['run', '--manifest-path', 'backend/Cargo.toml'], {
    cwd: root,
    stdio: 'inherit',
    detached: process.platform !== 'win32',
  })
  child.on('exit', (code, signal) => {
    if (!restarting && signal == null) console.log(`[dev-backend] exited with ${code}; waiting for changes`)
    child = null
  })
}

function stop() {
  return new Promise((resolve) => {
    if (!child) return resolve()
    child.once('exit', () => resolve())
    if (process.platform === 'win32') spawnSync('taskkill', ['/pid', String(child.pid), '/T', '/F'])
    else process.kill(-child.pid, 'SIGTERM')
  })
}

async function restart(file) {
  restarting = true
  console.log(`[dev-backend] ${file} changed, restarting`)
  await stop()
  restarting = false
  start()
}

for (const entry of watched) {
  watch(path.join(root, entry), { recursive: true }, (_event, file) => {
    if (!file || !relevant.test(file) || file.includes('target')) return
    clearTimeout(timer)
    timer = setTimeout(() => restart(path.join(entry, file)), 300)
  })
}

for (const sig of ['SIGINT', 'SIGTERM']) {
  process.on(sig, async () => {
    await stop()
    process.exit(0)
  })
}

start()
