const settingsForm = document.getElementById('settingsForm');
const serverBaseUrlInput = document.getElementById('serverBaseUrl');
const feedback = document.getElementById('feedback');
const deviceIdElement = document.getElementById('deviceId');
const openAdminButton = document.getElementById('openAdminButton');
const startExamButton = document.getElementById('startExamButton');
const activeServerElement = document.getElementById('activeServer');
const openSettingsButton = document.getElementById('openSettingsButton');
const closeSettingsButton = document.getElementById('closeSettingsButton');
const settingsOverlay = document.getElementById('settingsOverlay');

function setFeedback(message, isError = false) {
  feedback.textContent = message;
  feedback.classList.toggle('error', isError);
}

async function hydrateState() {
  try {
    const state = await window.examuqClient.getLauncherState();

    if (state?.serverBaseUrl) {
      serverBaseUrlInput.value = state.serverBaseUrl;
      activeServerElement.textContent = state.serverBaseUrl;
    }

    if (state?.deviceId) {
      deviceIdElement.textContent = state.deviceId;
    }

  } catch {
    setFeedback('Failed to read launcher state.', true);
  }
}

function setSettingsOverlayVisible(visible) {
  settingsOverlay.classList.toggle('show', visible);
  settingsOverlay.setAttribute('aria-hidden', visible ? 'false' : 'true');
}

async function startExam() {
  setFeedback('Membuka portal siswa...');

  const result = await window.examuqClient.startExam();

  if (!result?.ok) {
    setFeedback(result?.error || 'Failed to open student portal.', true);
    return;
  }

  setFeedback(`Portal dibuka: ${result.serverBaseUrl}`);
}

async function saveSettings(event) {
  event.preventDefault();

  const baseUrl = serverBaseUrlInput.value.trim();
  if (!baseUrl) {
    setFeedback('URL server tidak boleh kosong.', true);
    return;
  }

  const result = await window.examuqClient.updateSettings(baseUrl);

  if (!result?.ok) {
    setFeedback(result?.error || 'Gagal simpan pengaturan.', true);
    return;
  }

  activeServerElement.textContent = result.serverBaseUrl;
  setFeedback(`Server aktif: ${result.serverBaseUrl}`);
  setSettingsOverlayVisible(false);
}

async function openAdmin() {
  const result = await window.examuqClient.openAdmin();

  if (!result?.ok) {
    setFeedback(result?.error || 'Failed to open admin page.', true);
    return;
  }

  setFeedback('Admin page opened in default browser.');
}

settingsForm.addEventListener('submit', saveSettings);
startExamButton.addEventListener('click', startExam);
openAdminButton.addEventListener('click', openAdmin);
openSettingsButton.addEventListener('click', () => setSettingsOverlayVisible(true));
closeSettingsButton.addEventListener('click', () => setSettingsOverlayVisible(false));
settingsOverlay.addEventListener('click', (event) => {
  if (event.target === settingsOverlay) {
    setSettingsOverlayVisible(false);
  }
});

hydrateState();
