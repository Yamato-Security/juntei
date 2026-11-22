# Pemindaian YARA { #yara-scan }

Pindai berkas yang dikumpulkan menggunakan mesin Rust portabel. Mendukung Windows, macOS, dan Linux. Berkas dibaca sebagai data dan tidak pernah dijalankan. Perintah ini terpisah dari penguraian artefak otomatis.

## Pilih aturan { #choose-rules }

Core menjadi bawaan dengan prioritas akurasi dan kinerja. Extended memperluas perburuan. Full memiliki cakupan terluas, tetapi dapat menimbulkan lebih banyak positif palsu.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Gunakan berkas atau direktori aturan UTF-8 lokal tanpa mengunduh. Direktori ditelusuri secara rekursif. Include dibatasi pada direktori aturan yang dipilih dan diperluas sekali sesuai urutan dependensi. Aturan tidak valid atau tidak didukung menghentikan kompilasi, bukan dibuang diam-diam.

## Batas dan cakupan { #limits-and-scope }

Semua berkas reguler dapat dipindai, termasuk muatan yang dipulihkan dari karantina. Tautan simbolis dan berkas khusus dilewati. Arsip dan citra disk tidak diekstrak; memori aktif tidak dipindai. Gunakan berkas hasil ekstraksi yang stabil dan direktori keluaran baru di luar bukti.

- `-T, --threads <N>`: Jumlah pemindaian paralel; bawaan mengikuti jumlah CPU, maksimal 8.
- `--timeout <SECONDS>`: Batas waktu evaluasi per berkas dalam detik; bawaan 60.
- `--max-file-mib <MIB>`: Batas ukuran berkas opsional dalam MiB; tanpa batas secara bawaan. Berlaku juga untuk MFT kecuali diganti. Berkas di atas batas yang ditetapkan dilaporkan sebagai dilewati.
- `--rules-release YYYYMMDD`: Tanggal rilis Forge atau latest sebagai bawaan. Sumber yang diunduh dan hash disimpan untuk reproduksi.

## Hasil { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Kecocokan aturan/berkas dengan jalur relatif, nama komputer sumber bukti, hash, metadata, dan offset kecocokan terbatas. Byte yang cocok tidak diekspor.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Cakupan per berkas, termasuk yang dilewati, kesalahan baca, batas waktu, dan hasil sebagian.
- `Summary.json`: Status penyelesaian, pengaturan, jumlah, versi mesin, dan hash sumber.
- `Rules.yar`: Aturan unduhan persis atau salinan aturan lokal dengan include yang diperluas.
- `CompilationWarnings.txt` / `CompilationError.txt`: Peringatan kompilasi atau kesalahan jika kompilasi gagal.

Kode keluar: 0 selesai, 3 cakupan tidak lengkap, 2 kegagalan penyiapan/keluaran, dan 130 pembatalan. Kecocokan adalah petunjuk, bukan bukti kompromi; tidak ada kecocokan juga tidak membuktikan keamanan. Tinjau judul aturan dan konteks artefak. Dekripsi karantina sebelum mengidentifikasi muatannya.

## Kecocokan MFT dan pemulihan { #mft-recovery }

MFT diproses satu per satu tanpa batas ukuran bawaan dalam dua tahap: byte mentah dipetakan ke memori dan dipindai seperti berkas lain, lalu setiap rekaman FILE dibaca dari berkas satu per satu dan setiap aliran DATA residen dipindai sebagai berkas tersendiri, apa pun hasil pemindaian mentahnya. Hanya aliran yang cocok sendiri yang dipulihkan. Isi nonresiden dihitung, bukan diekspor, karena byte-nya tidak ada di MFT; rekaman yang rusak dilaporkan dan membuat cakupan tidak lengkap.

`--max-file-mib <MIB>` → `--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

Pemulihan dibatasi hingga 1000 aliran per eksekusi secara bawaan. Batas, rekaman rusak, dan batas waktu ditampilkan. Nama hasil memakai ekstensi karantina; struktur direktori bukti tidak dibuat ulang.

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## Rincian lebih lanjut { #further-details }

- [Mesin: Boreal](https://github.com/vthib/boreal)
- [Paket aturan: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Penggunaan lengkap dan referensi kolom (Inggris)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Penggunaan lengkap dan referensi kolom (Jepang)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
