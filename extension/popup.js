function sendMessage(message) {
  return new Promise((resolve) => {
    chrome.runtime.sendMessage(message, (response) => {
      resolve(response);
    });
  });
}

function queryCurrentTabId() {
  return new Promise((resolve) => {
    chrome.tabs.query({ active: true, lastFocusedWindow: true }, (tabs) => {
      if (tabs.length === 0 || typeof tabs[0].id !== 'number') {
        resolve(null);
        return;
      }

      resolve(tabs[0].id);
    });
  });
}

function renderState(state) {
  const statusText = document.getElementById('statusText');
  const toggleButton = document.getElementById('toggleButton');

  if (!statusText || !toggleButton) {
    return;
  }

  const isEnabled = Boolean(state?.enabled);
  statusText.textContent = `Status: ${isEnabled ? 'ON' : 'OFF'}`;
  toggleButton.textContent = isEnabled ? 'Turn OFF' : 'Turn ON';
  toggleButton.classList.toggle('is-on', isEnabled);
}

async function loadState() {
  const response = await sendMessage({ type: 'GET_STATE' });

  if (!response?.ok) {
    renderState({ enabled: false });
    return;
  }

  renderState(response.state);
}

async function handleToggle() {
  const current = await sendMessage({ type: 'GET_STATE' });
  const currentEnabled = Boolean(current?.state?.enabled);
  const nextEnabled = !currentEnabled;

  const allowTabId = nextEnabled ? await queryCurrentTabId() : null;

  const response = await sendMessage({
    type: 'SET_ENABLED',
    enabled: nextEnabled,
    allowTabId,
  });

  if (!response?.ok) {
    return;
  }

  renderState(response.state);
}

document.addEventListener('DOMContentLoaded', () => {
  const toggleButton = document.getElementById('toggleButton');
  if (toggleButton) {
    toggleButton.addEventListener('click', handleToggle);
  }

  loadState();
});
