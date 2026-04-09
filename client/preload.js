const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('examuqClient', {
  getLauncherState: () => ipcRenderer.invoke('client:get-launcher-state'),
  openPortal: (serverBaseUrl) => ipcRenderer.invoke('client:open-portal', { serverBaseUrl }),
  openAdmin: (serverBaseUrl) => ipcRenderer.invoke('client:open-admin', { serverBaseUrl }),
});
