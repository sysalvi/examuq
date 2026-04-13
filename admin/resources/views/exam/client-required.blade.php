<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Akses ExamUQ Diperlukan</title>
    <style>
        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            min-height: 100vh;
            font-family: "Nunito", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            color: #f8fafc;
            background:
                radial-gradient(95vw 95vw at 8% -20%, #f59e0b33 0%, transparent 58%),
                radial-gradient(85vw 85vw at 100% 0%, #22d3ee33 0%, transparent 60%),
                linear-gradient(140deg, #0f172a 0%, #111827 48%, #312e81 100%);
            display: grid;
            place-items: center;
            padding: 20px;
        }

        .card {
            width: 100%;
            max-width: 620px;
            border: 1px solid #334155;
            border-radius: 18px;
            padding: 24px;
            background: #0f172ae8;
            box-shadow: 0 18px 45px #02061766;
            animation: cardEntrance 380ms ease-out;
        }

        .tag {
            display: inline-block;
            border-radius: 999px;
            padding: 5px 10px;
            font-size: 12px;
            font-weight: 900;
            letter-spacing: 0.3px;
            color: #451a03;
            background: #fcd34d;
            margin-bottom: 8px;
        }

        h1 {
            margin: 0 0 8px;
            font-size: clamp(24px, 5vw, 33px);
            line-height: 1.12;
        }

        .title-row {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-bottom: 6px;
        }

        .title-icon {
            width: 38px;
            height: 38px;
            border-radius: 10px;
            border: 1px solid #f59e0b77;
            background: #451a03;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            color: #fcd34d;
            flex-shrink: 0;
            animation: iconFloat 2.4s ease-in-out infinite;
        }

        .title-icon svg {
            width: 20px;
            height: 20px;
        }

        p {
            margin: 0;
            line-height: 1.45;
            color: #cbd5e1;
            font-size: 14px;
        }

        .actions {
            margin-top: 18px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            flex-wrap: wrap;
        }

        .status {
            font-size: 13px;
            color: #bae6fd;
            border: 1px dashed #0ea5e9;
            border-radius: 999px;
            padding: 7px 12px;
            background: #082f49;
            animation: pulseStatus 1900ms ease-in-out infinite;
        }

        .btn {
            border: 1px solid #14b8a6;
            border-radius: 12px;
            padding: 10px 14px;
            font-size: 13px;
            font-weight: 900;
            background: linear-gradient(135deg, #0d9488 0%, #06b6d4 100%);
            color: #ffffff;
            cursor: pointer;
            transition: transform 120ms ease, filter 120ms ease;
        }

        .btn:hover {
            filter: brightness(1.05);
            transform: translateY(-1px);
        }

        .btn:active {
            transform: translateY(0);
            filter: brightness(0.98);
        }

        .tips {
            margin-top: 14px;
            border-top: 1px solid #334155;
            padding-top: 12px;
            font-size: 12px;
            color: #94a3b8;
        }

        @keyframes cardEntrance {
            from {
                opacity: 0;
                transform: translateY(10px) scale(0.98);
            }

            to {
                opacity: 1;
                transform: translateY(0) scale(1);
            }
        }

        @keyframes pulseStatus {
            0%,
            100% {
                box-shadow: 0 0 0 0 #0ea5e955;
            }

            50% {
                box-shadow: 0 0 0 6px #0ea5e911;
            }
        }

        @keyframes iconFloat {
            0%,
            100% {
                transform: translateY(0);
            }

            50% {
                transform: translateY(-2px);
            }
        }
    </style>
</head>
<body>
<main class="card" role="main" aria-labelledby="title">
    <span class="tag">AKSES TERBATAS</span>
    <div class="title-row">
        <span class="title-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 3l7 4v5c0 5-3.4 8.4-7 9-3.6-.6-7-4-7-9V7l7-4z"/>
                <path d="M12 9v4"/>
                <circle cx="12" cy="16.5" r="0.7" fill="currentColor"/>
            </svg>
        </span>
        <h1 id="title">Aktifkan ExamUQ Dulu</h1>
    </div>
    <p>Halaman ujian ini hanya terbuka lewat ExamUQ Client atau ExamUQ Extension. Kalau aplikasinya sudah aktif, sistem akan otomatis deteksi.</p>

    <div class="actions">
        <span id="liveStatus" class="status">Mencari Client/Extension aktif...</span>
        <button id="retryButton" class="btn" type="button">Cek Lagi Sekarang</button>
    </div>

    <p class="tips">Tip: pastikan toggle extension dalam posisi ON, lalu tunggu beberapa detik.</p>
</main>

<script>
    const liveStatusElement = document.getElementById('liveStatus');
    const retryButton = document.getElementById('retryButton');

    async function checkClientReady() {
        try {
            const response = await fetch('/', {
                method: 'GET',
                cache: 'no-store',
                credentials: 'include',
                headers: {
                    'Accept': 'text/html',
                },
            });

            if (response.status === 200) {
                liveStatusElement.textContent = 'Client terdeteksi. Mengarahkan ke login...';
                window.location.href = '/';
                return true;
            }

            return false;
        } catch (_error) {
            return false;
        }
    }

    async function runLiveCheck() {
        const ok = await checkClientReady();
        if (!ok) {
            liveStatusElement.textContent = 'Belum terdeteksi, menunggu Client/Extension aktif...';
        }
    }

    retryButton.addEventListener('click', runLiveCheck);
    runLiveCheck();
    window.setInterval(() => {
        if (document.hidden) {
            return;
        }

        runLiveCheck();
    }, 8000);
</script>
</body>
</html>
