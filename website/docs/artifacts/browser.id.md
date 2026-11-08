# Browser web { #web-browsers }

## Apa itu database browser { #what-the-browser-databases-are }

Browser menyimpan riwayat, unduhan, cookie, dan entri formulirnya dalam database SQLite di profil setiap
pengguna. Browser berbasis Chromium (Chrome, Edge, Brave, Opera, Vivaldi) dan browser berbasis Firefox
masing-masing menggunakan berkas dan tabelnya sendiri, tetapi mencatat jenis aktivitas yang sama:

- **Riwayat**: halaman yang dikunjungi, setiap kunjungan beserta waktunya, bagaimana browser sampai ke halaman
  itu (diketik, sebuah tautan, sebuah redirect, sebuah bookmark) dan, di Chromium, berapa lama halaman itu
  berada di depan.
- **Unduhan**: URL asal sebuah berkas (dan redirect di sepanjang jalannya), tempat berkas itu disimpan,
  ukurannya, kapan unduhan dimulai dan berakhir, dan apakah unduhan itu selesai.
- **Cookie**: situs yang menetapkannya, kapan cookie ditetapkan, terakhir dikirim, dan kedaluwarsa.
- **Riwayat formulir**: nilai yang diketik di formulir web dan kotak pencarian, seberapa sering, serta kapan
  pertama dan terakhir kali.

Aktivitas browser sering kali merupakan catatan paling langsung tentang apa yang dilakukan pengguna: halaman
phishing yang diikutinya, alat yang diunduhnya, webmail yang digunakannya, kata kunci pencarian yang diketiknya.

<span class="evidence-strength">Kesimpulan terbaik:</span> profil browser mencatat kunjungan ke URL tersebut
pada waktu yang tercatat, atau menyimpan unduhan ke path yang tercatat; database tidak menyatakan siapa yang
duduk di depan keyboard.

## Lokasi dan pengumpulan { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` adalah `Default`, `Profile 1`, `Profile 2`, dan seterusnya. Chromium memindahkan `Cookies` ke
`Network` pada versi 96; versi yang lebih lama menyimpannya di samping `History`. Firefox hingga versi 25
menyimpan unduhannya di `downloads.sqlite`; versi selanjutnya menyimpannya di `places.sqlite`.

Kumpulkan setiap berkas profil, dan setiap database beserta berkas di sampingnya yang namanya diawali nama
database tersebut: `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Write-ahead log"
    Firefox menyimpan database-nya dalam mode write-ahead log: perubahan terbaru hanya ada di berkas `-wal`
    sampai browser menyalinnya ke dalam database (checkpoint). Juntei membaca setiap database beserta berkas
    `-wal`-nya, seperti yang dilakukan browser. Tanpa berkas itu, kunjungan, cookie, dan entri formulir terbaru
    tidak muncul, dan entri yang dihapus di dalam log masih terbaca sebagai entri aktif.

## Entri terhapus { #deleted-entries }

Ketika browser menghapus riwayat, baris-barisnya keluar dari tabel, tetapi record-nya dapat tetap tersisa di
ruang kosong database, di freelist-nya, atau di versi lama halaman-halaman database: di berkas `-wal`, atau di
berkas database itu sendiri, di bawah `-wal` yang menggantikannya. Juntei memulihkan kunjungan, halaman, unduhan,
dan entri formulir yang terhapus dari sana, sebagai baris yang `Status`-nya `Deleted`:

- `Confidence` menyatakan seberapa yakin kecocokan record dengan tabelnya: `High` untuk record utuh yang setiap
  nilainya bertipe sesuai kolomnya, `Medium` atau `Low` untuk record yang rusak atau record yang cocok dengan
  beberapa tabel.
- `RecoveredFrom` menyatakan di mana record itu berada: area halaman (sebuah cell, sebuah freeblock, ruang
  unallocated, sebuah halaman bebas), halaman mana dan versi mana dari halaman itu, serta offset record di dalam
  halaman. Frame WAL dihitung mulai dari 0.
- `LostColumns` mencantumkan kolom yang tidak lagi dimiliki oleh record yang rusak: field-nya kosong.

Browser menghapus dengan `secure_delete` milik SQLite: ruang yang dibebaskan di dalam database diisi nol. Yang
dapat dipulihkan kemudian sebagian besar berada di versi halaman yang lebih lama: yang masih disimpan oleh berkas
`-wal`, dan salinan halaman di berkas database itu sendiri yang telah digantikan oleh `-wal`, sampai checkpoint
berikutnya. Salin berkas `-wal` bersama database-nya, dan lakukan sebelum browser dijalankan lagi.

## Peringatan bagi analis { #analyst-cautions }

- Waktu dalam UTC. Chromium dan Firefox mencatatnya hingga mikrodetik; riwayat formulir Chromium hingga detik.
- Kunjungan tidak berarti pengguna mengetik atau mengeklik: redirect, frame, dan halaman yang dibuka oleh program
  lain juga merupakan kunjungan. Baca `Transition`, `Typed`, dan `Hidden`.
- Chromium mengenkripsi nilai cookie: `Value` kosong untuk cookie tersebut. Juntei tidak melakukan dekripsi.
- Baris riwayat formulir `Removed` berasal dari `moz_deleted_formhistory` milik Firefox, yang diisi oleh Firefox
  untuk Android untuk Sync. Firefox desktop biasanya membiarkannya kosong.
- Entri terhapus dengan confidence `Low` mungkin merupakan fragmen dari record lain: periksa terhadap baris aktif
  dan bukti lainnya.
- Tidak adanya baris terhapus tidak berarti tidak ada yang dihapus: ruang yang diisi nol dan checkpoint tidak
  menyisakan apa pun untuk dipulihkan. Cookie yang terhapus belum dipulihkan.
- Isi database menentukan keluarga browser. Path profil yang sesuai memperinci `Browser` menjadi Chrome atau Firefox; path yang bertentangan tidak mengubah keluarga. Selain itu labelnya `Chromium-based`, `Firefox-based`, atau `Safari` (`Unknown` jika tidak diketahui). `User` dan `Profile` berasal dari path dan dapat hilang saat file disalin.
- Rollback journal (`-journal`) tidak dibaca. `WebCacheV01.dat` milik Internet Explorer dan Edge lama
  (legacy) dibaca oleh `parse-ese webcache`: lihat [Internet Explorer dan Edge lama](webcache.md).

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` mencari berkas berdasarkan nama dan mengenali masing-masing dari tabelnya: berkas `History` milik program
lain dilewati. `-f` membaca sebuah database apa pun namanya.

