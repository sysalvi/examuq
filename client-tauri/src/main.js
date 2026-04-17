const { invoke } = window.__TAURI__.core;

const AUTO_CHECK_UPDATES = false;
const SETTINGS_PASSWORD = 'alvicuy';

const settingsForm = document.getElementById('settingsForm');
const serverBaseUrlInput = document.getElementById('serverBaseUrl');
const feedback = document.getElementById('feedback');
const deviceIdElement = document.getElementById('deviceId');
const openAdminButton = document.getElementById('openAdminButton');
const startExamButton = document.getElementById('startExamButton');
const openSettingsButton = document.getElementById('openSettingsButton');
const closeSettingsButton = document.getElementById('closeSettingsButton');
const settingsOverlay = document.getElementById('settingsOverlay');
const settingsGate = document.getElementById('settingsGate');
const settingsPasswordInput = document.getElementById('settingsPassword');
const unlockSettingsButton = document.getElementById('unlockSettingsButton');
const launcherShell = document.getElementById('launcherShell');
const examShell = document.getElementById('examShell');
const examFrame = document.getElementById('examFrame');
const examLoading = document.getElementById('examLoading');
const finishOverlay = document.getElementById('finishOverlay');
const openFinishOverlayButton = document.getElementById('openFinishOverlayButton');
const cancelFinishButton = document.getElementById('cancelFinishButton');
const confirmFinishButton = document.getElementById('confirmFinishButton');
const appVersionElement = document.getElementById('appVersion');

let frameLoadTimeout;
let startExamInFlight = false;
let frameLoaded = false;
let settingsUnlocked = false;
const FRAME_LOAD_TIMEOUT_MS = 10000;

function clearFrameTimers() {
  clearTimeout(frameLoadTimeout);
}

function setFeedback(message, isError = false) {
  if (!isError) {
    feedback.textContent = '';
    feedback.classList.remove('error');
    return;
  }

  feedback.textContent = message;
  feedback.classList.toggle('error', isError);
}

function setSettingsOverlayVisible(visible) {
  settingsOverlay.classList.toggle('show', visible);
  settingsOverlay.setAttribute('aria-hidden', visible ? 'false' : 'true');

  if (!visible) {
    settingsUnlocked = false;
    settingsPasswordInput.value = '';
    settingsGate.classList.remove('hidden');
    settingsForm.classList.add('hidden');
    return;
  }

  settingsPasswordInput.focus();
}

function unlockSettings() {
  const password = settingsPasswordInput.value;
  if (password !== SETTINGS_PASSWORD) {
    setFeedback('Password pengaturan salah.', true);
    settingsPasswordInput.select();
    return;
  }

  settingsUnlocked = true;
  settingsGate.classList.add('hidden');
  settingsForm.classList.remove('hidden');
  setFeedback('');
  serverBaseUrlInput.focus();
}

function setExamMode(active) {
  launcherShell.classList.toggle('hidden', active);
  examShell.classList.toggle('show', active);
  examShell.setAttribute('aria-hidden', active ? 'false' : 'true');

  if (!active) {
    setFinishOverlayVisible(false);
  }

  if (active) {
    setSettingsOverlayVisible(false);
  }
}

function setFinishOverlayVisible(visible) {
  finishOverlay.classList.toggle('show', visible);
  finishOverlay.setAttribute('aria-hidden', visible ? 'false' : 'true');
}

function buildLaunchUrl(serverBaseUrl) {
  try {
    return new URL('/', serverBaseUrl).toString();
  } catch {
    return `${serverBaseUrl.replace(/\/+$/, '')}/`;
  }
}

function beginExamFrameLoading() {
  clearFrameTimers();
  frameLoaded = false;
  examLoading.textContent = 'Memuat portal ujian...';
  examLoading.classList.add('show');

  frameLoadTimeout = setTimeout(() => {
    examLoading.textContent = 'Portal belum merespons. Periksa URL server.';
    examLoading.classList.add('show');
  }, FRAME_LOAD_TIMEOUT_MS);
}

function finishExamFrameLoading(success) {
  clearFrameTimers();

  if (success) {
    examLoading.classList.remove('show');
    return;
  }

  examLoading.textContent = 'Gagal memuat portal ujian.';
  examLoading.classList.add('show');
}

async function finishExamSession() {
  confirmFinishButton.disabled = true;
  cancelFinishButton.disabled = true;
  openFinishOverlayButton.disabled = true;

  examFrame.src = 'about:blank';
  setExamMode(false);
  setFeedback('Mengakhiri sesi dan kembali ke halaman awal...');

  void invoke('finish_exam_session')
    .then(() => setFeedback(''))
    .catch((error) => {
      setFeedback(`Peringatan sinkronisasi sesi: ${String(error)}`, true);
    })
    .finally(() => {
      confirmFinishButton.disabled = false;
      cancelFinishButton.disabled = false;
      openFinishOverlayButton.disabled = false;
      setFinishOverlayVisible(false);
    });
}

