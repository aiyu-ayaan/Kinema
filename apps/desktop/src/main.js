const electron = require('electron')
const { app, BrowserWindow, Menu, dialog, ipcMain, nativeTheme, shell, session } =
  typeof electron === 'object' && electron !== null ? electron : {}
const dgram = require('dgram')
const fs = require('fs')
const http = require('http')
const https = require('https')
const net = require('net')
const os = require('os')
const path = require('path')
const { pathToFileURL } = require('url')
const { t, setLanguage, getLanguage, messages } = require('./i18n')

// The window shows the web UI served by the Kinema server itself.
// The server is chosen on the connect page (connect.html) and remembered.
const override = process.env.KINEMA_URL
const CONNECT = path.join(__dirname, 'connect.html')
const BEACON_PORT = 48555 // backend/src/discovery.rs

// App Flavors: 'dev' or 'prod'.
const flavor = process.env.KINEMA_FLAVOR || (!app || !app.isPackaged ? 'dev' : 'prod')
const isDev = flavor === 'dev'

// Set distinct app name and isolate userData per flavor
if (app) {
  const appData = app.getPath('appData')
  const userDataName = isDev ? 'KinemaDev' : 'Kinema'
  app.setPath('userData', path.join(appData, userDataName))
  app.setName(isDev ? 'Kinema Dev' : 'Kinema')
  if (app.setAppUserModelId) {
    app.setAppUserModelId(isDev ? 'com.aiyu.kinema.dev' : 'com.aiyu.kinema')
  }

  // Single instance lock per flavor
  if (!app.requestSingleInstanceLock()) app.exit(0)
  app.on('second-instance', () => showWindow())
}

/** servers: origins to try in order. pins: origin -> approved self-signed certificate (SHA-256, AA:BB:..). */
let config = { servers: [], pins: {}, token: null }
let win = null

const configFile = () =>
  app ? path.join(app.getPath('userData'), isDev ? 'kinema-dev.json' : 'kinema.json') : ''
const save = () => {
  if (!configFile()) return
  try {
    fs.writeFileSync(configFile(), JSON.stringify(config, null, 2), { mode: 0o600 })
  } catch (err) {
    console.error('Failed to save config:', err)
  }
}

const applyLanguage = () => setLanguage(config.language, app.getPreferredSystemLanguages(), app.getLocale())

const origin = (url) => {
  try {
    return new URL(url).origin
  } catch {
    return null
  }
}
const known = (url) => !!origin(url) && (config.servers.includes(origin(url)) || origin(url) === origin(override))

/** IPC is answered for the connect page and for pages of a chosen server only. */
const trusted = (event) => {
  const url = event.senderFrame?.url ?? ''
  return url.startsWith(pathToFileURL(CONNECT).href) || known(url)
}

function isLanHost(host) {
  if (host === 'localhost' || host.endsWith('.local') || host === '10.0.2.2' || host === '127.0.0.1' || host === '::1') return true
  const p = host.split('.').map(Number)
  if (p.length !== 4 || p.some(isNaN)) return false
  return p[0] === 10 || p[0] === 127 || (p[0] === 192 && p[1] === 168) || (p[0] === 172 && p[1] >= 16 && p[1] <= 31)
}

function normalize(input) {
  let s = String(input ?? '').trim().replace(/\/+$/, '')
  if (!s || /\s/.test(s)) return null
  const explicit = s.startsWith('https://') || s.startsWith('http://')
  if (!explicit) s = `https://${s}`
  try {
    const url = new URL(s)
    if (!url.hostname) return null
    if (!url.port && !explicit && isLanHost(url.hostname)) {
      url.port = '3000'
    }
    return url.origin
  } catch {
    return null
  }
}

function parseReply(text, host) {
  try {
    const reply = JSON.parse(text)
    if (reply.service !== 'kinema') return null
    const port = Number(reply.port)
    if (!port || port < 1 || port > 65535) return null
    const scheme = reply.tls === false ? 'http' : 'https'
    const name = String(reply.name || 'Kinema').slice(0, 60).trim() || 'Kinema'
    const h = host.includes(':') ? `[${host.split('%')[0]}]` : host
    return {
      name,
      url: `${scheme}://${h}:${port}`,
      version: reply.version || null,
    }
  } catch {
    return null
  }
}

/** Asks `url` for its health. `authorized` is false for a certificate no public authority vouches for. */
function probe(url, timeout = 3000) {
  return new Promise((resolve) => {
    const secure = url.startsWith('https:')
    const request = (secure ? https : http).get(`${url}/api/server/info`, { rejectUnauthorized: false, timeout }, (res) => {
      resolve({
        ok: res.statusCode === 200,
        authorized: !secure || res.socket.authorized,
        fingerprint: secure && res.socket.getPeerCertificate ? res.socket.getPeerCertificate().fingerprint256 : null,
      })
      res.resume()
    })
    request.on('timeout', () => request.destroy())
    request.on('error', () => resolve({ ok: false }))
  })
}

