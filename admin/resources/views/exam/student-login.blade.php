<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login Siswa - ExamUQ</title>
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
                radial-gradient(90vw 90vw at 10% -10%, #22d3ee44 0%, transparent 60%),
                radial-gradient(80vw 80vw at 100% 0%, #c084fc33 0%, transparent 58%),
                linear-gradient(140deg, #0f172a 0%, #111827 45%, #1e1b4b 100%);
            display: grid;
            place-items: center;
            padding: 20px;
        }

        .wrap {
            width: 100%;
            max-width: 640px;
        }

        .steps {
            display: grid;
            grid-template-columns: repeat(3, minmax(0, 1fr));
            gap: 8px;
            margin-bottom: 12px;
        }

        .step {
            border: 1px solid #334155;
            border-radius: 999px;
            text-align: center;
            padding: 7px 10px;
            font-size: 12px;
            background: #0b1227;
            color: #cbd5e1;
        }

        .step.active {
            border-color: #22d3ee;
            color: #67e8f9;
            background: #083344;
        }

        .card {
            border: 1px solid #334155;
            border-radius: 18px;
            padding: 24px;
            background: #0f172ae8;
            box-shadow: 0 18px 45px #02061766;
            animation: cardEntrance 360ms ease-out;
        }

        .badge {
            display: inline-block;
            margin-bottom: 10px;
            border-radius: 999px;
            padding: 5px 10px;
            font-size: 12px;
            font-weight: 800;
            letter-spacing: 0.3px;
            color: #022c22;
            background: #86efac;
            animation: badgePulse 2200ms ease-in-out infinite;
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
            background: #083344;
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

        .subtitle {
            margin: 0 0 20px;
            color: #cbd5e1;
            line-height: 1.45;
            font-size: 14px;
        }

        form {
            display: grid;
            gap: 12px;
        }

        .field {
            display: grid;
            gap: 6px;
            animation: fieldRise 300ms ease-out both;
        }

        .field:nth-of-type(1) {
            animation-delay: 40ms;
        }

        .field:nth-of-type(2) {
            animation-delay: 90ms;
        }

        .field:nth-of-type(3) {
            animation-delay: 140ms;
        }

        label {
            font-size: 13px;
            font-weight: 800;
            color: #e2e8f0;
        }

        input {
            width: 100%;
            border: 1px solid #475569;
            border-radius: 12px;
            background: #0b1227;
            color: #f8fafc;
            padding: 12px 14px;
            font-size: 14px;
            outline: none;
        }

        input:focus {
            border-color: #22d3ee;
            box-shadow: 0 0 0 3px #22d3ee33;
        }

        .error {
            margin: 0;
            font-size: 12px;
            color: #fda4af;
        }

        .cta {
            margin-top: 4px;
            border: 1px solid #14b8a6;
            border-radius: 12px;
            background: linear-gradient(135deg, #0d9488 0%, #06b6d4 100%);
            color: #f8fafc;
            padding: 12px 14px;
            font-size: 15px;
            font-weight: 900;
            cursor: pointer;
            transition: transform 120ms ease, filter 120ms ease;
        }

        .cta:hover {
            filter: brightness(1.06);
            transform: translateY(-1px);
        }

        .cta:active {
            transform: translateY(0);
            filter: brightness(0.98);
        }

        .cta:focus-visible {
            outline: 3px solid #22d3ee77;
            outline-offset: 1px;
        }

        .meta {
            margin-top: 14px;
            display: flex;
            justify-content: space-between;
            gap: 10px;
            flex-wrap: wrap;
            font-size: 12px;
            color: #94a3b8;
        }

        .meta a {
            color: #67e8f9;
            font-weight: 700;
            text-decoration: underline;
        }

        @media (max-width: 640px) {
            .card {
                padding: 18px;
                border-radius: 14px;
            }

            .steps {
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

        @keyframes fieldRise {
            from {
                opacity: 0;
                transform: translateY(6px);
            }

            to {
                opacity: 1;
                transform: translateY(0);
            }
        }

        @keyframes badgePulse {
            0%,
            100% {
                transform: translateY(0);
                filter: brightness(1);
            }

            50% {
                transform: translateY(-1px);
                filter: brightness(1.06);
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
<div class="wrap">
    <div class="steps" aria-hidden="true">
        <div class="step active">1. Login</div>
        <div class="step">2. Konfirmasi</div>
        <div class="step">3. Kerjakan Ujian</div>
    </div>

    <main class="card" role="main" aria-labelledby="title">
        <span class="badge">MISI UJIAN</span>
        <div class="title-row">
            <span class="title-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 3l7 4v5c0 5-3.4 8.4-7 9-3.6-.6-7-4-7-9V7l7-4z"/>
                    <path d="M9.5 12.2l1.8 1.8 3.6-3.6"/>
                </svg>
            </span>
            <h1 id="title">Siap Masuk Arena Ujian?</h1>
        </div>
        <p class="subtitle">Isi data kamu dulu. Kalau token benar, kamu lanjut ke halaman konfirmasi sebelum mulai mengerjakan.</p>

        <form method="post" action="{{ route('student.login.submit') }}">
            @csrf

            <div class="field">
                <label for="display_name">Nama</label>
                <input id="display_name" name="display_name" type="text" required autocomplete="off" value="{{ old('display_name') }}">
                @error('display_name')
                    <p class="error">{{ $message }}</p>
                @enderror
            </div>

            <div class="field">
                <label for="class_room">Kelas / Ruang</label>
                <input id="class_room" name="class_room" type="text" required autocomplete="off" value="{{ old('class_room') }}">
                @error('class_room')
                    <p class="error">{{ $message }}</p>
                @enderror
            </div>

            <div class="field">
                <label for="token_global">Token Ujian</label>
                <input id="token_global" name="token_global" type="text" required autocomplete="off">
                @error('token_global')
                    <p class="error">{{ $message }}</p>
                @enderror
            </div>

            <button class="cta" type="submit">Lanjut Konfirmasi</button>
        </form>

        <div class="meta">
            <span>Akses khusus lewat ExamUQ Client / Extension</span>
            <span>Admin? <a href="/admin">Masuk di sini</a></span>
        </div>
    </main>
</div>
</body>
</html>