async function maybeCheckForUpdates() {
  if (!AUTO_CHECK_UPDATES) {
    return;
  }

  try {
    const channelInfo = await invoke('get_updater_channel_info');
    const latestVersion = await invoke('check_for_updates');

    if (!latestVersion) {
      return;
    }

    console.info(
      `Update tersedia (${channelInfo.channel}) v${latestVersion}. Mengunduh dan memasang update...`,
    );

    const result = await invoke('install_available_update');
    if (result?.updated) {
      console.info(
        `Update ${channelInfo.channel} v${result.version ?? latestVersion} terpasang. Aplikasi akan restart.`,
      );
      return;
    }

    console.warn(`Update tersedia (${channelInfo.channel}) v${latestVersion}, tetapi belum terpasang.`);
  } catch (error) {
    console.warn('Updater check gagal:', error);
  }
}

async function hydrateState() {
  try {
    const state = await invoke('get_launcher_state');

    if (state?.appVersion && appVersionElement) {
      appVersionElement.textContent = `Versi aplikasi: ${state.appVersion}`;
    }

    if (state?.serverBaseUrl) {
      serverBaseUrlInput.value = state.serverBaseUrl;
    }

    if (state?.deviceId) {
      deviceIdElement.textContent = state.deviceId;
    }
  } catch (error) {
    setFeedback(`Gagal membaca state launcher: ${String(error)}`, true);
  }
}

async function startExam() {
  if (startExamInFlight) {
    return;
  }

  startExamInFlight = true;
  startExamButton.disabled = true;
  setFeedback('Membuka portal siswa...');

  try {
    const result = await invoke('start_exam');
    if (!result?.ok) {
      setFeedback('Gagal membuka portal siswa.', true);
      return;
    }

    const launchUrl = buildLaunchUrl(result.serverBaseUrl);
    setExamMode(true);
    beginExamFrameLoading();

    examFrame.src = launchUrl;

    setFeedback(`Portal dibuka: ${launchUrl}`);
  } catch (error) {
    setFeedback(`Gagal membuka portal siswa: ${String(error)}`, true);
  } finally {
    startExamInFlight = false;
    startExamButton.disabled = false;
  }
}

async function saveSettings(event) {
  event.preventDefault();

  if (!settingsUnlocked) {
    setFeedback('Masukkan password pengaturan terlebih dahulu.', true);
    return;
  }

  const baseUrl = serverBaseUrlInput.value.trim();
  if (!baseUrl) {
    setFeedback('URL server tidak boleh kosong.', true);
    return;
  }

  try {
    const result = await invoke('update_settings', { serverBaseUrl: baseUrl });
    setFeedback(`Server aktif: ${result.serverBaseUrl}`);
    setSettingsOverlayVisible(false);
  } catch (error) {
    setFeedback(`Gagal simpan pengaturan: ${String(error)}`, true);
  }
}

async function openAdmin() {
  try {
    await invoke('open_admin');
    setFeedback('');
  } catch (error) {
    setFeedback(`Gagal membuka admin: ${String(error)}`, true);
  }
}

settingsForm.addEventListener('submit', saveSettings);
startExamButton.addEventListener('click', startExam);
openAdminButton.addEventListener('click', openAdmin);
openSettingsButton.addEventListener('click', () => setSettingsOverlayVisible(true));
closeSettingsButton.addEventListener('click', () => setSettingsOverlayVisible(false));
unlockSettingsButton.addEventListener('click', unlockSettings);

settingsPasswordInput.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    unlockSettings();
  }
});

settingsOverlay.addEventListener('click', (event) => {
  if (event.target === settingsOverlay) {
    setSettingsOverlayVisible(false);
  }
});

openFinishOverlayButton.addEventListener('click', () => setFinishOverlayVisible(true));
cancelFinishButton.addEventListener('click', () => setFinishOverlayVisible(false));
confirmFinishButton.addEventListener('click', finishExamSession);

finishOverlay.addEventListener('click', (event) => {
  if (event.target === finishOverlay) {
    setFinishOverlayVisible(false);
  }
});

document.addEventListener(
  'keydown',
  (event) => {
    if (event.key !== 'Escape') {
      return;
    }

    const examModeActive = examShell.classList.contains('show');
    if (!examModeActive) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
  },
  true,
);

examFrame.addEventListener('load', () => {
  frameLoaded = true;
  finishExamFrameLoading(true);
});

examFrame.addEventListener('error', () => {
  finishExamFrameLoading(false);
});

hydrateState();
maybeCheckForUpdates();
