const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
  calculateLGR: (payload) => ipcRenderer.invoke('calculate-lgr', payload),
  previewTransferFunction: (payload) => ipcRenderer.invoke('preview-transfer-function', payload),
  getPresets: () => ipcRenderer.invoke('get-presets'),
  saveImage: (payload) => ipcRenderer.invoke('save-image', payload),
  saveSVG: (payload) => ipcRenderer.invoke('save-svg', payload),
  copyImageToClipboard: (base64) => ipcRenderer.invoke('copy-image', base64),
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
  checkUpdates: (token) => ipcRenderer.invoke('check-github-updates', token),
  getSystemInfo: () => ipcRenderer.invoke('get-system-info'),
});
