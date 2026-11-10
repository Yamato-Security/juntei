# Bukti dan interpretasi { #evidence-and-interpretation }

Forensik digital bukan sekadar membaca field. Forensik digital adalah menentukan observasi apa yang didukung oleh field tersebut,
seberapa kuat dukungan itu, dan apa yang dapat menghasilkan observasi yang sama secara tidak berbahaya.

## Empat kategori bukti yang berguna { #four-useful-evidence-categories }

| Kategori | Makna | Contoh |
|---|---|---|
| Keberadaan | Sebuah path atau berkas diketahui oleh Windows | Catatan berkas Amcache, entri ShimCache, catatan `$MFT` |
| Eksekusi | Windows mencatat peluncuran program atau status eksekusi | Waktu eksekusi Prefetch; flag eksekusi ShimCache yang didukung |
| Interaksi pengguna | Pengguna atau aplikasi mereferensikan/membuka sebuah item | RecentDocs, Jump List, LNK di folder Recent |
| Aktivitas sistem | Windows mencatat penggunaan sumber daya atau konfigurasi | Penghitung SRUM, konfigurasi layanan, aksi TaskCache |

Sebuah artefak dapat mendukung lebih dari satu kategori, tetapi jangan pernah menaikkan tingkatnya secara otomatis. Misalnya,
entri Amcache dengan path yang mencurigakan mendukung **keberadaan**. Waktu eksekusi Prefetch yang cocok menambahkan
bukti **eksekusi** yang lebih kuat. Catatan pembuatan USN dan path `$MFT` dapat menempatkan berkas di disk, dan
karantina Defender dapat menjelaskan mengapa berkas tersebut menghilang.

## Timestamp adalah pernyataan, bukan kebenaran universal { #timestamps-are-statements-not-universal-truth }

Tanyakan apa yang menyebabkan setiap timestamp:

- Waktu `$STANDARD_INFORMATION` dan `$FILE_NAME` mendeskripsikan metadata NTFS dan dapat berbeda satu sama lain.
- Waktu eksekusi Prefetch mendeskripsikan peluncuran yang tercatat, tetapi hanya waktu eksekusi terbaru yang tersisa.
- Waktu ShimCache biasanya adalah waktu terakhir diubahnya berkas yang di-cache—bukan waktu eksekusi.
- Waktu last-write kunci registry berlaku untuk kunci secara keseluruhan, bukan untuk nilai individual.
- RecentDocs memberikan waktu kunci kepada item terbaru; item yang lebih lama memiliki urutan tetapi biasanya tidak memiliki waktu pembukaan.
- Timestamp SRUM sering kali mengidentifikasi interval agregasi, bukan operasi jaringan yang tepat.

Gunakan UTC selama analisis, simpan nilai asli bila tersedia, dan dokumentasikan setiap konversi.

## Bukti positif dan negatif { #positive-and-negative-evidence }

Keberadaan catatan yang dipahami dengan baik dapat menjadi bukti positif yang kuat. Ketiadaan biasanya
lebih lemah karena artefak dapat tertimpa (rollover), cache memiliki batas ukuran, pengumpulan mungkin tidak lengkap, pengaturan berbeda-beda,
dan alat pembersih itu ada. Katakan “tidak teramati dalam artefak yang dikumpulkan” alih-alih “tidak terjadi.”

## Tangga koroborasi { #corroboration-ladder }

1. Validasi berkas sumber dan path pengumpulan.
2. Pastikan parser tidak melaporkan kerusakan yang relevan atau log transaksi yang tidak diterapkan.
3. Interpretasikan field sesuai dengan halaman artefaknya.
4. Temukan artefak independen yang mendukung tindakan yang sama.
5. Periksa log peristiwa, telemetri jaringan, memori, atau konten berkas bila tersedia.
6. Catat versi Juntei, perintah, filter, dan hash sumber secara tepat.

!!! tip "Tulis kesimpulan sesuai kekuatan buktinya"
    Utamakan “Prefetch mencatat `tool.exe` berjalan pada 10:02:11Z” daripada “penyerang mengeksekusi
    malware tepat pada 10:02:11Z.” Pernyataan kedua menambahkan identitas dan niat yang tidak dapat
    dibuktikan oleh Prefetch saja.