## Tabel keluaran { #output-tables }

CSV membuat empat berkas, masing-masing hanya berisi header-nya jika tidak ada baris. Baris JSON dan JSONL
membawa `Table` (`History`, `Downloads`, `Cookies`, atau `FormHistory`) setelah `SourceFile`.

| Berkas CSV | Isi |
|---|---|
| `Browser-History.csv` | Kunjungan, aktif dan terhapus; halaman terhapus yang URL-nya tidak dimiliki halaman aktif mana pun. |
| `Browser-Downloads.csv` | Unduhan, aktif dan terhapus. |
| `Browser-Cookies.csv` | Cookie. |
| `Browser-FormHistory.csv` | Entri formulir, aktif dan terhapus; record penghapusan milik Firefox. |

## Field { #fields }

Setiap tabel diawali dengan field berikut:

| Field | Interpretasi |
|---|---|
| `SourceFile` | Database tempat baris tersebut dibaca. |
| `User` | Pengguna Windows yang profilnya memuat path tersebut (folder setelah `Users`). |
| `Browser` | Keluarga browser dari isi, diperinci oleh path profil yang sesuai. |
| `Profile` | Profil browser: folder database. |
| `Status` | `Live`, `Deleted` (dipulihkan), atau `Removed` (record penghapusan Firefox). |

### Riwayat { #history }

