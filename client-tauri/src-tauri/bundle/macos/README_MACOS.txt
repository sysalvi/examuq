Panduan macOS untuk ExamUQ Client

Jika macOS menampilkan pesan bahwa aplikasi rusak atau tidak bisa dibuka, lakukan langkah berikut:

1. Pindahkan "ExamUQ Client.app" ke folder Applications.
2. Klik dua kali file "jalankan.command".
3. Jika diminta izin, lanjutkan saja.
4. Setelah proses selesai, aplikasi akan dibuka otomatis.

Jika ingin menjalankan manual lewat Terminal, gunakan perintah berikut:

  xattr -dr com.apple.quarantine "/Applications/ExamUQ Client.app"

Catatan:
- Bantuan ini disediakan karena aplikasi belum ditandatangani / notarized oleh Apple Developer.
- Solusi paling rapi jangka panjang tetap notarization Apple, tetapi untuk saat ini file ini adalah solusi bantu yang paling praktis.
