# Analisis otomatis hasil pengumpulan { #automatic-collection-parsing }

`auto-parse` adalah cara tercepat untuk mengubah hasil pengumpulan triase Windows dari banyak komputer menjadi
direktori analisis yang terstruktur. Berikan root bukti dan sebuah direktori keluaran baru. Perintah ini menemukan
sistem-sistem yang dikumpulkan, mengidentifikasi nama komputernya, dan menjalankan setiap parser Juntei yang kompatibel secara
paralel.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

Keluaran default adalah CSV. Tambahkan timeline JSONL untuk `jq` atau impor SIEM dengan tetap mempertahankan hasil
CSV tersebut:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Pisahkan pohon direktori keluaran dan bukti"
    Juntei menolak direktori keluaran yang berada di dalam `<EVIDENCE>` atau berisi `<EVIDENCE>`. Hal ini
    mencegah data yang dihasilkan ditemukan kembali sebagai bukti dan mencegah `--clobber`
    menyentuh berkas di koleksi sumber.

## Image disk VHDX { #vhdx-disk-images }

Dengan `--vhdx`, `auto-parse` mengurai sistem di dalam setiap image disk VHDX pada direktori bukti. Setiap image di-mount hanya-baca melalui FUSE selama penguraian, tanpa mengekstrak berkas. Lihat [Memasang Image Disk](mount.md).

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

Jika sebuah volume NTFS tidak dapat dibuka, Juntei melaporkan masalahnya dan mempertahankan volume yang dapat dibaca. Untuk citra yang hanya terbaca sebagian, `auto-parse --vhdx` mengembalikan kode keluar 1; `mount` juga mengembalikan 1 setelah pelepasan.

## Zona waktu keluaran { #output-time-zone }

Timestamp yang diurai secara default menggunakan UTC, terlepas dari zona waktu lokal komputer analis. UTC
menggunakan bentuk log peristiwa Windows `2026-10-20T08:46:38.2001668Z`: tanggal/waktu ISO 8601, `T`
di antara tanggal dan waktu, dan `Z` untuk UTC. Digit pecahan yang tersedia dipertahankan.
`GeneratedUTC` di `manifest.json` juga menggunakan bentuk ini. `Start time` dan `End time` pada konsol
menggunakan waktu lokal komputer analis, seperti `2026-11-04 09:15:02`, apa pun zona waktu
keluarannya.