| Field | Interpretasi |
|---|---|
| `RecordType` | `Visit`, atau `Page` untuk halaman terhapus yang kunjungannya mungkin juga sudah hilang. |
| `VisitTime` | Kapan kunjungan dimulai (UTC). |
| `URL`, `Title` | Halaman dan judulnya, sebagaimana terakhir terlihat. |
| `Transition` | Bagaimana browser sampai ke halaman tersebut. Chromium: tipe inti lalu kualifikasinya (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … dengan `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Berapa detik halaman berada di depan (Chromium; 0 jika tidak diukur). |
| `VisitCount` | Jumlah seluruh kunjungan ke halaman tersebut. |
| `Typed`, `TypedCount` | Apakah URL pernah diketik, dan berapa kali (Chromium). |
| `Hidden` | Disembunyikan dari daftar riwayat: subframe, sumber redirect, error. |
| `Frecency` | Peringkat halaman oleh Firefox untuk saran di bilah alamat. |
| `VisitID`, `FromVisitID` | Row id kunjungan, dan row id kunjungan asalnya (halaman perujuk, sumber redirect). Row id kunjungan kosong jika record yang dipulihkan kehilangannya. |
| `LastVisitTime` | Kunjungan terakhir ke sebuah halaman terhapus (UTC). |
| `PageSource` | Asal URL dan judul sebuah kunjungan terhapus: `Live` (halaman aktif), `Recovered` (halaman terhapus), atau `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Bagaimana sebuah entri terhapus dipulihkan; lihat [Entri terhapus](#deleted-entries). |

### Unduhan { #downloads }

| Field | Interpretasi |
|---|---|
| `StartTime`, `EndTime` | Kapan unduhan dimulai dan berakhir (UTC). |
| `URL`, `RedirectChain` | Asal berkas, dan setiap URL di sepanjang jalannya (Chromium; ` -> ` di antaranya dalam CSV). |
| `TargetPath`, `CurrentPath` | Tempat berkas disimpan (URI `file://` di Firefox), dan tempat berkas ditulis selama diunduh. |
| `ReceivedBytes`, `TotalBytes` | Byte yang diterima, dan ukuran berkas. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked`, atau `Dirty` (diblokir sebagai malware). |
| `DangerType` | Penilaian Chromium atas berkas: 0 tidak berbahaya, 1 tipe berkas berbahaya, 2 URL berbahaya, 3 konten berbahaya, 4 mungkin berbahaya, 5 tidak umum, 6 disimpan oleh pengguna, 7 host berbahaya, 8 berpotensi tidak diinginkan, … |
| `InterruptReason` | Mengapa unduhan yang terputus berhenti (Chromium; 0 jika tidak ada). |
| `Referrer`, `TabURL` | Halaman perujuk, dan halaman di tab yang memulai unduhan. |
| `MimeType`, `Opened` | Tipe berkas sebagaimana dikirim server, dan apakah berkas dibuka dari browser (Chromium). |
| `FileDeletedByUser` | Apakah pengguna menghapus berkas yang diunduh dari disk melalui daftar unduhan browser (Firefox). |
| `DownloadID` | Row id unduhan. Kosong jika record yang dipulihkan kehilangannya. |

### Cookie { #cookies }

| Field | Interpretasi |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Kapan cookie ditetapkan, terakhir dikirim, dan kedaluwarsa (UTC; `Expires` kosong untuk cookie sesi). |
| `Host`, `Name`, `Path` | Situs tujuan cookie, nama, dan path-nya. |
| `Value` | Nilai sebagaimana tersimpan: kosong jika dienkripsi oleh Chromium. |
| `Secure`, `HttpOnly`, `Persistent` | Hanya dikirim melalui HTTPS, disembunyikan dari skrip, dipertahankan antarsesi (Chromium). |
| `GoogleAnalytics` | Field hasil dekode dari cookie Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`): waktu kunjungan, sesi, serta sumber kampanye dan kata kunci pencarian. |
| `RowID` | Row id cookie. |

### Riwayat formulir { #form-history }

| Field | Interpretasi |
|---|---|
| `FirstUsed`, `LastUsed` | Kapan nilai tersebut pertama dan terakhir kali dimasukkan (UTC). |
| `RemovedTime` | Kapan entri dihapus menurut record penghapusan Firefox (UTC). |
| `FieldName`, `Value` | Field formulir (`email`, `q`, bilah pencarian Firefox: `searchbar-history`) dan apa yang diketik. |
| `Count` | Berapa kali nilai tersebut dimasukkan. |
| `GUID` | Pengenal entri milik Firefox, yang dirujuk oleh record penghapusan. |
| `Sources` | Dari mana Firefox menyimpan entri tersebut, pada versi yang menyimpannya (dipisahkan dengan garis vertikal dalam CSV). |
| `RowID` | ID baris entri; kosong/null jika hilang saat pemulihan. Nol yang benar-benar tercatat tetap `0`. |

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis tabel-tabel ke `BrowserActivity` sebagai `Browser-History.csv`,
`Browser-Downloads.csv`, `Browser-Cookies.csv`, dan `Browser-FormHistory.csv` (satu berkas `Browser.json` atau
`Browser.jsonl` dengan `-t json` atau `-t jsonl`), beserta `BrowserActivity/Browser-logs.txt`. Timeline gabungan
`-j` menyertakan setiap baris, dengan `Comp` dan pengenal `Artifact` `browser`. `-Z` mengonversi `VisitTime`,
`LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed`, dan
`RemovedTime`.

## Detail lebih lanjut { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
