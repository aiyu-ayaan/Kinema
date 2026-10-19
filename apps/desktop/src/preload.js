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
}

contextBridge.exposeInMainWorld('kinema', desktopBridge)
contextBridge.exposeInMainWorld('desktop', desktopBridge)
