const { contextBridge, ipcRenderer } = require('electron');
contextBridge.exposeInMainWorld('figureApi', {
  getData: () => ipcRenderer.invoke('get-script-figure'),
});
