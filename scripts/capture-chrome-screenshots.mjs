import { spawn } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

const SCREENSHOTS_DIR = path.join(process.cwd(), 'assets', 'screenshots')
fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true })

const TOKEN = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'
const PORT = 9222

console.log('Launching system Google Chrome in headless debugging mode...')
const chrome = spawn('google-chrome', [
  '--headless=new',
  `--remote-debugging-port=${PORT}`,
  '--ignore-certificate-errors',
  '--no-sandbox',
  '--disable-gpu',
  '--disable-background-networking',
  '--window-size=1920,1080',
  '--user-data-dir=/tmp/kinema_chrome_profile'
], { stdio: 'ignore' })

process.on('exit', () => {
  try { chrome.kill() } catch (e) {}
})

// Wait for port
async function waitPort() {
  for (let i = 0; i < 20; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${PORT}/json/version`)
      if (res.ok) return await res.json()
    } catch (e) {}
    await new Promise(r => setTimeout(r, 200))
  }
  throw new Error('Chrome failed to start on port ' + PORT)
}

const versionInfo = await waitPort()
console.log('Connected to Chrome:', versionInfo['Browser'])

// Open new target
const targetRes = await fetch(`http://127.0.0.1:${PORT}/json/new?https://127.0.0.1:3000/`, { method: 'PUT' })
const target = await targetRes.json()
const wsUrl = target.webSocketDebuggerUrl

const ws = new WebSocket(wsUrl)
await new Promise((resolve, reject) => {
  ws.onopen = resolve
  ws.onerror = reject
})

let id = 1
const callbacks = new Map()

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data)
  if (msg.id && callbacks.has(msg.id)) {
    const cb = callbacks.get(msg.id)
    callbacks.delete(msg.id)
    if (msg.error) cb.reject(new Error(JSON.stringify(msg.error)))
    else cb.resolve(msg.result)
  }
}

function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const msgId = id++
    callbacks.set(msgId, { resolve, reject })
    ws.send(JSON.stringify({ id: msgId, method, params }))
  })
}

// Enable CDP domains
await send('Page.enable')
await send('Runtime.enable')
await send('Network.enable')

// Set HiDPI viewport
await send('Emulation.setDeviceMetricsOverride', {
  width: 1920,
  height: 1080,
  deviceScaleFactor: 2,
  mobile: false
})

// Set cookie & localStorage
await send('Network.setCookie', {
  name: 'kinema_session',
  value: TOKEN,
  domain: '127.0.0.1',
  path: '/',
  secure: true
})

// Navigate and inject localStorage
console.log('Navigating to Kinema web app...')
await send('Page.navigate', { url: 'https://127.0.0.1:3000/' })
await new Promise(r => setTimeout(r, 1500))

await send('Runtime.evaluate', {
  expression: `
    localStorage.setItem('kinema.token', '${TOKEN}');
    localStorage.setItem('kinema.dev.token', '${TOKEN}');
    localStorage.setItem('kinema.theme', 'dark');
    localStorage.setItem('kinema.dev.theme', 'dark');
    document.documentElement.dataset.theme = 'dark';
  `
})

// Reload with token and dark theme active
await send('Page.navigate', { url: 'https://127.0.0.1:3000/' })
await new Promise(r => setTimeout(r, 2500))

await send('Runtime.evaluate', {
  expression: `
    localStorage.setItem('kinema.theme', 'dark');
    localStorage.setItem('kinema.dev.theme', 'dark');
    document.documentElement.dataset.theme = 'dark';
  `
})

async function capture(pageUrl, filename, waitMs = 2500) {
  console.log(`Navigating to ${pageUrl}...`)
  await send('Page.navigate', { url: pageUrl })
  await new Promise(r => setTimeout(r, waitMs))
  
  console.log(`Capturing ${filename}...`)
  const result = await send('Page.captureScreenshot', { format: 'png' })
  const buffer = Buffer.from(result.data, 'base64')
  const outPath = path.join(SCREENSHOTS_DIR, filename)
  fs.writeFileSync(outPath, buffer)
  console.log(`Saved: ${outPath} (${(buffer.length / 1024).toFixed(1)} KB)`)
}

// 1. Web Home Page
await capture('https://127.0.0.1:3000/', 'web-home.png', 3000)

// 2. Movie Details Page (Sintel)
await capture('https://127.0.0.1:3000/item/1', 'web-details.png', 3000)

// 3. Movies Library Page
await capture('https://127.0.0.1:3000/library/1', 'web-library.png', 3000)

// 4. Video Player (Sintel)
await capture('https://127.0.0.1:3000/play/1', 'web-player.png', 4000)

// 5. Settings Page
await capture('https://127.0.0.1:3000/settings', 'web-settings.png', 2500)

ws.close()
chrome.kill()
console.log('Chrome screenshots complete!')
