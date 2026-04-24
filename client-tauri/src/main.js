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
const examDisplayName = document.getElementById('examDisplayName');
const examTimer = document.getElementById('examTimer');
const examViewport = document.getElementById('examViewport');
const examLaunchForm = document.getElementById('examLaunchForm');
const examViewportPlaceholder = document.getElementById('examViewportPlaceholder');
const participantNameInput = document.getElementById('participantName');
const participantClassInput = document.getElementById('participantClass');
const participantTokenInput = document.getElementById('participantToken');
const submitParticipantButton = document.getElementById('submitParticipantButton');
const examLoading = document.getElementById('examLoading');
const finishOverlay = document.getElementById('finishOverlay');
const openFinishOverlayButton = document.getElementById('openFinishOverlayButton');
const cancelFinishButton = document.getElementById('cancelFinishButton');
const confirmFinishButton = document.getElementById('confirmFinishButton');
const appVersionElement = document.getElementById('appVersion');

let startExamInFlight = false;
let settingsUnlocked = false;
let currentDeviceId = '';
let examModeActive = false;
let examState = {
  sessionId: '',
  displayName: 'Peserta',
  deadlineAt: '',
  returnUrl: '',
  apiBase: '',
};
let examPollingTimer;
let examClockTimer;
let launchExamInFlight = false;

function isBlockedFunctionKey(key) {
  return /^F([1-9]|1[0-2])$/.test(key);
}

function isBlockedRefreshShortcut(event) {
  const key = String(event.key || '').toLowerCase();
  return (event.ctrlKey || event.metaKey) && key === 'r';
}

function shouldBlockHotkey(event) {
  return isBlockedFunctionKey(event.key) || isBlockedRefreshShortcut(event);
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
  examModeActive = active;
  launcherShell.classList.toggle('hidden', active);
  examShell.classList.toggle('show', active);
  examShell.setAttribute('aria-hidden', active ? 'false' : 'true');
  examViewport?.setAttribute('aria-hidden', active ? 'false' : 'true');

  if (!active) {
    setFinishOverlayVisible(false);
    stopExamStateSync();
    renderExamState();
    setExamLaunchFormVisible(false);
    clearParticipantForm();
  }

  if (active) {
    setSettingsOverlayVisible(false);
    startExamStateSync();
  }
}

function setExamLaunchFormVisible(visible) {
  if (!examLaunchForm) {
    return;
  }

  examLaunchForm.classList.toggle('show', visible);
  examLaunchForm.classList.toggle('hidden', !visible);
  examLaunchForm.setAttribute('aria-hidden', visible ? 'false' : 'true');

  if (examViewportPlaceholder) {
    examViewportPlaceholder.classList.toggle('hidden', visible);
  }
}

function clearParticipantForm() {
  if (participantTokenInput) {
    participantTokenInput.value = '';
  }
}

function setFinishOverlayVisible(visible) {
  finishOverlay.classList.toggle('show', visible);
  finishOverlay.setAttribute('aria-hidden', visible ? 'false' : 'true');
}

function returnToLauncherUi(message = '') {
  examState = {
    sessionId: '',
    displayName: 'Peserta',
    deadlineAt: '',
    returnUrl: '',
    apiBase: '',
  };
  setExamMode(false);
  setFinishOverlayVisible(false);
  setExamLoadingState(false, 'Menunggu mode ujian aktif...');
  setFeedback(message, false);
}

