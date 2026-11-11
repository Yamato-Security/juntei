# Persistence Hunter { #persistence-hunter }

Persistensi memungkinkan sebuah program berjalan kembali setelah reboot, sign-in, pemicu terjadwal, atau peristiwa
lain. Penyerang dapat menggunakan konfigurasi Windows yang normal untuk tujuan ini. Banyak program yang sah
menggunakan mekanisme yang persis sama, sehingga menemukan entri persistensi merupakan titik awal analisis.

`juntei persistence-hunter` adalah port Rust native dari
[PyrsistenceSniper oleh Hexastrike](https://github.com/hexastrike/PyrsistenceSniper), dengan 131 pemeriksaan:
katalog 130 pemeriksaan PyrsistenceSniper ditambah pemeriksaan COM Start Menu native.

Untuk penjelasan setiap mekanisme, lokasi buktinya, dan cara menginvestigasi sebuah temuan,
lihat [panduan persistensi 131 pemeriksaan](../persistence-hunter/index.md). Halaman
[ucapan terima kasih persistensi](../persistence-hunter/acknowledgements.md) menjelaskan proyek Python dan
PowerShell di balik katalog tersebut.

## Penggunaan { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Pemindaian pertama { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

Bukti dapat berupa volume Windows yang di-mount, hasil pengumpulan Velociraptor, atau direktori yang berisi
beberapa komputer yang dikumpulkan. Komputer-komputer tetap dipisahkan selama analisis, dan setiap hasil menyertakan
`Comp`. Escaping nama berkas hasil pengumpulan didekode saat menyelesaikan path Windows.

Untuk satu hive:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Satu hive tidak dapat memenuhi pemeriksaan yang memerlukan hive lain atau bukti sistem berkas. Baca laporan
coverage sebelum menganggap hasil yang kosong bermakna.

Tampilkan daftar check ID dan pemetaan ATT&CK-nya:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Gunakan check ID yang dicetak oleh `--list-checks`, atau ID ATT&CK seperti `T1547.001`.
ID induk seperti `T1547` memilih sub-tekniknya. `--technique` adalah alias untuk `--check`.
Setiap pemeriksaan menyertakan teknik, deskripsi, dan tautan referensinya di dalam temuan.

## Pemindaian paralel { #parallel-scans }

Perintah ini menggunakan worker pool bersama, dengan default sejumlah CPU logis. Atur
`-T, --threads <N>` untuk memilih jumlah worker, atau `-T 1` untuk pemindaian single-thread:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

Penemuan bukti, pemeriksaan independen, metadata payload, dan sumber timeline MFT/EVTX berjalan secara paralel. Hive
registry yang dimuat milik sebuah host dibagikan dalam mode baca saja, dan metadata cache untuk payload yang sama diinisialisasi sekali.
Host dimuat satu per satu untuk menghindari penggandaan memori hive di seluruh hasil pengumpulan. Temuan,
coverage, pemilihan timestamp, dan urutan peringatan tetap sama pada jumlah thread yang berbeda.

`auto-parse` sudah menjalankan proses parser secara paralel, sehingga memberikan satu worker kepada setiap proses child
`persistence-hunter`. Opsi `-T` miliknya sendiri mengendalikan jumlah proses parser yang berjalan bersamaan.

## Bukti yang perlu dikumpulkan { #evidence-to-collect }

| Sumber | Contoh pemeriksaan |
|---|---|
| `SYSTEM` dan log transaksi di sebelahnya | Layanan, perintah pemulihan layanan, paket LSA, pengaturan boot, penyedia autentikasi |
| `SOFTWARE` dan log transaksi di sebelahnya | Key Run, Winlogon, IFEO, registrasi COM, TaskCache, pengaturan Office |
| `NTUSER.DAT` dan `UsrClass.dat` setiap pengguna | Autostart pengguna, variabel lingkungan, override COM, pengaturan Office |
| `SAM` dan `SOFTWARE` | Anomali RID, akun tersembunyi, keanggotaan Administrators lokal yang tidak biasa |
| `Windows/System32/Tasks` | Perintah tugas terjadwal dan handler COM; perbandingan dengan TaskCache |
| Berkas `.library-ms` Start Menu per pengguna dan folder yang dinamai dengan CLSID | Kandidat pemicu aktivasi COM, digabungkan dengan registrasi kelas pengguna/mesin yang cocok |
| Folder Startup, profil PowerShell, template Office, pengaturan Terminal | Berkas dan perintah yang dijalankan oleh pemicu pengguna atau sistem |
| WMI `OBJECTS.DATA` | Instance event consumer dan binding filter-ke-consumer |
| BITS `qmgr.db` atau `qmgr*.dat` | Perintah notifikasi yang dipulihkan dan SID pemilik |
| `$MFT` dan berkas EVTX yang relevan | Korelasi timestamp independen |
| Executable, DLL, dan berkas katalog yang dirujuk | Hash SHA256 dan metadata deskriptif PE/sertifikat |

Log transaksi registry di-replay bila tersedia. `--no-logs` membaca hive sebagaimana dikumpulkan;
hive dirty yang tidak di-replay dapat kehilangan perubahan terbaru dan dilaporkan di coverage. Control
set saat ini digunakan untuk pemeriksaan sistem aktif. Pemeriksaan backdoor DSRM juga memeriksa
`ControlSet001`, `ControlSet002`, dan `CurrentControlSet` bila ada; periksa path yang dilaporkannya
terhadap `SYSTEM\\Select` sebelum menyebut pengaturan tersebut sebagai aktif. Control set historis lainnya
dapat diperiksa dengan perintah parser registry individual.

Pembacaan hive berbagi anggaran input sebesar 1 GiB per komputer. Hive mesin dicoba sebelum hive pengguna;
input atau log transaksi yang tidak muat disebutkan dalam coverage parsial. Hive lain yang dapat dibaca dan
pemeriksaan berbasis berkas tetap dilanjutkan. Hanya lokasi hive kanonis atau ekspor yang diberikan secara eksplisit yang menetapkan
identitas hive; salinan di dalam direktori sementara milik pengguna tidak dapat menggantikan hive asli pengguna tersebut.
Drive sistem asli yang tercatat di SOFTWARE dipetakan ke volume sistem yang ditangkap, sehingga me-mount
volume `C:` asli di bawah label drive bukti yang berbeda tidak membuang metadata payload.

`auto-parse` juga menjalankan hunter dan menyimpan `persistence/PersistenceHunter.csv` (atau `.json`/`.jsonl`)
di dalam direktori setiap komputer, dengan `PersistenceHunter-logs.txt` di sebelahnya. Temuannya juga masuk ke
timeline SIEM opsional `-j <FILE>`. Analisis otomatis menggunakan `--min-severity info` untuk mempertahankan
inventaris konfigurasi yang lengkap, termasuk entri yang cocok dengan aturan allow.

## Registrasi COM dan pemicu aktivasi { #com-registrations-and-activation-triggers }

Pemeriksaan `scheduled_task_files` memilih principal setiap tindakan menggunakan `Actions Context`
dari XML tugas dan `Principal id` yang cocok. Principal berupa grup dipertahankan sebagai grup;
principal tersebut tidak dapat menetapkan registrasi kelas milik pengguna individual mana yang akan digunakan.

Kandidat COM handler diselesaikan secara terpisah dalam tampilan kelas native dan 32-bit. Hanya
`UsrClass.dat`/`NTUSER.DAT` yang ditangkap milik pengguna yang cocok dan `SOFTWARE` milik mesin yang dipertimbangkan.
Principal SID digabungkan melalui path `ProfileList` yang ditangkap. Nama akun, termasuk
`HOST\user` dan `DOMAIN\user`, lebih mengutamakan profil `user.DOMAIN` yang cocok daripada profil
`user` biasa. Alias host lokal dan `.\user` menggunakan nama host yang ditangkap untuk pencocokan sufiks;
sufiks yang persis sama didahulukan atas alias host lainnya. Profil biasa hanya dipertimbangkan
bila tidak ada profil berkualifikasi yang cocok. Setiap kecocokan seperti itu secara eksplisit diberi label
**folder-name heuristic** di `Description`; hal ini tidak membuktikan kepemilikan akun atau keanggotaan domain. Kecocokan yang ambigu
tetap tidak terselesaikan dan dilaporkan baik di deskripsi temuan maupun di coverage; ambiguitas
di antara profil berkualifikasi tidak memicu fallback ke profil biasa. Hal ini tidak bergantung
pada `Volatile Environment`, yang biasanya hanya ada di memori dan tidak disimpan bersama hive registry. CLSID pengguna
yang duplikat membayangi child dari CLSID mesin. Principal grup menggunakan kandidat COM mesin dan mempertahankan
keterbatasan tersebut di deskripsi temuan; principal grup yang normal tidak dengan sendirinya membuat
coverage menjadi parsial atau menghabiskan entri ringkasan error.

Resolver mengikuti `InprocServer32` dan `LocalServer32`, menggunakan path `CodeBase` terkelola
(termasuk subkey versi), dan menggabungkan `LocalService` milik AppID dengan `ImagePath` layanan SYSTEM
yang dikumpulkan. Asosiasi nama executable AppID dipertahankan, tetapi basename tidak menetapkan
path lengkap sebuah executable. URL CodeBase `file:` lokal didekode; URL jarak jauh dicatat
tanpa diakses. Registrasi yang hilang mempertahankan CLSID dengan
`ResolutionStatus: unresolved_com_class`, field path/hash kosong, dan `Exists` yang tidak diketahui.

Pemeriksaan `start_menu_com` membaca URL XML `.library-ms` Start Menu per pengguna dan mengumpulkan
folder yang dinamai dengan CLSID, termasuk direktori kosong. Pemeriksaan ini melaporkan path pemicu, CLSID, kandidat
server, dan asal-usul registry. Akuisisi yang hanya berbasis berkas tidak dapat mempertahankan direktori kosong,
dan hive pengguna yang hilang mencegah pencarian registrasi per pengguna secara lengkap. Symbolic link
tidak diikuti. Ini adalah kandidat konfigurasi: arsitektur, elevasi, UAC, kebijakan, dan
konteks aktivasi dapat memengaruhi apakah sebuah registrasi benar-benar digunakan. Dukung eksekusi
dengan bukti proses, pemuatan modul, atau bukti independen lainnya.

## Klasifikasi dan profil { #classification-and-profiles }

Klasifikasi minimum default adalah `medium`. Untuk memeriksa inventaris lengkap, termasuk
entri yang cocok dengan allowlist bawaan:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Gunakan semua tingkat keparahan saat meninjau hasil pengumpulan yang tidak lengkap"
    Aturan allow yang luas dapat cocok sebagian dengan perintah penyerang bahkan ketika penanda tangannya tidak ada
    atau berbeda. Upstream mengklasifikasikan kecocokan parsial sebagai `LOW`, yang disembunyikan oleh ambang default
    `medium` pada perintah manual. Hal ini dapat memengaruhi tugas terjadwal berbahaya di bawah direktori Windows.
    Gunakan `--min-severity info` untuk investigasi yang lengkap dan tinjau `PolicyReason`, `SignerStatus`,
    perintah, dan bukti sumber secara bersamaan. `auto-parse` sudah mempertahankan entri-entri ini.

| Klasifikasi | Makna |
|---|---|
| `HIGH` | Aturan block yang dikonfigurasi cocok sepenuhnya |
| `MEDIUM` | Tidak ada aturan allow atau block yang cocok |
| `LOW` | Kondisi wajib dari aturan allow cocok, tetapi metadata penanda tangan tidak cocok atau tidak tersedia |
| `INFO` | Aturan allow cocok sepenuhnya |

Ini adalah klasifikasi kebijakan, bukan probabilitas kompromi yang terukur. Entri `MEDIUM` default
bukan bukti adanya malware. `INFO` tidak menetapkan bahwa sebuah berkas memiliki signature yang valid atau bahwa
penyerang tidak menyalahgunakannya. Kecocokan allow parsial berada di bawah ambang default, jadi sertakan
`--min-severity low` atau `info` saat meninjau hasil pengumpulan yang minim atau menyesuaikan allowlist.

Berikan profil YAML dengan `--profile profile.yaml`. Profil ini **menggantikan** profil bawaan. Aturan global
dan per pemeriksaan di dalam berkas yang dipilih digabungkan; aturan block yang cocok sepenuhnya akan menang.
Sebagai contoh:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Kondisi aturan yang didukung adalah `path_matches`, `value_matches`, `hash` (64 digit heksadesimal SHA256),
`not_lolbin`, dan `signer` (substring yang tidak peka huruf besar/kecil). Semua kondisi wajib yang dinyatakan harus
cocok. Metadata penanda tangan adalah kondisi lunak yang dijelaskan di atas. Regex menggunakan sintaks regular expression
milik Rust: lookaround dan backreference tidak didukung. Check ID yang tidak dikenal, ekspresi yang tidak valid,
dan field profil yang tidak dikenal menghasilkan error alih-alih melemahkan kebijakan secara diam-diam.

## Field keluaran { #output-fields }

CSV adalah format berkas default. Gunakan `-t json` untuk objek JSON yang mudah dibaca atau `-t jsonl` untuk keluaran
ringkas dengan satu objek per baris. Tanpa `-o`, `-t` yang eksplisit hanya menulis data ke stdout; peringatan coverage
dikirim ke stderr. Tanpa kedua opsi tersebut, Juntei mencetak ringkasan dan hingga 30 kandidat.

| Field | Cara menginterpretasikannya |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Komputer dan mekanisme persistensi mana yang dideskripsikan oleh baris tersebut |
| `Path`, `Value` | Lokasi registry, berkas, tugas, atau record yang dipulihkan beserta konten yang dikonfigurasi |
| `Description`, `References` | Mengapa mekanisme tersebut penting dan di mana dapat membaca lebih lanjut |
| `AccessGained` | Cakupan umum mekanisme tersebut (`USER` atau `SYSTEM`; BITS juga dapat mengidentifikasi `LOCAL_SERVICE`/`NETWORK_SERVICE`); bukan bukti bahwa perintah tertentu memperoleh hak akses tersebut |
| `Severity`, `PolicyReason` | Klasifikasi profil, penjelasan kebijakan yang cocok, dan setiap kualifikasi kecocokan parsial atau penanda tangan yang belum diverifikasi |
| `SourceFile`, `RecordStatus` | Sumber hasil pengumpulan yang persis, path-nya yang diawali dengan input sebagaimana diberikan (`./HOST/...` dari auto-parse), dan konteks alokasi/replay/pemulihan; telusuri kembali ke bukti asli |
| `KeyLastWrite` | Timestamp **key** registry. Timestamp ini tidak memberi tanggal pada nilai individual |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Waktu perubahan yang dikorelasikan secara independen, bukti sumber, dan apakah korelasi berhasil |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Path payload dan hasil resolusi offline; bukti yang tidak dikumpulkan bernilai tidak diketahui, bukan `false` |
| `SHA256` | SHA256 dari byte payload yang dikumpulkan, bila dapat dibaca |
| `Signer`, `SignerStatus` | Metadata deskriptif sertifikat/katalog yang disematkan dan ketersediaannya; kepercayaan sertifikat dan validitas signature tidak diverifikasi |
| `IsLolbin`, `Launcher` | Apakah launcher atau payload yang dikonfigurasi ada di daftar LOLBAS bawaan, dan launcher yang digunakan untuk payload yang diproksikan |
| `IsBuiltin`, `IsInOS-Directory` | Konteks untuk perintah Windows umum dan direktori sistem; tidak ada field yang menetapkan legitimasi |

Perintah `rundll32.exe` dapat menyebutkan payload DLL. Juntei mempertahankan launcher sambil menyelesaikan dan
menghitung hash payload. Referensi di luar komputer yang dikumpulkan, drive yang hilang, kecocokan
tidak peka huruf besar/kecil yang ambigu, atau variabel lingkungan yang tidak terselesaikan tidak diganti dengan berkas dari
mesin analis.

## Timestamp dan keterbatasan { #timestamps-and-limitations }

Hunter tidak menggunakan waktu modifikasi berkas salinan di VM analis sebagai timestamp serangan.
Bila tersedia, hunter mengorelasikan penulisan registry dengan peristiwa Sysmon 13, instalasi tugas dengan
Security 4698 atau Task Scheduler 106, instalasi layanan dengan System 7045, dan konfigurasi berbasis berkas
dengan timestamp `$MFT` asli. Asal-usulnya muncul di `ChangeSource`.
Sebuah peristiwa dapat memberi tanggal pada perubahan konfigurasi tanpa membuktikan bahwa payload yang dikonfigurasi berjalan.
Peristiwa System lokal 6011 dapat membuktikan nama komputer sebelumnya. Hunter mengikuti rantai penggantian nama
terverifikasi yang berjangkar pada komputer yang dikumpulkan, mempertahankan nilai `Comp` saat ini sambil mencatat
komputer peristiwa asli dan bukti record penggantian nama di `ChangeSource`. Nama asing yang tidak terbukti,
log yang diarsipkan, log yang diteruskan, dan instalasi Windows lain tidak diterima melalui aturan ini.

ComputerName/ActiveComputerName, Hostname/NV Hostname TCP/IP, dan hostname/FQDN client-info yang ditangkap
menyediakan alias host, mencakup perbedaan DNS/NetBIOS dan penggantian nama yang tertunda. Tanpa identitas yang ditangkap,
satu nama di log utama dapat mendukung pemberian tanggal dengan asal-usul identitas yang belum diverifikasi secara eksplisit
dan peringatan coverage. Beberapa nama yang belum diverifikasi tidak diatribusikan ke host. Nama Computer
yang ditolak dilaporkan sebagai keterbatasan coverage korelasi; channel-nya tetap ditandai sebagai telah dikumpulkan.

Gunakan `--mft <FILE>` untuk memilih MFT yang dikumpulkan secara eksternal untuk satu komputer. Gunakan `--no-timeline`
untuk melewati korelasi timestamp. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH`, dan `REJECTED`
menjelaskan `LastChange` yang kosong; field yang kosong bukan bukti bahwa entri tersebut tidak berbahaya.

Pemulihan WMI dan BITS adalah carving terbatas, bukan parser transaksi database yang lengkap.
String atau binding yang dipulihkan dapat mencakup data lama/terhapus; record-record ini harus didukung oleh bukti lain.
Skema kelas WMI saja tidak dihitung sebagai subscription. Entri TaskCache tanpa berkas XML tugas yang dikumpulkan
adalah ketidaksesuaian yang perlu ditinjau; hasil pengumpulan triase mungkin saja tidak menyertakan berkas tersebut.
Metadata katalog dan PE belum diverifikasi dan mungkin tidak ada dalam hasil pengumpulan. Pemeriksaan penggantian
berkas aksesibilitas bergantung pada metadata signature referensi yang sesuai; bukti katalog yang hilang
tidak boleh menyebabkan setiap executable aksesibilitas ditandai.

Coverage dicatat untuk setiap pemeriksaan, termasuk pemeriksaan tanpa temuan. `complete` berarti
sumber hasil pengumpulan yang berlaku telah dibaca; hal ini tidak membuktikan bahwa hasil pengumpulan lengkap atau bahwa
komputer tidak memiliki persistensi lain. `partial`, `not_collected`, `failed`, dan `disabled` menjelaskan
batasannya. Simpan `--coverage Coverage.csv` dan log parser bersama temuan Anda.

Port ini menggunakan format CSV/JSON/JSONL milik Juntei. Port ini tidak memanggil program Python upstream, tidak melakukan query ke
layanan pengayaan online, dan tidak mereproduksi UI HTML/XLSX upstream.

## Detail lebih lanjut { #further-details }

- [Implementasi dan profil PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
- [Teknik persistensi MITRE ATT&CK](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Inventaris autostart registry](../artifacts/persistence.md)
- [Layanan](../artifacts/services.md) dan [TaskCache](../artifacts/taskcache.md)
