---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, Dewi Forensik Digital](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> adalah parser lintas platform dari Yamato Security untuk artefak forensik Windows
yang <em>bukan</em> log peristiwa. Juntei mengubah bukti yang dikumpulkan menjadi catatan CSV, JSON, dan JSONL
yang konsisten sambil mempertahankan konteks yang dibutuhkan investigator untuk menghindari kesimpulan yang keliru.
</p>

<div class="jt-cta" markdown>
[Mulai di sini :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Pelajari artefak :material-archive-search:](artifacts/index.md){ .md-button }
[Referensi perintah :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## Dari artefak mentah menuju kesimpulan yang dapat dipertanggungjawabkan { #from-raw-artifact-to-defensible-conclusion }

Juntei dapat memberi tahu Anda apa yang dicatat oleh sebuah artefak; situs ini menjelaskan apa yang **dibuktikan dan tidak
dibuktikan** oleh catatan tersebut. Mulailah dari halaman artefak meskipun Anda belum pernah melakukan DFIR Windows sebelumnya.
Setiap panduan mencakup tujuannya, lokasi di disk, catatan mengenai retensi dan pengumpulan, perintah penguraian,
field keluaran, interpretasi, korelasi, dan bacaan lebih mendalam.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Riwayat sistem berkas__

    ---

    Gunakan `$MFT` dan Jurnal USN untuk merekonstruksi keberadaan berkas, nama, perpindahan, penghapusan,
    perubahan metadata, dan alternate data stream.

-   :material-play-circle-outline:{ .lg .middle } __Bukti program__

    ---

    Prefetch dapat memberikan dukungan kuat atas eksekusi. Amcache dan ShimCache menambahkan konteks inventaris, path, hash,
    kompatibilitas, dan historis—tetapi memerlukan interpretasi yang cermat.

-   :material-account-clock:{ .lg .middle } __Aktivitas pengguna__

    ---

    Berkas LNK, Jump Lists, RecentDocs, dan SRUM mengungkap item yang dibuka, aktivitas aplikasi,
    lokasi lepas pasang atau jaringan, penggunaan sumber daya, dan konteks historis pengguna.

-   :material-shield-bug:{ .lg .middle } __Bukti keamanan__

    ---

    Karantina Defender dapat mempertahankan label ancaman, path asli, asal unduhan, hash,
    pemilik, dan bahkan memulihkan byte yang dikarantina secara aman untuk analisis di lingkungan terisolasi.

-   :material-database-search:{ .lg .middle } __Status registry__

    ---

    Baca data registry yang teralokasi maupun yang dapat dipulihkan, putar ulang log transaksi, dan periksa layanan,
    tugas terjadwal, akun lokal, serta status dokumen terbaru.

-   :material-link-variant:{ .lg .middle } __Korelasikan, jangan menebak__

    ---

    Tidak ada satu artefak pun yang menceritakan keseluruhan kisah. Panduan-panduan ini menunjukkan cara menggabungkan sumber-sumber independen
    menjadi timeline yang didukung bukti.

</div>

!!! warning "Keluaran forensik adalah bukti, bukan putusan"
    Path yang tampak mencurigakan adalah petunjuk. Sebuah timestamp mungkin mendeskripsikan metadata berkas, bukan
    eksekusi. Catatan yang tidak ada tidak membuktikan bahwa suatu tindakan tidak terjadi. Pertahankan bukti sumber,
    catat versi alat dan perintah yang digunakan, serta korroborasi kesimpulan-kesimpulan penting.
