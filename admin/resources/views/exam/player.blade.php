<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ExamUQ Player - {{ $exam->title }}</title>
    <style>
        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            background: #0f172a;
            color: #e2e8f0;
        }

        .layout {
            min-height: 100vh;
            display: grid;
            grid-template-rows: auto 1fr;
        }

        .topbar {
            display: flex;
            flex-wrap: wrap;
            gap: 12px;
            align-items: center;
            justify-content: space-between;
            padding: 12px 16px;
            background: #111827;
            border-bottom: 1px solid #1f2937;
        }

        .meta {
            display: flex;
            flex-wrap: wrap;
            gap: 8px 16px;
            font-size: 13px;
            color: #cbd5e1;
        }

        .timer {
            font-size: 14px;
            font-weight: 700;
            color: #f8fafc;
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 6px 10px;
        }

        .timer.warning {
            color: #fecaca;
            border-color: #dc2626;
            background: #450a0a;
        }

        .actions {
            display: flex;
            gap: 8px;
            align-items: center;
        }

        .btn {
            border: 1px solid #334155;
            background: #1e293b;
            color: #f8fafc;
            border-radius: 8px;
            padding: 8px 12px;
            font-size: 13px;
            cursor: pointer;
        }

        .btn.danger {
            border-color: #dc2626;
            background: #7f1d1d;
        }

        .frame-wrap {
            height: calc(100vh - 64px);
        }

        iframe {
            width: 100%;
            height: 100%;
            border: 0;
            background: #ffffff;
        }

        .overlay {
            position: fixed;
            inset: 0;
            display: none;
            place-items: center;
            background: rgba(2, 6, 23, 0.88);
            padding: 24px;
            z-index: 50;
        }

        .overlay.show {
            display: grid;
        }

        .overlay.fullscreen-guard {
            z-index: 60;
            background: rgba(2, 6, 23, 0.92);
        }

        .overlay.fullscreen-guard .overlay-card {
            border-color: #dc2626;
        }

        .overlay.fullscreen-guard .overlay-card p {
            margin-bottom: 12px;
        }

        .overlay-card {
            width: 100%;
            max-width: 540px;
            background: #111827;
            border: 1px solid #334155;
            border-radius: 12px;
            padding: 20px;
        }

        .overlay-card h2 {
            margin: 0 0 8px;
            font-size: 22px;
        }

        .overlay-card p {
            margin: 0;
            color: #cbd5e1;
            line-height: 1.5;
        }

        .overlay-card .btn {
            margin-top: 8px;
        }

        .frame-wrap.blocked iframe {
            pointer-events: none;
            filter: blur(2px) brightness(0.5);
        }
    </style>
</head>
<body>
<div class="layout">
    <header class="topbar">
        <div class="meta">
            <span><strong>Ujian:</strong> {{ $exam->title }}</span>
            <span><strong>Nama:</strong> {{ $examSession->display_name }}</span>
            <span><strong>Kelas/Ruang:</strong> {{ $examSession->class_room }}</span>
        </div>

        <div class="actions">
            <div id="timer" class="timer">Waktu: --:--:--</div>
            <button id="endButton" class="btn danger" type="button">Akhiri Sesi</button>
        </div>
    </header>

    <main class="frame-wrap" role="main" aria-label="Exam player">
        <iframe id="examFrame" src="{{ $exam->exam_url }}" allowfullscreen></iframe>
    </main>
</div>

<section id="overlay" class="overlay" aria-live="polite">
    <div class="overlay-card">
        <h2>Sesi Ujian Ditutup</h2>
        <p id="overlayMessage">Waktu ujian berakhir. Silakan hubungi pengawas jika perlu bantuan.</p>
    </div>
</section>

<section id="fullscreenGuard" class="overlay fullscreen-guard" aria-live="polite">
    <div class="overlay-card">
        <h2>Mode Fullscreen Wajib</h2>
        <p id="fullscreenMessage">Masuk fullscreen untuk melanjutkan ujian.</p>
        <button id="fullscreenButton" class="btn" type="button">Masuk Fullscreen</button>
    </div>
