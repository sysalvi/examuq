# ExamUQ Client Tauri

Client desktop alternatif berbasis Tauri (lebih ringan dari Electron) dengan fitur setara:

- default server `http://10.10.20.2:6996`
- mode ujian fullscreen + anti-keluar dasar
- pengaturan server dalam overlay modal
- shortcut rahasia keluar (darurat operator):
  - `Ctrl + Shift + X` (Windows/Linux)
  - `Cmd + Shift + X` (macOS)
- siswa dapat keluar normal lewat tombol **Selesai Ujian** (konfirmasi), lalu kembali ke launcher

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

## Tauri v2 Updater (stable/beta via build-time env)

Konfigurasi updater di project ini memakai **build-time channel**, tanpa toggle UI.

### 1) Generate signing key

```bash
npx tauri signer generate -w ~/.tauri/examuq-updater.key
```

Lalu salin public key (`*.pub`) ke `src-tauri/tauri.conf.json` pada:

- `plugins.updater.pubkey`

> `pubkey` di repo saat ini adalah contoh. Ganti dengan public key milik environment release Anda.

### 2) Build per channel

```bash
npm run build:stable
npm run build:beta
```

Channel dipilih dari env build-time `EXAMUQ_UPDATE_CHANNEL` dan dipakai di Rust untuk menentukan endpoint updater.

Beta hanya aktif jika build memakai `EXAMUQ_ALLOW_BETA=true`. Tanpa flag ini, channel otomatis dipaksa ke `stable`.

### 3) Signing env saat build

```bash
TAURI_SIGNING_PRIVATE_KEY="<base64-private-key>" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD="<password-jika-ada>" \
npm run build:stable
```

Atau gunakan `TAURI_SIGNING_PRIVATE_KEY_PATH` jika private key disimpan sebagai file.
