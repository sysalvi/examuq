# ExamUQ Client Tauri

Client desktop alternatif berbasis Tauri (lebih ringan dari Electron) dengan fitur setara:

- default server `http://10.10.20.2:6996`
- mode ujian fullscreen + anti-keluar dasar
- pengaturan server dalam overlay modal
- shortcut rahasia keluar:
  - arm: `Ctrl/Cmd + Shift + C`
  - execute: `Ctrl/Cmd + Shift + B` (dalam 5 detik)
  - saat execute, client mencoba end session dahulu sebelum menutup window ujian

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```
