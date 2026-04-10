const { app, BrowserWindow, ipcMain, session, shell, globalShortcut } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const APP_STATE_FILE = 'client-state.json';
const DEFAULT_SERVER_BASE_URL = 'http://10.10.20.2:6996';
const SHUTDOWN_ARM_WINDOW_MS = 5000;

let launcherWindow = null;
let portalWindow = null;
let requestHeaderInterceptorAttached = false;
let portalOrigin = null;
let deviceIdCache = null;
let forceClosePortalWindow = false;
let shutdownArmTimestamp = 0;

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

function ensureClientState() {
  const state = readState();
  const normalizedBaseUrl = state.serverBaseUrl
    ? normalizeBaseUrl(state.serverBaseUrl)
    : DEFAULT_SERVER_BASE_URL;

  const nextState = {
    ...state,
    serverBaseUrl: normalizedBaseUrl,
  };

  writeState(nextState);

  return nextState;
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

function getCurrentServerBaseUrl() {
  const state = ensureClientState();

  return state.serverBaseUrl;
}

function setCurrentServerBaseUrl(rawValue) {
  const normalizedBaseUrl = normalizeBaseUrl(rawValue);
  const state = readState();

  writeState({
    ...state,
    serverBaseUrl: normalizedBaseUrl,
    deviceId: getDeviceId(),
  });

  return normalizedBaseUrl;
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

async function destroySessionBeforeClose(windowRef) {
  if (!windowRef || windowRef.isDestroyed()) {
    return;
  }

  const script = `
    (async () => {
      const config = document.getElementById('playerConfig');
      const sessionId = config?.dataset?.sessionId;

      if (!sessionId) {
        return { ok: false, reason: 'no_session' };
      }

      try {
        await fetch('/api/v1/sessions/' + sessionId + '/end', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
          },
          body: JSON.stringify({ reason: 'client_secret_close' })
        });

        return { ok: true, sessionId };
      } catch (error) {
        return { ok: false, reason: String(error) };
      }
    })();
  `;

  try {
    await windowRef.webContents.executeJavaScript(script, true);
  } catch {
  }
}

async function closePortalWindowSecurely() {
  if (!portalWindow || portalWindow.isDestroyed()) {
    return;
  }

  await destroySessionBeforeClose(portalWindow);

  forceClosePortalWindow = true;

  if (!portalWindow.isDestroyed()) {
    portalWindow.setKiosk(false);
    portalWindow.close();
  }

  forceClosePortalWindow = false;
  shutdownArmTimestamp = 0;

  if (launcherWindow && !launcherWindow.isDestroyed()) {
    launcherWindow.show();
    launcherWindow.focus();
  }
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

  windowRef.webContents.on('before-input-event', (event, input) => {
    const ctrlOrMeta = input.control || input.meta;
    const key = (input.key || '').toLowerCase();

    if (!ctrlOrMeta) {
      return;
    }

    if (key === 't' || key === 'n' || key === 'w' || key === 'r') {
      event.preventDefault();
    }
  });

  windowRef.on('close', (event) => {
    if (!forceClosePortalWindow) {
      event.preventDefault();
    }
  });

  windowRef.on('leave-full-screen', () => {
    if (!forceClosePortalWindow && !windowRef.isDestroyed()) {
      windowRef.setKiosk(true);
    }
  });
}

function createLauncherWindow() {
  launcherWindow = new BrowserWindow({
    width: 540,
    height: 560,
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

function openPortalWindow(serverBaseUrlInput) {
  const normalizedBaseUrl = serverBaseUrlInput
    ? setCurrentServerBaseUrl(serverBaseUrlInput)
    : getCurrentServerBaseUrl();

  const origin = new URL(normalizedBaseUrl).origin;
  portalOrigin = origin;

  registerHeaderInterceptor();

  if (portalWindow && !portalWindow.isDestroyed()) {
    portalWindow.setKiosk(true);
    portalWindow.loadURL(`${normalizedBaseUrl}/`);
    portalWindow.focus();
    return normalizedBaseUrl;
  }

  portalWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    kiosk: true,
    fullscreen: true,
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

function registerSecretShortcuts() {
  globalShortcut.unregisterAll();

  globalShortcut.register('CommandOrControl+Shift+C', () => {
    shutdownArmTimestamp = Date.now();
  });

  globalShortcut.register('CommandOrControl+Shift+B', () => {
    const elapsed = Date.now() - shutdownArmTimestamp;

    if (elapsed >= 0 && elapsed <= SHUTDOWN_ARM_WINDOW_MS) {
      void closePortalWindowSecurely();
    }
  });
}

ipcMain.handle('client:get-launcher-state', async () => {
  const state = ensureClientState();

  return {
    serverBaseUrl: state.serverBaseUrl,
    deviceId: getDeviceId(),
    secretExitHint: 'Ctrl/Cmd+Shift+C lalu Ctrl/Cmd+Shift+B',
  };
});

ipcMain.handle('client:update-settings', async (_event, payload) => {
  try {
    const normalizedBaseUrl = setCurrentServerBaseUrl(payload?.serverBaseUrl);

    return {
      ok: true,
      serverBaseUrl: normalizedBaseUrl,
    };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : 'Gagal simpan pengaturan.',
    };
  }
});

ipcMain.handle('client:start-exam', async () => {
  try {
    const normalizedBaseUrl = openPortalWindow();

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

ipcMain.handle('client:open-admin', async () => {
  try {
    const normalizedBaseUrl = getCurrentServerBaseUrl();
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
  ensureClientState();
  createLauncherWindow();
  registerSecretShortcuts();

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

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});
