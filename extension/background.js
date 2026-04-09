const STATE_KEY = 'examuqGuardState';
const DEVICE_ID_KEY = 'examuqDeviceId';

const DEFAULT_STATE = {
  enabled: false,
  allowTabId: null,
};

const HEADER_RULE_IDS = [1001, 1002, 1003];

function storageGet(keys) {
  return new Promise((resolve) => {
    chrome.storage.local.get(keys, (result) => resolve(result));
  });
}

function storageSet(values) {
  return new Promise((resolve) => {
    chrome.storage.local.set(values, () => resolve());
  });
}

function dnrUpdateDynamicRules(payload) {
  return new Promise((resolve) => {
    chrome.declarativeNetRequest.updateDynamicRules(payload, () => resolve());
  });
}

function tabsRemove(tabId) {
  return new Promise((resolve) => {
    chrome.tabs.remove(tabId, () => resolve());
  });
}

function tabsUpdate(tabId, updateProps) {
  return new Promise((resolve) => {
    chrome.tabs.update(tabId, updateProps, () => resolve());
  });
}

function tabsQuery(queryInfo) {
  return new Promise((resolve) => {
    chrome.tabs.query(queryInfo, (tabs) => resolve(tabs));
  });
}

function randomId(length) {
  const alphabet = 'abcdefghijklmnopqrstuvwxyz0123456789';
  let out = '';

  for (let i = 0; i < length; i += 1) {
    out += alphabet[Math.floor(Math.random() * alphabet.length)];
  }

  return out;
}

async function ensureDeviceId() {
  const data = await storageGet([DEVICE_ID_KEY]);
  const existing = data[DEVICE_ID_KEY];

  if (typeof existing === 'string' && existing.length > 0) {
    return existing;
  }

  const newId = `exq-${Date.now()}-${randomId(8)}`;
  await storageSet({ [DEVICE_ID_KEY]: newId });

  return newId;
}

async function getState() {
  const data = await storageGet([STATE_KEY]);
  const raw = data[STATE_KEY];

  if (!raw || typeof raw !== 'object') {
    return { ...DEFAULT_STATE };
  }

  return {
    enabled: Boolean(raw.enabled),
    allowTabId: typeof raw.allowTabId === 'number' ? raw.allowTabId : null,
  };
}

async function saveState(nextState) {
  await storageSet({ [STATE_KEY]: nextState });
}

async function setBadge(enabled) {
  await chrome.action.setBadgeBackgroundColor({ color: enabled ? '#b91c1c' : '#16a34a' });
  await chrome.action.setBadgeText({ text: enabled ? 'ON' : '' });
  await chrome.action.setTitle({ title: enabled ? 'ExamUQ Guard: ON' : 'ExamUQ Guard: OFF' });
}

async function syncHeaderRules(enabled) {
  if (!enabled) {
    await dnrUpdateDynamicRules({ removeRuleIds: HEADER_RULE_IDS, addRules: [] });
    return;
  }

  const deviceId = await ensureDeviceId();

  await dnrUpdateDynamicRules({
    removeRuleIds: HEADER_RULE_IDS,
    addRules: [
      {
        id: 1001,
        priority: 1,
        action: {
          type: 'modifyHeaders',
          requestHeaders: [
            {
              header: 'X-ExamUQ-Client-Type',
              operation: 'set',
              value: 'chrome_extension',
            },
          ],
        },
        condition: {
          urlFilter: '|http',
          resourceTypes: ['main_frame', 'sub_frame', 'xmlhttprequest'],
        },
      },
      {
        id: 1002,
        priority: 1,
        action: {
          type: 'modifyHeaders',
          requestHeaders: [
            {
              header: 'X-ExamUQ-Source',
              operation: 'set',
              value: 'extension',
            },
          ],
        },
        condition: {
          urlFilter: '|http',
          resourceTypes: ['main_frame', 'sub_frame', 'xmlhttprequest'],
        },
      },
      {
        id: 1003,
        priority: 1,
        action: {
          type: 'modifyHeaders',
          requestHeaders: [
            {
              header: 'X-ExamUQ-Device-Id',
              operation: 'set',
              value: deviceId,
            },
          ],
        },
        condition: {
          urlFilter: '|http',
          resourceTypes: ['main_frame', 'sub_frame', 'xmlhttprequest'],
        },
      },
    ],
  });
}

async function applyState(state) {
  await setBadge(state.enabled);
  await syncHeaderRules(state.enabled);
}

async function setEnabled(enabled, allowTabId = null) {
  const nextState = {
    enabled,
    allowTabId: enabled ? allowTabId : null,
  };

  await saveState(nextState);
  await applyState(nextState);

  return nextState;
}

async function initState() {
  const state = await getState();
  await saveState(state);
  await applyState(state);
}

chrome.runtime.onInstalled.addListener(() => {
  initState();
});

chrome.runtime.onStartup.addListener(() => {
  initState();
});

chrome.tabs.onCreated.addListener(async (tab) => {
  const state = await getState();

  if (!state.enabled) {
    return;
  }

  if (state.allowTabId && tab.id === state.allowTabId) {
    return;
  }

  if (typeof tab.id === 'number') {
    await tabsRemove(tab.id);
  }

  if (state.allowTabId) {
    await tabsUpdate(state.allowTabId, { active: true });
  }
});

chrome.commands.onCommand.addListener(async (command) => {
  if (command !== 'silent-disable') {
    return;
  }

  await setEnabled(false, null);
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || typeof message !== 'object') {
    sendResponse({ ok: false, error: 'invalid_message' });
    return;
  }

  if (message.type === 'GET_STATE') {
    getState().then((state) => sendResponse({ ok: true, state }));
    return true;
  }

  if (message.type === 'SET_ENABLED') {
    (async () => {
      const enabled = Boolean(message.enabled);
      let allowTabId = typeof message.allowTabId === 'number' ? message.allowTabId : null;

      if (enabled && allowTabId === null) {
        const tabs = await tabsQuery({ active: true, lastFocusedWindow: true });
        if (tabs.length > 0 && typeof tabs[0].id === 'number') {
          allowTabId = tabs[0].id;
        }
      }

      const nextState = await setEnabled(enabled, allowTabId);
      sendResponse({ ok: true, state: nextState });
    })();

    return true;
  }

  sendResponse({ ok: false, error: 'unknown_message_type' });
});
