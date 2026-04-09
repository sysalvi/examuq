const STATE_KEY = 'examuqGuardState';

let isGuardEnabled = false;
let originalWindowOpen = window.open;
let lastAutoReloadAt = 0;

function updateGuardState(state) {
  const prevEnabled = isGuardEnabled;
  const enabled = Boolean(state?.enabled);
  isGuardEnabled = enabled;

  if (enabled) {
    window.open = () => null;
  } else {
    window.open = originalWindowOpen;
  }

  const now = Date.now();
  if (!prevEnabled && enabled && now - lastAutoReloadAt > 5000) {
    lastAutoReloadAt = now;
    window.location.reload();
  }
}

function loadState() {
  chrome.storage.local.get([STATE_KEY], (result) => {
    updateGuardState(result[STATE_KEY]);
  });
}

function isBlockedShortcut(event) {
  const ctrlOrMeta = event.ctrlKey || event.metaKey;
  const key = event.key.toLowerCase();

  if (ctrlOrMeta && key === 't') {
    return true;
  }

  if (ctrlOrMeta && key === 'n') {
    return true;
  }

  if (ctrlOrMeta && event.shiftKey && key === 't') {
    return true;
  }

  return false;
}

window.addEventListener(
  'keydown',
  (event) => {
    if (!isGuardEnabled) {
      return;
    }

    if (!isBlockedShortcut(event)) {
      return;
    }

    event.preventDefault();
    event.stopImmediatePropagation();
  },
  true,
);

document.addEventListener(
  'click',
  (event) => {
    if (!isGuardEnabled) {
      return;
    }

    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }

    const anchor = target.closest('a[target="_blank"]');
    if (!anchor) {
      return;
    }

    event.preventDefault();
    event.stopImmediatePropagation();
  },
  true,
);

chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== 'local' || !changes[STATE_KEY]) {
    return;
  }

  updateGuardState(changes[STATE_KEY].newValue);
});

loadState();