const accepts = (url, p) => p.ok && (p.authorized || (!!p.fingerprint && config.pins[url] === p.fingerprint))

const chromiumFingerprint = (pin) => 'sha256/' + Buffer.from(pin.replace(/:/g, ''), 'hex').toString('base64')

function verifyCertificate({ hostname, certificate, verificationResult }, callback) {
  const approved =
    verificationResult !== 'net::OK' &&
    (Object.values(config.pins).some((pin) => chromiumFingerprint(pin) === certificate.fingerprint) ||
      (!!override && hostname === new URL(override).hostname))
  callback(approved ? 0 : -3)
}

// Window overlay title bar matching Kinema Ember dark and light themes
const titleBar = () =>
  nativeTheme && nativeTheme.shouldUseDarkColors
    ? { color: '#1A110F', symbolColor: '#F1DFDB', height: 35 }
    : { color: '#FFF8F6', symbolColor: '#231917', height: 35 }

if (nativeTheme) {
  nativeTheme.on('updated', () => win?.setTitleBarOverlay(titleBar()))
}

if (ipcMain) {
  ipcMain.on('theme:set', (event, theme) => {
    if (trusted(event) && ['system', 'light', 'dark'].includes(theme)) nativeTheme.themeSource = theme
  })
  ipcMain.on('theme:get', (event) => {
    event.returnValue = nativeTheme ? nativeTheme.themeSource : 'dark'
  })
}

function showConnect(message = '') {
  if (!win) return createWindow()
  win.loadFile(CONNECT, {
    query: {
      message,
      flavor: isDev ? 'dev' : 'prod',
      theme: nativeTheme.themeSource,
    },
  })
}

/** Opens the saved address that answers first. */
async function start() {
  const urls = override ? [override] : config.servers
  if (!urls.length) return showConnect()

  const url = await Promise.any(
    urls.map(async (u) => {
      const p = await probe(u)
      if (!(override ? p.ok : accepts(u, p))) throw new Error(u)
      return u
    }),
  ).catch(() => null)

  if (url) {
    return win.loadURL(url).catch(() => showConnect(t('connect.cannotLoad', { url })))
  }
  const tried = urls.map((u) => {
    try {
      return new URL(u).host
    } catch {
      return u
    }
  }).join(t('connect.hostOr', ' or '))
  showConnect(tried ? t('connect.cannotReach', { hosts: tried }) : '')
}

/** Kinema servers on local network: broadcasts probe, servers answer with port. */
function discover(wait = 1500) {
  return new Promise((resolve) => {
    const found = new Map()
    const socket = dgram.createSocket('udp4')
    socket.on('error', () => {})
    socket.on('message', (message, from) => {
      const parsed = parseReply(message.toString('utf8'), from.address)
      if (parsed) found.set(parsed.url, parsed)
    })
    socket.bind(() => {
      socket.setBroadcast(true)
      const broadcasts = Object.values(os.networkInterfaces())
        .flat()
        .filter((a) => a && a.family === 'IPv4' && !a.internal)
        .map((a) => {
          const ip = a.address.split('.').map(Number)
          const mask = a.netmask.split('.').map(Number)
          return ip.map((part, i) => part | (~mask[i] & 255)).join('.')
        })
      for (const address of new Set([...broadcasts, '255.255.255.255'])) {
        socket.send('KINEMA_DISCOVER', BEACON_PORT, address, () => {})
      }
    })

    setTimeout(async () => {
      socket.close()
      // If nothing was found, check localhost/loopback
      if (found.size === 0) {
        for (const host of ['127.0.0.1', 'localhost']) {
          const testUrl = `https://${host}:3000`
          const p = await probe(testUrl, 400)
          if (p.ok) {
            found.set(testUrl, { name: 'Kinema', url: testUrl, thisComputer: true })
            break
          }
        }
      }
      resolve([...found.values()])
    }, wait)
  })
}

