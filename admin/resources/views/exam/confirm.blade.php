<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Konfirmasi Ujian - {{ $exam->title }}</title>
    <style>
        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            min-height: 100vh;
            font-family: "Nunito", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            color: #e2e8f0;
            background:
                radial-gradient(90vw 90vw at 8% -15%, #22d3ee33 0%, transparent 60%),
                radial-gradient(80vw 80vw at 100% 0%, #a78bfa33 0%, transparent 58%),
                linear-gradient(140deg, #0f172a 0%, #111827 50%, #1e1b4b 100%);
            display: grid;
            place-items: center;
            padding: 20px;
        }

        .card {
            width: 100%;
            max-width: 640px;
            border: 1px solid #334155;
            border-radius: 18px;
            background: #111827e8;
            padding: 24px;
            box-shadow: 0 18px 45px #02061766;
            animation: cardEntrance 360ms ease-out;
        }

        .tag {
            display: inline-block;
            margin-bottom: 8px;
            border-radius: 999px;
            padding: 5px 10px;
            font-size: 12px;
            font-weight: 900;
            letter-spacing: 0.3px;
            color: #082f49;
            background: #67e8f9;
        }

        h1 {
            margin: 0 0 8px;
            font-size: clamp(24px, 5vw, 34px);
            line-height: 1.1;
        }

        .title-row {
            display: flex;
            align-items: center;
            gap: 12px;
        }

        .title-icon {
            width: 38px;
            height: 38px;
            border-radius: 10px;
            border: 1px solid #22d3ee77;
            background: #082f49;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            color: #67e8f9;
            flex-shrink: 0;
            animation: iconFloat 2.4s ease-in-out infinite;
        }

        .title-icon svg {
            width: 20px;
            height: 20px;
        }

        p {
            margin: 0 0 16px;
            color: #cbd5e1;
            line-height: 1.45;
            font-size: 14px;
        }

        dl {
            margin: 0;
            display: grid;
            grid-template-columns: 160px 1fr;
            gap: 10px 14px;
            font-size: 14px;
            padding: 14px;
            border-radius: 12px;
            border: 1px solid #334155;
            background: #0b1227;
        }

        dt {
            color: #94a3b8;
            font-weight: 700;
        }

        dd {
            margin: 0;
            font-weight: 800;
            color: #f8fafc;
        }

        .actions {
            margin-top: 20px;
            display: flex;
            justify-content: flex-end;
        }

        .btn {
            border: 1px solid #14b8a6;
            border-radius: 12px;
            background: linear-gradient(135deg, #0d9488 0%, #06b6d4 100%);
            color: #ffffff;
            font-size: 15px;
            font-weight: 900;
            padding: 11px 18px;
            cursor: pointer;
            transition: transform 120ms ease, filter 120ms ease;
        }

        .btn:hover {
            filter: brightness(1.05);
            transform: translateY(-1px);
        }

        .btn:active {
            transform: translateY(0);
        }

        .hint {
            margin-top: 12px;
            font-size: 12px;
            color: #94a3b8;
            text-align: right;
        }

        @media (max-width: 640px) {
            .card {
                padding: 18px;
            }

            dl {
                grid-template-columns: 1fr;
            }
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
    <span class="tag">KONFIRMASI</span>
    <div class="title-row">
        <span class="title-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2.8l8 4.4v5.6c0 5.8-3.8 9.6-8 10.4-4.2-.8-8-4.6-8-10.4V7.2l8-4.4z"/>
                <path d="M8.8 12.1l2.2 2.2 4.2-4.2"/>
            </svg>
        </span>
        <h1 id="title">Konfirmasi Sebelum Ujian</h1>
    </div>
    <p>Pastikan data berikut benar. Setelah klik <strong>Mulai Ujian</strong>, sistem akan mencoba masuk ke mode fullscreen.</p>

    <dl>
        <dt>Nama</dt>
        <dd>{{ $examSession->display_name }}</dd>

        <dt>Kelas / Ruang</dt>
        <dd>{{ $examSession->class_room }}</dd>

        <dt>Nama Ujian</dt>
        <dd>{{ $exam->title }}</dd>
    </dl>

    <form id="startExamForm" class="actions" method="get" action="{{ route('exam.gateway') }}">
        <input type="hidden" name="lt" value="{{ request()->query('lt') }}">
        <button id="startExamButton" class="btn" type="button">Mulai Ujian</button>
    </form>
    <p class="hint">Jika fullscreen gagal otomatis, sistem tetap meminta fullscreen di halaman berikutnya.</p>
</main>

<script>
    const startExamButton = document.getElementById('startExamButton');
    const startExamForm = document.getElementById('startExamForm');

    async function tryEnterFullscreen() {
        if (document.fullscreenElement) {
            return;
        }

        try {
            await document.documentElement.requestFullscreen();
        } catch (_error) {
            console.warn('Fullscreen request failed before exam start:', _error);
        }
    }

    startExamButton.addEventListener('click', async () => {
        await tryEnterFullscreen();
        startExamForm.submit();
    });
</script>
</body>
</html>