Pilih offset tampilan tetap dengan `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Offset adalah nilai `HH:MM` dua digit bertanda (jam `00–23`, menit `00–59`). Nama seperti
`JST`, nilai yang disingkat seperti `+9`, dan rentang yang tidak valid menghasilkan error beserta contoh sintaksis.
Hilangkan opsi ini atau gunakan `--timezone "+00:00"` untuk UTC. Offset tetap tidak menerapkan aturan
daylight saving.

Opsi ini berlaku untuk timestamp yang diurai dalam hasil CSV, JSON, dan JSONL per artefak serta
ekspor JSONL `-j` tambahan. Misalnya, `2026-10-20T08:46:38.2001668Z` menjadi
`2026-10-20T17:46:38.2001668+09:00`; keduanya mendeskripsikan saat yang sama.

Nama field tetap stabil, termasuk nama yang sudah ada yang berakhiran `UTC`. Nilai yang dikonversi secara eksplisit
membawa offset yang dipilih. `OutputTimeZone` di kedua manifest mencatat offset tampilan
yang diminta, sedangkan `GeneratedUTC` tetap berupa metadata eksekusi dalam UTC. Log per parser mempertahankan ringkasan UTC asli dari parser.

Angka FILETIME/epoch mentah, dictionary nilai registry, konfigurasi zona waktu sumber, metadata
teks, nilai yang hanya berisi tanggal, dan nilai jam lokal tanpa offset yang diketahui dipertahankan. Juntei
tidak menambahkan `Z` ke waktu lokal yang tidak diketahui. Jam shell item yang berasal dari FAT (`Item*UTC` di
ShellBags/OpenSaveMRU dan `TargetItems` LNK/Jump List) tidak memiliki offset yang tercatat; label
UTC lama atau sufiks `Z` yang ditampilkan tidak membuktikan UTC, sehingga opsi offset mempertahankannya.
Kolom CSV yang berisi ringkasan teks bebas mempertahankan
teks yang tercatat; JSON/JSONL mempertahankan timestamp terurai yang bersarang sebagai field terstruktur.

UTC menggunakan keluaran parser yang ada secara langsung. Offset eksplisit yang bukan nol menambahkan tahap
pemformatan streaming di dalam setiap worker parser; ini mengubah nilai tampilan tanpa mengubah bukti.

Jika sebuah offset akan memindahkan timestamp ke luar tahun `0000–9999`, nilai tersebut mempertahankan offset
eksplisit aslinya. Nilai dengan lebih dari sembilan digit pecahan juga tetap tidak berubah alih-alih
dibulatkan. Juntei mencatat peringatan di berkas `*-logs.txt` milik parser dan mengonversi timestamp
lainnya. CSV/JSON yang rusak atau error keluaran tetap menggagalkan tugas dan membiarkan berkas hasil
asli yang terpengaruh tidak berubah.

## Apa yang ditemukannya { #what-it-discovers }

Juntei mencari secara rekursif direktori bernama `Windows` yang berisi direktori `System32`, `Prefetch`,
atau `AppCompat`, tanpa membedakan huruf besar/kecil. Pengenalan dua yang terakhir juga mendukung pengumpulan
terarah yang hanya berisi Prefetch atau Amcache. Ini mengecualikan direktori profil pengguna seperti
`AppData\Roaming\Microsoft\Windows`. Instalasi lama yang bersarang seperti
`C:\Windows.old\Windows` ditetapkan ke volume luar yang dikumpulkan alih-alih menjadi komputer
duplikat. Kedalaman maksimum default adalah 10. Ini mencakup path Velociraptor yang umum seperti:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Ubah batas ini hanya jika koleksi bersarang lebih dalam:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Untuk setiap sistem yang ditemukan, nama komputer dipilih dengan urutan berikut:

1. `hostname` di `client_info.json` pada direktori leluhur terdekat (dengan nilainya dipertahankan sebagai `FQDN` di manifest).
2. `ComputerName` di hive registry `SYSTEM` yang dikumpulkan.
3. Nama direktori koleksi terdekat yang berguna, dengan melewati drive dan struktur `uploads/auto` Velociraptor
   serta menghapus sufiks seperti `_F.…`.

Nama dibuat aman untuk digunakan sebagai nama direktori. Nama duplikat menerima sufiks numerik baik di
direktori keluarannya maupun nilai `Comp` sehingga sistem yang berbeda tetap dapat dibedakan di JSON.
Manifest menyimpan nama host persis seperti yang ditemukan dalam field `Hostname` terpisah.

Setelah mencantumkan sistem-sistem tersebut, `auto-parse` mencetak satu baris `Timezone:` tepat di atas
`Parser tasks`. Baris ini menampilkan zona waktu keluaran yang dipilih: `UTC` secara default, atau nilai seperti
`UTC+09:00` ketika `--timezone "+09:00"` digunakan. Labelnya berwarna hijau dan nilainya menggunakan
warna teks default terminal, seperti nilai ringkasan lainnya. `-K, --no-color` membuat baris tersebut polos.
Konfigurasi zona waktu sistem yang dikumpulkan tersedia di hasil
[Informasi sistem](../artifacts/system-info.md).

## Parser yang dijalankannya { #parsers-it-runs }

Kegagalan satu parser tidak menghentikan parser lainnya. `auto-parse` menjalankan tugas-tugas ini untuk setiap komputer
yang ditemukan. Tugas-tugas ini menggunakan perintah yang dikelompokkan berdasarkan sumber yang sama seperti penguraian manual, seperti
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum`, dan `parse-file prefetch`:

