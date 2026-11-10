# Investigasi pertama { #first-investigation }

Alur kerja ini mengasumsikan Anda menerima hasil pengumpulan triase offline di `evidence/`.

## 1. Pertahankan sumber { #1-preserve-the-source }

Bekerjalah dari salinan yang telah diverifikasi. Catat hash, waktu pengumpulan, zona waktu, host, volume, dan alat
pengumpulan. Juntei membaca bukti, tetapi opsi seperti ekstraksi resident stream atau karantina membuat
berkas keluaran baru; arahkan berkas-berkas tersebut ke direktori analisis yang terpisah.

## 2. Inventarisasi apa yang Anda miliki { #2-inventory-what-you-have }

Cari path yang tercantum di [Pengumpulan bukti](collection.md). Artefak yang hilang adalah hal yang wajar. Jangan
berasumsi bahwa hasil pengumpulan berisi log transaksi hanya karena berisi hive registry.

## 3. Jalankan ringkasan terlebih dahulu { #3-run-summaries-first }

Banyak perintah mencetak ringkasan singkat ketika tidak ada berkas keluaran yang diminta:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Baca peringatan kerusakan, database dirty, dan log yang belum diterapkan sebelum memercayai record.

Untuk mengurai setiap artefak yang didukung dalam hasil pengumpulan dari banyak komputer dalam satu operasi, gunakan:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Lihat [Analisis otomatis hasil pengumpulan](../commands/auto-parse.md) untuk penemuan host, paralelisme,
tata letak keluaran, dan JSONL gabungan untuk ingestion SIEM.

## 4. Simpan keluaran terstruktur { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Utamakan JSONL untuk scripting dan nilai bersarang, dan CSV untuk spreadsheet atau penampil timeline.

## 5. Lakukan pivot dari pengenal yang andal { #5-pivot-from-a-reliable-identifier }

Mulailah dengan nama berkas, path lengkap, SHA-1/SHA-256, SID pengguna, entri/sequence MFT, GUID tugas,
nama layanan, atau jendela timestamp. Normalisasi huruf besar/kecil hanya untuk field yang diperlakukan Windows secara tidak peka huruf besar/kecil.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Korelasikan { #6-correlate }

Gunakan [Investigasi lintas artefak](../resources/workflows.md). Urutan yang praktis adalah:

1. Prefetch untuk eksekusi dan waktu eksekusi terbaru.
2. `$MFT` dan Jurnal USN untuk pembuatan, path, penggantian nama, dan penghapusan.
3. Amcache dan ShimCache untuk keberadaan historis dan metadata.
4. LNK, Jump Lists, RecentDocs, dan SRUM untuk konteks pengguna/aplikasi.
5. Layanan registry dan TaskCache untuk persistensi.
6. Karantina Defender untuk deteksi, penghapusan, asal, dan hash.

## 7. Dokumentasikan reprodusibilitas { #7-document-reproducibility }

Catat path sumber, hash sumber, versi Juntei, perintah yang persis, hash keluaran, zona waktu sistem,
dan setiap pemfilteran yang dilakukan kemudian. Jangan pernah mengedit keluaran bukti yang dihasilkan secara diam-diam.