if (ipcMain) {
  ipcMain.handle('server:discover', (event) => (trusted(event) ? discover() : []))

  // Connect to a server address, prompting for TLS trust if self-signed.
  ipcMain.handle('server:connect', async (event, address) => {
    if (!trusted(event)) return ''
    const url = normalize(address)
    if (!url) return t('connect.addressInvalid')
    const p = await probe(url, 6000)
    const host = new URL(url).host
    if (!p.ok) return t('connect.noAnswer', { host })

    if (!accepts(url, p)) {
      const { response } = await dialog.showMessageBox(win, {
        type: 'question',
        buttons: [t('cert.cancel'), t('cert.trust')],
        defaultId: 1,
        cancelId: 0,
        message: t('cert.title'),
        detail: t('cert.detail', { host, fingerprint: p.fingerprint }),
      })
      if (response !== 1) return ''
      config.pins[url] = p.fingerprint
    }

    // Another server has its own accounts; update current server
    if (!config.servers.includes(url)) {
      config = { ...config, servers: [url], token: null }
    }
    save()
    win.loadURL(url).catch(() => showConnect(t('connect.cannotLoad', { url })))
    return ''
  })

  ipcMain.on('server:choose', (event) => {
    if (trusted(event)) showConnect()
  })
  ipcMain.on('server:cancel', (event) => {
    if (trusted(event)) start()
  })
  ipcMain.on('server:hasSaved', (event) => {
    event.returnValue = trusted(event) && (config.servers.length > 0 || !!override)
  })

  ipcMain.on('token:get', (event) => {
    event.returnValue = trusted(event) ? config.token : null
  })
  ipcMain.on('token:set', (event, token) => {
    if (trusted(event)) {
      config.token = token
      save()
    }
  })

  ipcMain.on('flavor:get', (event) => {
    event.returnValue = isDev ? 'dev' : 'prod'
  })

  ipcMain.on('i18n:get', (event) => {
    event.returnValue = trusted(event) ? messages() : {}
  })
  ipcMain.on('i18n:language', (event) => {
    event.returnValue = trusted(event) ? getLanguage() : 'en'
  })
  ipcMain.on('i18n:set', (event, tag) => {
    if (!trusted(event) || typeof tag !== 'string') return
    config.language = tag
    save()
    applyLanguage()
  })
}

function appIcon() {
  const iconFile = isDev ? 'icon-dev.png' : 'icon.png'
  const isPackaged = app ? app.isPackaged : false
  const p = isPackaged ? path.join(process.resourcesPath, iconFile) : path.join(__dirname, '../build', iconFile)
  return fs.existsSync(p) ? p : path.join(__dirname, '../build/icon.png')
}

function showWindow() {
  if (!app || !app.isReady()) return
  if (!win) return createWindow()
  if (win.isMinimized()) win.restore()
  win.show()
  win.focus()
}

function removeDefaultMenu(menuModule = Menu) {
  if (menuModule && typeof menuModule.setApplicationMenu === 'function') {
    menuModule.setApplicationMenu(null)
  }
}

function createWindow() {
  win = new BrowserWindow({
    title: isDev ? 'Kinema (Dev)' : 'Kinema',
    width: 1280,
    height: 800,
    minWidth: 400,
    minHeight: 500,
    titleBarStyle: 'hidden',
    titleBarOverlay: titleBar(),
    backgroundColor: titleBar().color,
    icon: appIcon(),
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
    },
  })

  // REMOVE ALL DEFAULT MENU ITEMS
  win.setMenu(null)
  if (win.removeMenu) win.removeMenu()

  if (isDev) {
    win.on('page-title-updated', (event, title) => {
      event.preventDefault()
      win.setTitle(title ? `${title} (Dev)` : 'Kinema (Dev)')
    })
  }

  win.once('ready-to-show', () => win.show())
  win.on('closed', () => (win = null))

  win.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url)
    return { action: 'deny' }
  })
  win.webContents.on('will-navigate', (event, url) => {
    if (!known(url)) {
      event.preventDefault()
      shell.openExternal(url)
    }
  })

  win.webContents.on('did-fail-load', (_event, errorCode, errorDescription, validatedURL) => {
    if (errorCode !== -3 && validatedURL && !validatedURL.startsWith('devtools://')) {
      console.warn('Page failed to load:', errorCode, errorDescription, validatedURL)
      showConnect(t('connect.cannotLoad', { url: validatedURL }))
    }
  })

  if (isDev) {
    win.webContents.on('before-input-event', (_event, input) => {
      if (input.key === 'F12' || (input.control && input.shift && input.key.toLowerCase() === 'i')) {
        win.webContents.toggleDevTools()
      }
    })
  }

  start()
}

if (app) {
  app.whenReady().then(() => {
    try {
      if (configFile() && fs.existsSync(configFile())) {
        config = { ...config, ...JSON.parse(fs.readFileSync(configFile(), 'utf8')) }
      }
    } catch (err) {
      console.error('Failed to read config:', err)
    }
    applyLanguage()

    // Remove default application menu globally (File, Edit, View, Window, Help)
    removeDefaultMenu(Menu)

    session.defaultSession.setCertificateVerifyProc(verifyCertificate)

    createWindow()
    app.on('activate', () => showWindow())
  })

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit()
  })
}

// Export helper functions for automated testing
module.exports = {
  isLanHost,
  normalize,
  parseReply,
  accepts,
  chromiumFingerprint,
  removeDefaultMenu,
}