- Amcache
- `$MFT`
- Record redo/undo NTFS `$LogFile`
- Record Prefetch dan timeline eksekusi Prefetch terpisah
- Jurnal USN
- Indeks direktori `$I30`
- Sektor boot `$Boot` dan deskriptor keamanan `$Secure:$SDS`
- SRUM
- Pencatatan Akses Pengguna (UAL)
- Riwayat, unduhan, cookie, dan riwayat formulir browser web (browser berbasis Chromium dan Firefox)
- Pintasan LNK
- Jump Lists
- ShimCache
- Persistence Hunter (131 pemeriksaan persistensi lintas sumber dengan profil allow/block)
- Metadata karantina Defender
- Riwayat dialog Open dan Save As
- semua key dan nilai registry, termasuk data terhapus yang dapat dipulihkan
- layanan, RecentDocs, TaskCache, akun SAM, lokasi autostart registry, UserAssist, BAM,
  program terinstal, MuiCache, ShellBags, dan perangkat USB
- informasi sistem: nama, versi Windows, zona waktu, shutdown terakhir, profil, dan SID
- kebijakan audit: peristiwa keamanan mana yang diatur untuk dicatat oleh sistem
- konfigurasi boot (BCD dan BCD-Template)
- profil jaringan, antarmuka, share, port proxy, dan route
- nama dan waktu LSA secrets, serta cached domain logon yang sedang digunakan (tidak ada yang didekripsi)

Persistence Hunter mempertahankan setiap tingkat keparahan (`--min-severity info`), termasuk kecocokan allow parsial.
`LOW` adalah klasifikasi untuk diselidiki; metadata signature yang tidak ada juga dapat menghasilkannya untuk perintah
berbahaya. Inventaris otomatis tidak membuang entri-entri ini dengan menggunakan ambang batas medium
default dari perintah manual.

Hunter menggunakan satu worker di dalam setiap proses parser otomatis. Ini mencegah `auto-parse -T <N>`
melipatgandakan anggaran prosesnya dengan pool lain seukuran jumlah CPU di setiap proses anak. `persistence-hunter -T <N>`
yang dijalankan secara mandiri mengendalikan thread worker milik hunter itu sendiri.

Keluaran CSV `$MFT` menyertakan semua timestamp dan nama pendek. Keluaran registry menyertakan data mentah dan
interpretasi FILETIME untuk nilai `REG_QWORD`. Keluaran SRUM JSON dan JSONL menyertakan metadata
value-state. Berkas Defender dan stream `$MFT` residen **tidak diekstrak** secara otomatis.

Dengan `--mft-slack`, keluaran `$MFT` juga menyertakan nama berkas yang tersisa di slack record FILE, sama seperti yang ditulis oleh [`parse-ntfs mft -x`](../artifacts/mft.md): baris bertanda `FromSlack` dalam CSV, dan `SlackFileNames` dalam JSON dan JSONL. `--code-page` berlaku untuk berkas LNK, Jump List, ShellBags, dan OpenSaveMRU.

Artefak yang tidak dikumpulkan merupakan tugas `skipped` yang normal, bukan kegagalan.

## Tata letak keluaran { #output-layout }