async function finishExamSession() {
  confirmFinishButton.disabled = true;
  cancelFinishButton.disabled = true;
  openFinishOverlayButton.disabled = true;
  setExamLoadingState(true, 'Mengakhiri sesi ujian...');
  returnToLauncherUi('Mengakhiri sesi dan kembali ke halaman awal...');

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

async function submitParticipantData() {
  if (launchExamInFlight) {
    return;
  }

  const displayName = participantNameInput?.value?.trim() || '';
  const classRoom = participantClassInput?.value?.trim() || '';
  const tokenGlobal = participantTokenInput?.value?.trim() || '';

  if (!displayName) {
    setFeedback('Nama peserta wajib diisi.', true);
    participantNameInput?.focus();
    return;
  }

  if (!classRoom) {
    setFeedback('Kelas wajib diisi.', true);
    participantClassInput?.focus();
    return;
  }

  if (!tokenGlobal) {
    setFeedback('Token ujian wajib diisi.', true);
    participantTokenInput?.focus();
    return;
  }

  launchExamInFlight = true;
  if (submitParticipantButton) {
    submitParticipantButton.disabled = true;
  }

  setExamLoadingState(true, 'Memvalidasi data peserta dan token...');

  try {
    const result = await invoke('launch_exam_from_client', {
      payload: {
        displayName,
        classRoom,
        tokenGlobal,
      },
    });

    if (!result?.ok) {
      setFeedback('Gagal membuka ujian.', true);
      setExamLoadingState(false, 'Isi data peserta untuk memulai ujian.');
      return;
    }

    setExamLaunchFormVisible(false);
    setExamLoadingState(true, 'Memuat portal ujian...');
    setFeedback('Data peserta diterima. Mengarahkan ke ujian...');
  } catch (error) {
    setFeedback(`Gagal validasi peserta/token: ${String(error)}`, true);
    setExamLoadingState(false, 'Isi data peserta untuk memulai ujian.');
  } finally {
    launchExamInFlight = false;
    if (submitParticipantButton) {
      submitParticipantButton.disabled = false;
    }
  }
}

function formatDuration(totalSeconds) {
  const safe = Math.max(0, totalSeconds);
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  const seconds = safe % 60;
  return [hours, minutes, seconds].map((value) => String(value).padStart(2, '0')).join(':');
}

function renderExamState() {
  examDisplayName.textContent = examState.displayName?.trim() || 'Peserta';

  if (!examModeActive) {
    examTimer.textContent = '--:--:--';
    return;
  }

  if (!examState.deadlineAt) {
    examTimer.textContent = 'Menunggu data';
    return;
  }

  const deadline = new Date(examState.deadlineAt);
  if (Number.isNaN(deadline.getTime())) {
    examTimer.textContent = 'Format waktu invalid';
    return;
  }

  const diffInSeconds = Math.max(0, Math.floor((deadline.getTime() - Date.now()) / 1000));
  examTimer.textContent = formatDuration(diffInSeconds);
}

function setExamLoadingState(active, message = '') {
  examLoading.textContent = message || (active ? 'Memuat portal ujian...' : 'Portal ujian siap.');
  examLoading.dataset.state = active ? 'loading' : 'ready';
}

function applyExamState(nextState = {}) {
  examState = {
    ...examState,
    ...nextState,
  };
  renderExamState();
}

async function syncExamStateFromNative() {
  if (!examModeActive) {
    return;
  }

  try {
    const state = await invoke('get_exam_overlay_state');
    if (state) {
      applyExamState(state);
    }
  } catch (error) {
    console.warn('Gagal sinkronisasi state ujian:', error);
  }
}

function stopExamStateSync() {
  clearInterval(examPollingTimer);
  clearInterval(examClockTimer);
  examPollingTimer = undefined;
  examClockTimer = undefined;
}

function startExamStateSync() {
  stopExamStateSync();
  renderExamState();
  void syncExamStateFromNative();
  examPollingTimer = window.setInterval(() => {
    void syncExamStateFromNative();
  }, 5000);
  examClockTimer = window.setInterval(() => {
    renderExamState();
  }, 1000);
}

window.__EXAMUQ_ENTER_EXAM_MODE__ = () => {
  setExamMode(true);
};

window.__EXAMUQ_SET_EXAM_LOADING__ = (active = false, message = '') => {
  setExamLoadingState(Boolean(active), String(message || ''));
};

window.__EXAMUQ_SYNC_EXAM_STATE__ = (state = {}) => {
  applyExamState(state);
};

window.__EXAMUQ_RETURN_TO_LAUNCHER__ = (message = 'Mode ujian dihentikan dan kembali ke halaman awal.') => {
  returnToLauncherUi(message);
};

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
      currentDeviceId = state.deviceId;
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
  setFeedback('Masuk mode ujian...');
  setExamMode(true);
  setExamLaunchFormVisible(true);
  setExamLoadingState(false, 'Isi data peserta untuk memulai ujian.');

  try {
    const result = await invoke('start_exam');
    if (!result?.ok) {
      returnToLauncherUi('Gagal masuk mode ujian.');
      setFeedback('Gagal masuk mode ujian.', true);
      return;
    }

    setExamLaunchFormVisible(true);
    setExamLoadingState(false, 'Isi data peserta untuk memulai ujian.');
    participantNameInput?.focus();
    setFeedback('Mode ujian aktif. Isi data peserta dan token.');
  } catch (error) {
    returnToLauncherUi(`Gagal membuka portal siswa: ${String(error)}`);
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

openFinishOverlayButton.addEventListener('click', finishExamSession);
cancelFinishButton.addEventListener('click', () => setFinishOverlayVisible(false));
confirmFinishButton.addEventListener('click', finishExamSession);

finishOverlay.addEventListener('click', (event) => {
  if (event.target === finishOverlay) {
    setFinishOverlayVisible(false);
  }
});

submitParticipantButton?.addEventListener('click', submitParticipantData);

participantTokenInput?.addEventListener('keydown', (event) => {
  if (event.key !== 'Enter') {
    return;
  }

  event.preventDefault();
  void submitParticipantData();
});

document.addEventListener(
  'keydown',
  (event) => {
    if (shouldBlockHotkey(event)) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }

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

hydrateState();
maybeCheckForUpdates();
