const { contextBridge, ipcRenderer } = require('electron')

// The UI is loaded from the Kinema server itself, so API calls stay same-origin.
const desktopBridge = {
  platform: process.platform,
  isDesktop: true,
  flavor: ipcRenderer.sendSync('flavor:get'),
  chooseServer: () => ipcRenderer.send('server:choose'),
  setTheme: (theme) => ipcRenderer.send('theme:set', theme),
  token: {
    get: () => ipcRenderer.sendSync('token:get'),
    set: (token) => ipcRenderer.send('token:set', token),
  },
  // Connect page helpers
  connect: (address) => ipcRenderer.invoke('server:connect', address),
  discover: () => ipcRenderer.invoke('server:discover'),
  cancel: () => ipcRenderer.send('server:cancel'),
  hasSavedServer: () => ipcRenderer.sendSync('server:hasSaved'),
  getTheme: () => ipcRenderer.sendSync('theme:get'),
  getMessages: () => ipcRenderer.sendSync('i18n:get'),
  // This app's own releases (main.js, updates.js).
  updates: {
    info: () => ipcRenderer.invoke('update:info'),
    check: () => ipcRenderer.invoke('update:check'),
    install: () => ipcRenderer.invoke('update:install'),
    skip: () => ipcRenderer.invoke('update:skip'),
    // Download progress, 0..1, then null. Returns the function that stops listening.
    onProgress: (listener) => {
      const heard = (_event, fraction) => listener(fraction)
      ipcRenderer.on('update:progress', heard)
      return () => ipcRenderer.removeListener('update:progress', heard)
    },
  },
}

contextBridge.exposeInMainWorld('kinema', desktopBridge)
contextBridge.exposeInMainWorld('desktop', desktopBridge)