Hasil diorganisasikan berdasarkan komputer dan kategori investigasi. Direktori kategori menggunakan
PascalCase, seperti `Accounts` dan `FileAndFolderOpening`. Tabel Amcache dan Jump Lists berada
langsung di kategorinya dengan prefiks nama berkas. Nama direktori komputer mempertahankan ejaan sebagaimana ditemukan. Direktori keluaran
yang sudah ada tidak dimigrasikan; pilih direktori keluaran baru saat beralih dari penamaan yang lama. Setiap parser menulis
berkas `<artifact>-logs.txt` di samping hasilnya, termasuk ketika artefak tidak dikumpulkan.
CSV menyimpan tabel terpisah karena setiap artefak memiliki kolom yang berbeda:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   └── Browser-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Kategori | Artefak |
|---|---|
| `Accounts` | Akun lokal SAM, nama dan waktu LSA secrets |
| `Applications` | Riwayat aplikasi (jejak alat pihak ketiga) |
| `Applications` | Program terinstal |
| `BrowserActivity` | Riwayat, unduhan, cookie, dan riwayat formulir browser web, termasuk entri terhapus yang dipulihkan |
| `ExternalDevices` | Perangkat dan volume USB |
| `FileAndFolderOpening` | Jump Lists, pintasan LNK, Microsoft Office, daftar MRU, dialog Open dan Save As, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, Jurnal USN |
| `NetworkActivity` | Profil jaringan, antarmuka, share, port proxy, dan route; koneksi, lalu lintas, dan push notification SRUM |
| `LateralMovement` | Koneksi keluar Remote Desktop, PuTTY, dan WinSCP milik pengguna, mapped drive, dan share; Pencatatan Akses Pengguna (klien dari role-role sebuah server) |
| `Persistence` | Konfigurasi boot (BCD), autostart registry, layanan, TaskCache |
| `ProcessExecution` | Kompatibilitas aplikasi, BAM dan DAM, FeatureUsage dan item yang disematkan, MuiCache, Prefetch, timeline Prefetch, aktivitas aplikasi/CPU SRUM, UserAssist |
| `Registry` | Key dan nilai hive umum, termasuk record yang dipulihkan |
| `SecurityProducts` | Metadata karantina Defender dan konten hasil ekstraksi opsional; pengaturan keamanan (Defender, UAC, perlindungan kredensial, RDP, firewall, logging) |
| `SystemInformation` | Nama, versi Windows, zona waktu, shutdown terakhir, profil, SID, dan kebijakan audit; energi/kapasitas SRUM, identity map, dan provider yang tidak dikenal |

Kategori membantu mengorganisasikan investigasi; kategori bukan bukti adanya aktivitas tertentu. Misalnya,
entri Amcache dan ShimCache saja tidak menetapkan adanya eksekusi. SRUM mencatat penggunaan sumber daya;
lihat panduan artefak sebelum menarik kesimpulan bahwa sebuah program telah berjalan.

CSV, JSON, dan JSONL menggunakan kategori yang sama. Satu berkas `<artifact>-logs.txt` per tugas parser berisi
perintah persis dan pesan-pesannya; misalnya, `HOST1/FileSystem/MFT-logs.txt` dan
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON dan JSONL menggunakan nama berkas log yang sama.
Amcache dan Jump Lists masing-masing memiliki satu log bersama di samping tabel-tabelnya yang berprefiks. Tabel provider SRUM ditempatkan langsung
di `NetworkActivity`, `ProcessExecution`, dan `SystemInformation`, dengan berkas
`SystemInformation/SRUM-logs.txt` bersama. Tidak ada subdirektori `logs/` yang dibuat.

