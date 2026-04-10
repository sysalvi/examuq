const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('examuqClient', {
  getLauncherState: () => ipcRenderer.invoke('client:get-launcher-state'),
  updateSettings: (serverBaseUrl) => ipcRenderer.invoke('client:update-settings', { serverBaseUrl }),
  startExam: () => ipcRenderer.invoke('client:start-exam'),
  openAdmin: () => ipcRenderer.invoke('client:open-admin'),
});
