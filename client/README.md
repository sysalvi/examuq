# ExamUQ Client

Desktop launcher untuk membuka portal siswa ExamUQ dengan header identitas client otomatis.

## Development

```bash
npm install
npm run start
```

## Build local

```bash
npm run dist
```

Output build ada di folder `dist/`.

## Behavior

- Menambahkan header request ke origin server yang dipilih:
  - `X-ExamUQ-Client-Type: desktop_client`
  - `X-ExamUQ-Source: client`
  - `X-ExamUQ-Device-Id: <generated-id>`
- Blok pembukaan tab/window baru pada portal ujian.
