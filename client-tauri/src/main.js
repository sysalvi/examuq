const { invoke } = window.__TAURI__.core;

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

function setSettingsOverlayVisible(visible) {
  settingsOverlay.classList.toggle('show', visible);
  settingsOverlay.setAttribute('aria-hidden', visible ? 'false' : 'true');
}

async function hydrateState() {
  try {
    const state = await invoke('get_launcher_state');

    if (state?.serverBaseUrl) {
      serverBaseUrlInput.value = state.serverBaseUrl;
      activeServerElement.textContent = state.serverBaseUrl;
    }

    if (state?.deviceId) {
      deviceIdElement.textContent = state.deviceId;
    }
  } catch (error) {
    setFeedback(`Gagal membaca state launcher: ${String(error)}`, true);
  }
}

async function startExam() {
  setFeedback('Membuka portal siswa...');

  try {
    const result = await invoke('start_exam');
    if (!result?.ok) {
      setFeedback('Gagal membuka portal siswa.', true);
      return;
    }

    setFeedback(`Portal dibuka: ${result.serverBaseUrl}`);
  } catch (error) {
    setFeedback(`Gagal membuka portal siswa: ${String(error)}`, true);
  }
}

async function saveSettings(event) {
  event.preventDefault();

  const baseUrl = serverBaseUrlInput.value.trim();
  if (!baseUrl) {
    setFeedback('URL server tidak boleh kosong.', true);
    return;
  }

  try {
    const result = await invoke('update_settings', { serverBaseUrl: baseUrl });
    activeServerElement.textContent = result.serverBaseUrl;
    setFeedback(`Server aktif: ${result.serverBaseUrl}`);
    setSettingsOverlayVisible(false);
  } catch (error) {
    setFeedback(`Gagal simpan pengaturan: ${String(error)}`, true);
  }
}

async function openAdmin() {
  try {
    await invoke('open_admin');
    setFeedback('Admin dibuka di browser default.');
  } catch (error) {
    setFeedback(`Gagal membuka admin: ${String(error)}`, true);
  }
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
