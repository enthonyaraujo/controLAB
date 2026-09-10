const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
  calculateLGR: (payload) => ipcRenderer.invoke('calculate-lgr', payload),
  previewTransferFunction: (payload) => ipcRenderer.invoke('preview-transfer-function', payload),
  getPresets: () => ipcRenderer.invoke('get-presets'),
  calculateRouth: (payload) => ipcRenderer.invoke('calculate-routh', payload),
  getRouthPresets: () => ipcRenderer.invoke('get-routh-presets'),
  evaluateRouthK: (payload) => ipcRenderer.invoke('evaluate-routh-k', payload),
  runScientificAnalysis: (action, payload) => ipcRenderer.invoke('run-scientific-analysis', action, payload),
  saveImage: (payload) => ipcRenderer.invoke('save-image', payload),
  saveSVG: (payload) => ipcRenderer.invoke('save-svg', payload),
  copyImageToClipboard: (base64) => ipcRenderer.invoke('copy-image', base64),
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
  checkUpdates: (token) => ipcRenderer.invoke('check-github-updates', token),
  getSystemInfo: () => ipcRenderer.invoke('get-system-info'),
  downloadUpdatePackage: (payload) => ipcRenderer.invoke('download-update-package', payload),
  cancelUpdateDownload: () => ipcRenderer.invoke('cancel-update-download'),
  openUpdateFolder: (filePath) => ipcRenderer.invoke('open-update-folder', filePath),
  installUpdatePackage: (payload) => ipcRenderer.invoke('install-update-package', payload),
  onUpdateDownloadProgress: (callback) => {
    const handler = (event, data) => callback(data);
    ipcRenderer.on('on-update-download-progress', handler);
    return () => ipcRenderer.removeListener('on-update-download-progress', handler);
  },
});
