# Tangkapan Layar { #screenshots }

Jelajahi command line Juntei, penguraian otomatis, dan hasil investigasi yang tertata.
Klik sebuah tangkapan layar untuk melihatnya dalam ukuran penuh.

## 1. Startup dan perintah { #1-startup-and-commands }

<figure markdown>

[![Layar startup Juntei yang menampilkan rilis CODE BLUE dan perintah yang tersedia](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Layar startup Juntei mencantumkan penguraian otomatis, parser khusus per sumber, dan Persistence Hunter.</figcaption>

</figure>

## 2. Penguraian otomatis dalam waktu kurang dari 20 detik { #2-automatic-parsing-in-under-20-seconds }

Pada sebuah **MacBook Pro M4**, eksekusi yang ditampilkan di bawah ini mengurai **8,7 GiB data Windows yang dikumpulkan**
dari **enam sistem** dan menyimpan **4,4 GiB keluaran dalam waktu kurang dari 20 detik**.
Perintah [auto-parse](commands/auto-parse.md) menjalankan 240 tugas parser menggunakan 16 proses
paralel dan menghasilkan 482 berkas hasil. Terdapat **210 tugas yang berhasil, 30 dilewati
karena artefaknya tidak dikumpulkan, dan nol kegagalan**.

<figure markdown>

[![auto-parse Juntei menyelesaikan enam sistem, 8,7 GiB masukan dan 4,4 GiB keluaran dalam waktu kurang dari 20 detik](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Penemuan host otomatis, penguraian paralel, dan ringkasan penyelesaian yang menampilkan ukuran masukan, ukuran keluaran, hasil tugas, dan waktu yang berlalu.</figcaption>

</figure>

## 3. Hasil yang tertata untuk investigasi { #3-results-organized-for-investigation }

<figure markdown>

[![Folder keluaran satu sistem yang dikelompokkan ke dalam kategori DFIR, dengan CSV akun dan log parser-nya](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Hasil setiap komputer dikelompokkan berdasarkan kategori DFIR. CSV artefak dan berkas <code>-logs.txt</code> yang sesuai tetap berdampingan, sehingga mudah untuk menemukan baik catatan maupun detail penguraiannya.</figcaption>

</figure>