</section>

<div
    id="playerConfig"
    data-session-id="{{ $examSession->id }}"
    data-deadline-at="{{ $deadlineAtIso ?? '' }}"
    data-server-now="{{ $serverNowIso }}"
></div>

<script>
    const playerConfigElement = document.getElementById('playerConfig');
    const sessionId = Number(playerConfigElement?.dataset.sessionId || 0);
    const deadlineAtIso = playerConfigElement?.dataset.deadlineAt || null;
    const serverNowIso = playerConfigElement?.dataset.serverNow || new Date().toISOString();

    const timerElement = document.getElementById('timer');
    const endButton = document.getElementById('endButton');
    const overlayElement = document.getElementById('overlay');
    const overlayMessageElement = document.getElementById('overlayMessage');
    const fullscreenGuardElement = document.getElementById('fullscreenGuard');
    const fullscreenMessageElement = document.getElementById('fullscreenMessage');
    const fullscreenButton = document.getElementById('fullscreenButton');
    const frameWrap = document.querySelector('.frame-wrap');
    const examFrame = document.getElementById('examFrame');

    let ended = false;
    let fullscreenArmed = false;
    let fullscreenArmTimerId = null;
    let fullscreenChangeTimerId = null;
    let heartbeatTimerId = null;
    let heartbeatInFlight = false;

    const FULLSCREEN_GRACE_MS = 6500;
    const HEARTBEAT_BASE_MS = 75000;
    const HEARTBEAT_JITTER_MS = 20000;

    function showOverlay(message) {
        overlayMessageElement.textContent = message;
        overlayElement.classList.add('show');
    }

    function setFrameBlocked(blocked) {
        frameWrap.classList.toggle('blocked', blocked);
        fullscreenGuardElement.classList.toggle('show', blocked && !ended);
    }

    async function requestFullscreen() {
        if (document.fullscreenElement) {
            return true;
        }

        try {
            await document.documentElement.requestFullscreen();
            return true;
        } catch (_error) {
            console.warn('Fullscreen request failed on player:', _error);
            return false;
        }
    }

    async function endSession(reason) {
        if (ended) {
            return;
        }

        ended = true;

        if (heartbeatTimerId) {
            window.clearTimeout(heartbeatTimerId);
            heartbeatTimerId = null;
        }

        try {
            await fetch(`/api/v1/sessions/${sessionId}/end`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                },
                body: JSON.stringify({ reason }),
            });
        } catch (_error) {
            console.warn('Failed to end session gracefully:', _error);
        }

        examFrame.src = 'about:blank';
        setFrameBlocked(false);

        if (reason === 'timeout') {
            showOverlay('Waktu ujian habis. Sesi Anda telah ditutup otomatis.');
            return;
        }

        showOverlay('Sesi ujian ditutup. Anda dapat menutup halaman ini.');
    }

    async function handleFullscreenGuard(forceMessage = null) {
        if (ended || !fullscreenArmed) {
            return;
        }

        const inFullscreen = Boolean(document.fullscreenElement);
        if (inFullscreen) {
            setFrameBlocked(false);
            return;
        }

        if (forceMessage) {
            fullscreenMessageElement.textContent = forceMessage;
        } else {
            fullscreenMessageElement.textContent = 'Masuk fullscreen untuk melanjutkan ujian.';
        }

        setFrameBlocked(true);
    }

    function formatDuration(totalSeconds) {
        const safeSeconds = Math.max(0, totalSeconds);
        const hours = Math.floor(safeSeconds / 3600);
        const minutes = Math.floor((safeSeconds % 3600) / 60);
        const seconds = safeSeconds % 60;

        return [hours, minutes, seconds]
            .map((value) => String(value).padStart(2, '0'))
            .join(':');
    }

    function startTimer() {
        if (!deadlineAtIso) {
            timerElement.textContent = 'Waktu: tanpa batas';
            return;
        }

        const deadlineTime = new Date(deadlineAtIso).getTime();
        const serverNowTime = new Date(serverNowIso).getTime();
        const localNowTime = Date.now();
        const clockOffset = serverNowTime - localNowTime;

        const tick = () => {
            if (ended) {
                return;
            }

            const now = Date.now() + clockOffset;
            const remainingMs = deadlineTime - now;
            const remainingSeconds = Math.floor(remainingMs / 1000);

            timerElement.textContent = `Waktu: ${formatDuration(remainingSeconds)}`;
            timerElement.classList.toggle('warning', remainingSeconds <= 300);

            if (remainingMs <= 0) {
                endSession('timeout');
            }
        };

        tick();
        window.setInterval(tick, 1000);
    }

    function startHeartbeat() {
        const nextHeartbeatDelay = () => {
            const visibilityMultiplier = document.hidden ? 2 : 1;
            const deterministicSpread = ((sessionId % 17) * 700) % 6000;
            const randomJitter = Math.floor(Math.random() * HEARTBEAT_JITTER_MS);

            return (HEARTBEAT_BASE_MS * visibilityMultiplier) + deterministicSpread + randomJitter;
        };

        const scheduleHeartbeat = (delayMs) => {
            if (ended) {
                return;
            }

            heartbeatTimerId = window.setTimeout(runHeartbeat, delayMs);
        };

        const runHeartbeat = async () => {
            if (ended) {
                return;
            }

            if (heartbeatInFlight) {
                scheduleHeartbeat(nextHeartbeatDelay());
                return;
            }

            heartbeatInFlight = true;

            try {
                await fetch(`/api/v1/sessions/${sessionId}/heartbeat`, {
                    method: 'POST',
                    headers: {
                        'Accept': 'application/json',
                    },
                });
            } catch (_error) {
                console.warn('Heartbeat request failed:', _error);
            } finally {
                heartbeatInFlight = false;
                scheduleHeartbeat(nextHeartbeatDelay());
            }
        };

        const initialSpreadDelay = 1500 + Math.floor(Math.random() * 3000);
        scheduleHeartbeat(initialSpreadDelay);
    }

    function startFullscreenEnforcement() {
        const onFullscreenChange = () => {
            if (fullscreenChangeTimerId) {
                window.clearTimeout(fullscreenChangeTimerId);
            }

            fullscreenChangeTimerId = window.setTimeout(async () => {
                if (ended) {
                    return;
                }

                const exited = !document.fullscreenElement;
                if (exited) {
                    if (!fullscreenArmed) {
                        return;
                    }

                    handleFullscreenGuard('Fullscreen terdeteksi keluar. Masuk fullscreen lagi untuk lanjut ujian.');
                    return;
                }

                setFrameBlocked(false);
            }, 280);
        };

        const armFullscreenEnforcement = () => {
            if (ended) {
                return;
            }

            fullscreenArmed = true;

            if (!document.fullscreenElement) {
                handleFullscreenGuard('Masuk fullscreen untuk melanjutkan ujian.');
            }
        };

        document.addEventListener('fullscreenchange', onFullscreenChange);

        fullscreenButton.addEventListener('click', async () => {
            const ok = await requestFullscreen();
            if (!ok && fullscreenArmed) {
                handleFullscreenGuard('Gagal masuk fullscreen. Pastikan izin browser aktif lalu coba lagi.');
            }
        });

        requestFullscreen().catch((error) => {
            console.warn('Initial fullscreen request failed:', error);
        });

        if (fullscreenArmTimerId) {
            window.clearTimeout(fullscreenArmTimerId);
        }

        fullscreenArmTimerId = window.setTimeout(armFullscreenEnforcement, FULLSCREEN_GRACE_MS);
    }

    endButton.addEventListener('click', () => {
        endSession('manual_end_by_student');
    });

    startTimer();
    startHeartbeat();
    startFullscreenEnforcement();
</script>
</body>
</html>
