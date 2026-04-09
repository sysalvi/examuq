# ExamUQ — Project Plan (Draft v1)

Dokumen ini adalah ringkasan perencanaan awal proyek **ExamUQ** (tanpa implementasi kode).

## 1) Tujuan Produk

Membangun sistem ujian berbasis web yang membungkus platform ujian pihak ketiga (contoh: Google Form / Quizizz) ke dalam aplikasi dengan kontrol anti-cheat dan monitoring terpusat.

## 2) Struktur Proyek

```text
/examuq
  /admin      -> ExamUQ Admin (Next.js + MySQL)
  /client     -> ExamUQ Client (Electron)
  /extension  -> ExamUQ Extension (mobile/browser path)
```

## 3) Komponen Sistem

### A. ExamUQ Admin
- Stack: Next.js + TypeScript + MySQL + ORM (Prisma)
- Fungsi utama:
  - Kelola data pengawas
  - Kelola data ujian
  - Generate token ujian
  - Monitoring sesi ujian realtime

### B. ExamUQ Client (Desktop)
- Stack: Electron + JavaScript/TypeScript
- Fungsi utama:
  - Input data siswa + token
  - Validasi token ke server admin
  - Menjalankan ujian dalam mode terbatas (anti-cheat)
  - Mengirim heartbeat/status sesi ke monitoring

### C. ExamUQ Extension (Mobile Focus)
- Fungsi utama:
  - Alternatif akses ujian untuk perangkat HP/non-laptop
  - Validasi token + data siswa
  - Monitoring event dasar (sesuai kemampuan platform)

## 4) Alur Utama Bisnis

1. Guru membuat ujian di Admin:
   - Input minimal: kelas, mata pelajaran, waktu ujian, link ujian
   - Output: token ujian

2. Siswa membuka Client/Extension:
   - Isi data siswa + token
   - Jika token valid, diarahkan ke halaman ujian

3. Monitoring pengawas:
   - Saat siswa ujian, status sesi tampil realtime di Admin

## 5) Data Ujian yang Direkomendasikan

Selain data minimal, disarankan menambah:
- Tahun ajaran & semester
- Tipe ujian (UH/UTS/UAS/tryout)
- Durasi ujian
- Grace period keterlambatan
- Mode token (global/per-siswa)
- Batas perangkat per siswa
- Whitelist domain ujian
- Aturan anti-cheat per ujian

## 6) Roadmap Implementasi (Tanpa Kode)

### Phase 0 — Finalisasi Requirement
- Definisi rule anti-cheat per OS
- Keputusan jalur mobile (extension vs PWA/app wrapper)

### Phase 1 — Desain Teknis
- ERD database
- API contract
- Sequence flow lengkap

### Phase 2 — Admin MVP
- Auth + role
- CRUD pengawas/ujian/token
- Monitoring realtime dasar

### Phase 3 — Client MVP
- Login siswa + token
- Webview ujian + event monitoring

### Phase 4 — Extension/Mobile MVP
- Validasi token + monitoring dasar

### Phase 5 — UAT & Hardening
- Uji beban pengguna
- Uji jaringan tidak stabil
- Uji bypass anti-cheat umum

## 7) Risiko & Catatan Penting

- Anti-cheat tidak bisa 100% foolproof, target realistis: deterrence + monitoring + evidence.
- Operasi level OS (contoh kill explorer.exe) harus disiapkan mekanisme restore aman.
- Wajib menjaga privasi data, audit log, dan kebijakan penggunaan yang jelas.

## 8) Next Deliverables (Sesi Berikutnya)

1. Desain ERD detail (tabel + relasi)
2. Spesifikasi endpoint API (request/response)
3. SOP Pengawas dan SOP Siswa
4. Blueprint anti-cheat policy per platform (Windows/macOS/mobile)