Nama berkas CSV default adalah `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv`, dan
`DefenderQuarantine.csv`. Tabel yang berasal dari registry menggunakan prefiks `Registry-`, termasuk
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv`, dan `Registry-SystemInfo.csv`.
Key dan nilai hive umum ada di `Registry-Dump.csv`. Tabel Amcache adalah
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv`, dan `Registry-Amcache-Summary.csv`. Jump Lists menggunakan
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` dan
`JumpLists-CustomDestinations.csv`. Tidak ada subdirektori Amcache atau Jump Lists.
Tabel SRUM menggunakan nama berkas `SRUM-` langsung di bawah kategorinya, misalnya
`NetworkActivity/SRUM-NetworkConnections.csv`.
Tabel Pencatatan Akses Pengguna berada di `LateralMovement`, satu berkas per tabel, seperti
`UAL-Clients.csv` dan `UAL-SystemIdentity.csv`, beserta `UAL-logs.txt`; lihat
[Pencatatan Akses Pengguna (UAL)](../artifacts/ual.md#automatic-parsing).
Tabel browser web berada di `BrowserActivity`: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv`, dan `Browser-FormHistory.csv`, beserta `Browser-logs.txt`; lihat
[Browser web](../artifacts/browser.md#automatic-parsing).
Lihat [tabel penempatan SRUM selengkapnya](../artifacts/srum.md#categorized-auto-parse-output).

Log sesuai dengan nama artefak yang ditulis dengan huruf kapital, seperti `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`,
dan `Registry-UserAssist-logs.txt`. Berkas JSON/JSONL menggunakan stem yang sama, misalnya `MFT.jsonl`; pengenal
`Artifact` record adalah `mft` untuk query. Keluaran multitabel memiliki
`Registry-Amcache-logs.txt` atau `JumpLists-logs.txt` di samping tabel-tabelnya. Log SRUM bersama berada
di `SystemInformation`.


`manifest.json` mencatat `Hostname` asli yang ditemukan, identitas `Comp` keluaran/record yang unik,
provenans nama host, path bukti, `Category` artefak, command line, durasi, exit
code, path hasil, path log, timeline JSONL yang diminta dan error ekspor apa pun, apakah ekstraksi Defender diminta,
serta apakah setiap tugas
berhasil, dilewati, atau gagal.

Auto-parse tidak menulis path absolut apa pun, sehingga hasil dapat dibagikan tanpa memperlihatkan lokasi bukti,
hasil, atau Juntei di sistem analisis:

- Path bukti relatif terhadap direktori bukti. Terminal mencantumkan setiap komputer beserta
  direktorinya di dalam bukti (`HOST1: ./HOST1_F`), dan parser dijalankan di direktori bukti serta
  diberi path semacam itu. Kolom path sumber pada hasil (`SourceFile`, `SourcePath`, …), log,
  serta `EvidenceRoot` dan `WindowsDirectories` di manifest menunjukkan letak setiap berkas di dalam
  bukti (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Path hasil di manifest dan log (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, path
  keluaran di `Command`) serta `ExtractedFile` milik Defender relatif terhadap direktori keluaran
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` untuk timeline di sampingnya). `Command`
  dimulai dengan nama program (`juntei`); path buktinya relatif terhadap direktori bukti
  dan path keluarannya relatif terhadap direktori keluaran.
- `EvidenceDirectory` dan `OutputDirectory` adalah nama kedua direktori tersebut. `ManifestVersion` 2
  menandai manifest yang mencatat path hasil dari direktori keluaran; manifest yang lebih lama tidak memiliki
  `ManifestVersion` dan mencatatnya sebagaimana diberikan ke `-o`. Eksekusi ulang dengan `-C` membaca keduanya.
- Terminal menampilkan path keluaran atau timeline sebagaimana diberikan jika path tersebut relatif, dan dari
  direktori saat ini jika path tersebut absolut (`../results/`).

Setelah jumlah proses paralel, terminal menampilkan ukuran total berkas dari
sistem Windows yang ditemukan (`Total filesize: 26.1 MiB`).

Pesan progres menampilkan komputer dan artefak yang sedang aktif diikuti dengan perkiraan ukuran
input, misalnya `HOST1 / prefetch (~2.0 MiB)`. Sekelompok berkas menggunakan ukuran gabungannya; setiap
tugas MFT, LogFile, atau Jurnal USN menampilkan ukuran inputnya masing-masing. Perkiraan tidak menyertakan log transaksi
tambahan dan berkas pengayaan serta tidak memprediksi ukuran keluaran yang diurai.

`manifest.csv` menyediakan indeks tugas yang sama dalam bentuk tabel yang ramah spreadsheet, dengan satu baris per
tugas parser. Isinya mencakup `Comp`, `Hostname` asli, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error`, dan `Command`. `Purpose` bernilai `artifact` untuk hasil yang dikategorikan atau `json_timeline` untuk
tahap JSONL native tambahan yang digunakan bersama CSV. `OutputType` adalah format keluaran parser yang sebenarnya.
`Command` adalah array JSON berisi executable dan argumennya, dengan mempertahankan spasi dan tanda kutip.
Untuk SRUM, `Output` adalah direktori komputer, `OutputFiles` adalah array JSON berisi path keluaran
provider yang persis, dan `Category` menyebutkan lokasi log bersama (`SystemInformation`).
Berkas dikategorikan secara individual; `OutputFiles` juga muncul di manifest JSON.

Filter `Status` ke `skipped` atau `failed` untuk menemukan tugas yang perlu ditinjau; `Error` milik tugas yang dilewati
dapat kosong, dengan penjelasannya ada di berkas `Log`-nya.

Ringkasan terminal menyertakan jumlah total dan ukuran berkas reguler yang saat ini berada di bawah
direktori keluaran, termasuk kedua manifest, log parser, dan berkas hasil ekstraksi. Symbolic link tidak
diikuti. Ukuran diskalakan secara otomatis menggunakan pemformat yang sama dengan `Total filesize`, dengan
satuan seperti `KiB`, `MiB`, dan `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Tabel Amcache dan SRUM menggunakan `-Source1`, `-Source2`, dst. ketika beberapa database sumber
dikumpulkan. Log parser bersama mencatat path dari asli ke akhir, dan record JSON/JSONL
mempertahankan provenans sumber. Tugas CSV Amcache/Jump Lists mencantumkan path tabel persisnya di
`OutputFiles` manifest; `Output` miliknya adalah direktori kategori investigasi. Ekspor JSON dan JSONL
Amcache dan Jump Lists sudah berupa berkas tunggal bernama `Registry-Amcache.json[l]` dan
`JumpLists.json[l]`. Timeline gabungan `-j` mempertahankan isi record dan nilai `Artifact` yang sama.
`--clobber` menggunakan manifest sebelumnya untuk menghapus keluaran yang tercatat, termasuk nama-nama sebelumnya,
dengan tetap mempertahankan berkas kategori yang tidak terkait. Koleksi volume tunggal yang normal menulis record `$MFT` dan Jurnal USN langsung ke
`FileSystem/MFT.csv` dan `FileSystem/UsnJournal.csv`. Volume tambahan yang dikumpulkan menggunakan berkas
saudara bernomor seperti `FileSystem/MFT-2.csv` dan `FileSystem/MFT-2-logs.txt`.

## Ingesti JSON dan SIEM { #json-and-siem-ingestion }

Setiap record JSON atau JSONL menerima dua field tingkat teratas:

- `Comp`: sistem sumber.
- `Artifact`: parser Juntei yang menghasilkan record, seperti `mft` atau `prefetch-timeline`.

`-j, --json-timeline <FILE>` membuat satu berkas JSONL tambahan, terlepas dari format hasil
individual. Semua keluaran yang dikategorikan beserta log-nya tetap dipertahankan:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Nama berkas digunakan sebagaimana diberikan; path relatif bersifat relatif terhadap direktori kerja saat ini.
Berkas dapat berada di root keluaran, subdirektori keluaran lain yang tidak dicadangkan, atau di luar direktori
keluaran. Bukti, manifest, hasil/log per komputer, dan direktori parser sementara tidak dapat
digunakan sebagai tujuan. Berkas yang sudah ada memerlukan `-C`; ekspor yang gagal membiarkannya utuh.

Dalam **mode CSV**, Juntei menjalankan tahap JSONL native tambahan sehingga berkas SIEM mempertahankan
tipe numerik dan boolean, data bersarang, field mentah, dan metadata yang dapat dihilangkan oleh tabel CSV.
Tugas tambahan ini berbagi batas konkurensi `-T` yang sama, tidak mengekstrak konten karantina lagi, dan
memiliki log sidecar `*-json-timeline-logs.txt` tersendiri. Kedua manifest membedakan tujuan dan
format keluarannya. Berkas JSONL sementara dihapus setelah eksekusi berhasil; berkas tersebut dipertahankan jika sebuah
parser atau ekspor gagal. Perhitungkan waktu pemrosesan tambahan dan ruang disk sementara.

Dalam **mode JSON**, Juntei mengalirkan hasil yang ada ke JSONL ringkas tanpa mengurai ulang
bukti. Laporan Amcache diperluas menjadi record summary, finding, file, dan device individual yang sama
seperti JSONL native. Dalam **mode JSONL**, record beranotasi yang ada digabungkan secara langsung.
Semua mode menggunakan struktur record SIEM yang sama. Dataset mempertahankan timestamp dan field artefak
native; dataset tidak menciptakan timestamp peristiwa bersama atau mengurutkan artefak yang heterogen secara global.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` mencatat path `JSON-Timeline` yang diminta dan `JSON-TimelineError` apa pun. Record
tugasnya, serta `manifest.csv`, tetap menunjuk ke keluaran artefak yang dipertahankan. Tugas CSV khusus timeline
yang berhasil merujuk ke berkas JSONL akhir alih-alih berkas sementara yang telah dihapus. Jika ada parser yang gagal,
eksekusi berakhir dengan status tidak berhasil dan timeline hanya berisi hasil yang berhasil; periksa manifest
dan log kategori sebelum menganggapnya lengkap. Ringkasan jumlah/ukuran berkas mencakup berkas di dalam
direktori keluaran; timeline eksternal dilaporkan secara terpisah sebagai `JSONL timeline: <FILE>`.

## Performa { #performance }

Tugas parser merupakan proses terpisah, sehingga artefak yang berat bagi CPU dapat berjalan bersamaan dan kerusakan pada
satu parser tidak dapat menghentikan seluruh eksekusi. Secara default, Juntei menggunakan jumlah CPU logis. Batasi
konkurensi ketika bukti berada di disk yang lambat atau sistem memiliki sedikit memori:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Penggunaan { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Gunakan direktori keluaran baru jika memungkinkan. `-C` hanya menghapus atau menimpa target
yang dihasilkan secara persis untuk eksekusi ini, sehingga artefak yang kini tidak ada tidak dapat meninggalkan hasil usang. Opsi ini tidak menghapus
berkas yang tidak terkait atau seluruh pohon keluaran. Opsi ini juga menghapus direktori
`SecurityProducts/DefenderQuarantined/` setiap komputer, dan direktori `defender-quarantined/`
yang dicadangkan dari tata letak datar sebelumnya, sebelum penguraian, baik eksekusi baru menggunakan `-x` maupun tidak.

Gunakan direktori keluaran baru saat beralih dari tata letak datar yang lebih lama. Berkas CSV/JSON sebelumnya
tidak dipindahkan atau dihapus secara otomatis.

!!! danger "Perlakukan konten Defender hasil ekstraksi sebagai malware"
    `-x, --extract-defender-quarantined` menulis konten yang dikarantina ke disk di bawah direktori
    `SecurityProducts/DefenderQuarantined/` setiap komputer. Stream biner yang dipulihkan berakhiran `.quarantined`, dan Juntei
    menghapus izin eksekusinya. Setiap deskriptor keamanan biner juga mendapat laporan `.security_descriptor.txt`
    yang dapat dibaca, berisi kepemilikan, izin, dan SDDL. Jangan pernah mengeksekusi payload yang dipulihkan. Gunakan sistem
    analisis yang terisolasi dan perkirakan bahwa perangkat lunak antivirus atau EDR akan memberi peringatan atau menghapus konten yang dipulihkan.

Lihat [membaca deskriptor keamanan yang dipulihkan](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
untuk field laporan dan keterbatasan forensiknya.

Auto-parse juga membaca koneksi jarak jauh milik pengguna (RDP, PuTTY, WinSCP, dan mapped drive).
