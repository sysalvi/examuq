const form = document.getElementById('launcherForm');
const serverBaseUrlInput = document.getElementById('serverBaseUrl');
const feedback = document.getElementById('feedback');
const deviceIdElement = document.getElementById('deviceId');
const openAdminButton = document.getElementById('openAdminButton');

function setFeedback(message, isError = false) {
  feedback.textContent = message;
  feedback.classList.toggle('error', isError);
}

async function hydrateState() {
  try {
    const state = await window.examuqClient.getLauncherState();

    if (state?.lastServerBaseUrl) {
      serverBaseUrlInput.value = state.lastServerBaseUrl;
    }

    if (state?.deviceId) {
      deviceIdElement.textContent = state.deviceId;
    }
  } catch {
    setFeedback('Failed to read launcher state.', true);
  }
}

async function openPortal(event) {
  event.preventDefault();

  const baseUrl = serverBaseUrlInput.value.trim();
  if (!baseUrl) {
    setFeedback('Server base URL wajib diisi.', true);
    return;
  }

  setFeedback('Opening student portal...');

  const result = await window.examuqClient.openPortal(baseUrl);

  if (!result?.ok) {
    setFeedback(result?.error || 'Failed to open student portal.', true);
    return;
  }

  setFeedback(`Portal opened: ${result.serverBaseUrl}`);
}

async function openAdmin() {
  const baseUrl = serverBaseUrlInput.value.trim();
  if (!baseUrl) {
    setFeedback('Isi server URL dulu sebelum buka admin.', true);
    return;
  }

  const result = await window.examuqClient.openAdmin(baseUrl);
  if (!result?.ok) {
    setFeedback(result?.error || 'Failed to open admin page.', true);
    return;
  }

  setFeedback('Admin page opened in default browser.');
}

form.addEventListener('submit', openPortal);
openAdminButton.addEventListener('click', openAdmin);

hydrateState();
