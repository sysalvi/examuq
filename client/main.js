const { app, BrowserWindow, ipcMain, session, shell } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const APP_STATE_FILE = 'client-state.json';

let launcherWindow = null;
let portalWindow = null;
let requestHeaderInterceptorAttached = false;
let portalOrigin = null;
let deviceIdCache = null;

function getStatePath() {
  return path.join(app.getPath('userData'), APP_STATE_FILE);
}

function readState() {
  try {
    const raw = fs.readFileSync(getStatePath(), 'utf-8');
    const parsed = JSON.parse(raw);

    if (!parsed || typeof parsed !== 'object') {
      return {};
    }

    return parsed;
  } catch {
    return {};
  }
}

function writeState(nextState) {
  fs.writeFileSync(getStatePath(), JSON.stringify(nextState, null, 2));
}

function normalizeBaseUrl(rawValue) {
  const input = String(rawValue || '').trim();

  if (!input) {
    throw new Error('URL server wajib diisi.');
  }

  const withProtocol = /^https?:\/\//i.test(input) ? input : `http://${input}`;
  const parsed = new URL(withProtocol);

  return `${parsed.protocol}//${parsed.host}`;
}

function getDeviceId() {
  if (deviceIdCache) {
    return deviceIdCache;
  }

  const state = readState();
  if (typeof state.deviceId === 'string' && state.deviceId.length > 0) {
    deviceIdCache = state.deviceId;
    return deviceIdCache;
  }

  const hashSeed = `${app.getName()}-${process.platform}-${app.getPath('home')}-${Date.now()}`;
  const newDeviceId = `desktop-${crypto.createHash('sha256').update(hashSeed).digest('hex').slice(0, 16)}`;
  const next = { ...state, deviceId: newDeviceId };
  writeState(next);
  deviceIdCache = newDeviceId;

  return deviceIdCache;
}

function registerHeaderInterceptor() {
  if (requestHeaderInterceptorAttached) {
    return;
  }

  const targetSession = session.defaultSession;

  targetSession.webRequest.onBeforeSendHeaders((details, callback) => {
    let currentOrigin = null;

    try {
      currentOrigin = new URL(details.url).origin;
    } catch {
      callback({ requestHeaders: details.requestHeaders });
      return;
    }

    if (!portalOrigin || currentOrigin !== portalOrigin) {
      callback({ requestHeaders: details.requestHeaders });
      return;
    }

    const headers = {
      ...details.requestHeaders,
      'X-ExamUQ-Client-Type': 'desktop_client',
      'X-ExamUQ-Source': 'client',
      'X-ExamUQ-Device-Id': getDeviceId(),
    };

    callback({ requestHeaders: headers });
  });

  requestHeaderInterceptorAttached = true;
}

function createLauncherWindow() {
  launcherWindow = new BrowserWindow({
    width: 540,
    height: 620,
    resizable: false,
    title: 'ExamUQ Client Launcher',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  launcherWindow.loadFile(path.join(__dirname, 'renderer', 'index.html'));
  launcherWindow.on('closed', () => {
    launcherWindow = null;
  });
}

function applyPortalGuards(windowRef, allowedOrigin) {
  windowRef.webContents.setWindowOpenHandler(() => {
    return { action: 'deny' };
  });

  windowRef.webContents.on('will-navigate', (event, nextUrl) => {
    try {
      const nextOrigin = new URL(nextUrl).origin;
      if (nextOrigin !== allowedOrigin) {
        event.preventDefault();
      }
    } catch {
      event.preventDefault();
    }
  });

  windowRef.webContents.on('will-attach-webview', (event) => {
    event.preventDefault();
  });

  windowRef.webContents.on('new-window', (event) => {
    event.preventDefault();
  });

  windowRef.webContents.on('before-input-event', (event, input) => {
    const ctrlOrMeta = input.control || input.meta;
    const key = (input.key || '').toLowerCase();

    if (!ctrlOrMeta) {
      return;
    }

    if (key === 't' || key === 'n' || key === 'w') {
      event.preventDefault();
    }
  });
}

function openPortalWindow(baseUrl) {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl);
  const origin = new URL(normalizedBaseUrl).origin;
  portalOrigin = origin;

  registerHeaderInterceptor();

  const state = readState();
  writeState({
    ...state,
    lastServerBaseUrl: normalizedBaseUrl,
    deviceId: getDeviceId(),
  });

  if (portalWindow && !portalWindow.isDestroyed()) {
    portalWindow.loadURL(`${normalizedBaseUrl}/`);
    portalWindow.focus();
    return normalizedBaseUrl;
  }

  portalWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    autoHideMenuBar: true,
    title: 'ExamUQ Client',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  applyPortalGuards(portalWindow, origin);
  portalWindow.loadURL(`${normalizedBaseUrl}/`);

  portalWindow.on('closed', () => {
    portalWindow = null;
  });

  if (launcherWindow && !launcherWindow.isDestroyed()) {
    launcherWindow.hide();
  }

  return normalizedBaseUrl;
}

ipcMain.handle('client:get-launcher-state', async () => {
  const state = readState();

  return {
    lastServerBaseUrl: state.lastServerBaseUrl || '',
    deviceId: getDeviceId(),
  };
});

ipcMain.handle('client:open-portal', async (_event, payload) => {
  try {
    const normalizedBaseUrl = openPortalWindow(payload?.serverBaseUrl);

    return {
      ok: true,
      serverBaseUrl: normalizedBaseUrl,
    };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : 'Gagal membuka portal ujian.',
    };
  }
});

ipcMain.handle('client:open-admin', async (_event, payload) => {
  try {
    const normalizedBaseUrl = normalizeBaseUrl(payload?.serverBaseUrl);
    await shell.openExternal(`${normalizedBaseUrl}/admin`);

    return { ok: true };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : 'Gagal membuka admin.',
    };
  }
});

app.whenReady().then(() => {
  createLauncherWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createLauncherWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
