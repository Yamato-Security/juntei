# Internet Explorer dan Edge lama { #internet-explorer-and-legacy-edge }

## Apa itu database WebCache { #what-the-webcache-database-is }

`WebCacheV01.dat` adalah database fungsi Internet Windows (WinINet) di profil setiap pengguna, sebuah
database ESE. Internet Explorer 10 dan 11 menyimpan riwayat, unduhan, cache, dan cookie-nya di sana, begitu
pula Edge lama (legacy), yaitu Edge di Windows 10 sebelum Edge menjadi berbasis Chromium. Setiap jenis record
adalah sebuah kontainer (container), yaitu tabel tersendiri:

- **Riwayat** (`History`, dan `MSHist01…` untuk satu hari atau satu minggu): URL yang dikunjungi, masing-masing
  beserta akun yang tercatat untuknya, kapan terakhir dikunjungi, dan seberapa sering. File Explorer juga
  mencatat berkas lokal yang dibukanya di sana, sebagai URL `file:///`.
- **Unduhan** (`iedownload`): URL asal sebuah berkas, tempat berkas itu disimpan dan ditulis selama diunduh,
  ukurannya, kapan unduhan dimulai, halaman perujuk, dan tipe MIME.
- **Cookie**: di Windows 10 dan yang lebih baru, cookie itu sendiri (tabel `CookieEntryEx`), beserta host,
  nama, nilai, dan waktunya; sebelumnya, berkas cookie yang menyimpannya.
- **Cache** (`Content`): berkas yang diunduh untuk menampilkan halaman, masing-masing beserta URL-nya, lokasi
  salinannya di disk, dan header respons HTTP. Program lain yang menggunakan WinINet juga menyimpan unduhannya
  di cache ini.
- Penyimpanan web (`DOMStore`), daftar kompatibilitas (`iecompat`), dan lainnya.

Database ini bertahan lebih lama daripada browser-nya: Internet Explorer sudah dihentikan dan Edge berbasis
Chromium menyimpan database SQLite-nya sendiri (lihat [Browser web](browser.md)), tetapi database ini masih ada
di setiap profil, dengan apa yang tercatat sebelumnya, serta apa yang masih dicatat oleh File Explorer dan
program lain.

<span class="evidence-strength">Kesimpulan terbaik:</span> profil pengguna mencatat URL, unduhan, atau berkas
cache tersebut pada waktu yang tercatat; database tidak menyatakan siapa yang duduk di depan keyboard, atau
bahwa sebuah berkas cache pernah dibuka.

## Lokasi dan pengumpulan { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Berkas yang dirujuk oleh kontainer (berkas cache, berkas cookie) berada di foldernya sendiri:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, dan di bawah
`%LOCALAPPDATA%\Packages\` untuk aplikasi.

Kumpulkan seluruh folder `WebCache`. Windows terus membuka database ini selama pengguna masuk (log on),
sehingga salinan dari sistem yang sedang berjalan biasanya dirty: perubahan terbarunya hanya ada di log
transaksi `V01*.log`.

!!! warning "Log transaksi"
    Juntei tidak memutar ulang log transaksi ESE. Database yang tidak ditutup dengan bersih dihitung dalam
    ringkasan sebagai dibiarkan dirty dan dilaporkan sebagai masalah: perubahan yang masih hanya ada di
    log-nya tidak akan muncul. Untuk menyertakannya, putar ulang terlebih dahulu log tersebut ke dalam
    salinan folder, dengan menjalankan `esentutl /r V01 /d` milik Windows di dalam salinan itu.

## Peringatan bagi analis { #analyst-cautions }

- Waktu dalam UTC, hingga 100 nanodetik.
- Entri riwayat adalah sebuah URL, bukan sebuah kunjungan: `AccessedTime` adalah kunjungan terakhirnya dan
  `AccessCount` berapa banyak kunjungannya. Kontainer `History` menyimpan satu entri per URL; kontainer
  satu hari atau satu minggu menyimpan entri periode tersebut, dan namanya memuat hari pertama periode itu
  serta hari setelah hari terakhirnya (`MSHist012026110120261102`: 1 November 2026).
- URL `file:///` di riwayat adalah berkas lokal yang dibuka dari File Explorer dan program lain, bukan
  halaman web.
- `Account` adalah akun yang tercatat untuk sebuah entri (`Visited: alice@…`); `User` adalah folder profil
  tempat path database berada. Keduanya bisa berbeda: folder profil tidak selalu dinamai sesuai akunnya,
  dan database yang disalin ke tempat lain tidak memiliki folder profil di path-nya.
- `Package` menyatakan milik siapa sebuah entri: Edge lama (`microsoft.microsoftedge_8wekyb3d8bbwe`),
  Internet Explorer dalam Enhanced Protected Mode (`windows_ie_ac_001`), atau aplikasi lain. Jika kosong,
  entri itu milik pengguna sendiri: Internet Explorer, File Explorer, dan program lain.
- Microsoft tidak mendokumentasikan record unduhan. Juntei membaca waktu mulai dan ukurannya hanya jika
  header-nya muat di dalam record, dan path serta string lainnya hanya jika dua string terakhir berupa URL dan path
  Windows. Jika tidak, baris tersebut berisi URL milik entri itu sendiri, dan waktu aksesnya bila waktu
  mulai tidak dapat dibaca, disertai sebuah masalah. `Publisher` adalah nama yang dicatat browser, bukan
  hasil pemeriksaan tanda tangan berkas.
- Entri cache menyatakan bahwa berkas tersebut diunduh dan di-cache. Salinan di disk bisa sudah hilang,
  diganti, atau tidak pernah dibuka.
- Entri terhapus belum dipulihkan: ESE dapat menyimpan record yang dihapus di halaman-halamannya sampai
  halaman itu dibersihkan, tetapi Juntei hanya membaca record yang aktif.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` mencari `WebCacheV01.dat` berdasarkan nama; `-f` membaca sebuah database apa pun namanya.

## Tabel keluaran { #output-tables }

CSV membuat lima berkas, masing-masing hanya berisi header-nya jika tidak ada baris. Baris JSON dan JSONL
membawa `Table` (`History`, `Downloads`, `Cookies`, `Entries`, atau `Containers`) setelah `SourceFile`.

| Berkas CSV | Isi |
|---|---|
| `WebCache-History.csv` | Entri dari kontainer riwayat. |
| `WebCache-Downloads.csv` | Unduhan. |
| `WebCache-Cookies.csv` | Cookie (Windows 10 dan yang lebih baru). |
| `WebCache-Entries.csv` | Entri dari semua kontainer lain: berkas cache, berkas cookie, penyimpanan web, daftar kompatibilitas, dan sisanya. |
| `WebCache-Containers.csv` | Kontainer. |

## Field { #fields }

Setiap tabel diawali dengan field berikut:

| Field | Interpretasi |
|---|---|
| `SourceFile` | Database tempat baris tersebut dibaca. |
| `User` | Pengguna Windows yang profilnya memuat path tersebut (folder setelah `Users`). |
| `Package` | Aplikasi pemilik data baris tersebut: folder di bawah `Packages` pada direktori kontainernya (atau direktori tabel cookie-nya). Kosong untuk data milik pengguna sendiri. |

### Riwayat { #history }

| Field | Interpretasi |
|---|---|
| `Container` | Kontainer: `History`, atau `MSHist01` diikuti hari pertama periodenya dan hari setelah hari terakhirnya. |
| `AccessedTime` | Kunjungan terakhir ke URL (UTC); di kontainer satu hari atau satu minggu, kunjungan terakhir pada periode itu. |
| `Account` | Akun yang tercatat untuk entri tersebut. |
| `URL` | Halaman atau berkas, tanpa awalan `Visited: account@`. |
| `AccessCount` | Berapa kali URL tersebut dikunjungi. |
| `EntryID`, `ContainerID` | ID entri di kontainernya, dan ID kontainer. |

### Unduhan { #downloads }

| Field | Interpretasi |
|---|---|
| `StartTime` | Kapan unduhan dimulai (UTC). |
| `URL` | Asal berkas. |
| `TargetPath`, `CurrentPath` | Tempat berkas disimpan, dan tempat berkas ditulis selama diunduh: berkas `.partial` untuk Edge lama, salinan di cache untuk Internet Explorer. |
| `TotalBytes` | Ukuran berkas. |
| `Referrer` | Halaman perujuk. |
| `MimeType` | Tipe berkas sebagaimana dikirim server (Internet Explorer). |
| `Publisher` | Nama yang dicatat Edge lama untuk penerbit berkas. |
| `AccessedTime` | Waktu akses milik entri itu sendiri (UTC), paling sering sama dengan waktu mulai. |
| `DownloadID`, `ContainerID` | ID entri di kontainernya, dan ID kontainer. |

### Cookie { #cookies }

| Field | Interpretasi |
|---|---|
| `LastModified`, `Expires` | Kapan cookie ditetapkan, dan kapan kedaluwarsa (UTC). |
| `Host`, `Name`, `Path` | Situs tujuan cookie, nama, dan path-nya. |
| `Value` | Nilai sebagaimana tersimpan. |
| `Flags` | Flag cookie, sebagaimana tersimpan. |
| `Directory` | Direktori yang diberikan database untuk tabel cookie: milik siapa cookie tersebut (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` milik Edge lama, direktori sebuah aplikasi). |
| `CookieTable`, `EntryID` | Tabel (`CookieEntryEx_<n>`) dan ID cookie di dalamnya. |

### Entri { #entries }

| Field | Interpretasi |
|---|---|
| `Container` | Kontainer: `Content` (berkas cache), `Cookies` (berkas cookie), `DOMStore` (penyimpanan web), `iecompat`, …. |
| `AccessedTime` | Kapan entri terakhir digunakan (UTC). |
| `ModifiedTime` | Perubahan terakhir pada sumber daya, sebagaimana diberikan server (UTC). |
| `CreationTime` | Kapan entri dibuat (UTC). |
| `ExpiryTime`, `SyncTime` | Kapan entri kedaluwarsa, dan kapan terakhir diperiksa ke server (UTC). |
| `URL` | URL sebagaimana tersimpan: URL untuk berkas cache, `Cookie:alice@example.com/` untuk berkas cookie, `DOMStore:…`, `iecompat:…`. |
| `Account` | Akun sebuah berkas cookie. |
| `Filename`, `FilePath` | Berkas entri, dan lokasinya: direktori kontainer dan subfolder yang dirujuk entri. |
| `FileSize` | Ukuran berkas. |
| `AccessCount`, `SyncCount` | Berapa kali entri digunakan, dan diperiksa ke server. |
| `EntryType`, `Flags` | Tipe dan flag entri, sebagaimana tersimpan. |
| `RedirectURL` | Tujuan redirect URL tersebut. |
| `ResponseHeaders`, `RequestHeaders` | Header respons HTTP (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) dan header permintaan, sebagaimana tersimpan, lengkap dengan baris-barisnya. |
| `EntryID`, `ContainerID` | ID entri di kontainernya, dan ID kontainer. |

### Kontainer { #containers }

| Field | Interpretasi |
|---|---|
| `ContainerID`, `Name` | ID dan nama kontainer. |
| `Directory` | Lokasi berkas-berkasnya di disk. |
| `SecureDirectories` | Subfolder dari `Directory` tempat berkas-berkasnya berada (dipisahkan koma dalam CSV). |
| `PartitionID` | `M` untuk kontainer milik pengguna sendiri, `L` untuk kontainer Protected Mode Internet Explorer (folder `Low`), SID app container untuk kontainer milik sebuah aplikasi. |
| `LastAccessTime`, `LastScavengeTime` | Kapan kontainer terakhir digunakan, dan terakhir dibersihkan (UTC). |

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis tabel-tabel ke `BrowserActivity` sebagai `WebCache-History.csv`,
`WebCache-Downloads.csv`, `WebCache-Cookies.csv`, `WebCache-Entries.csv`, dan
`WebCache-Containers.csv` (satu berkas `WebCache.json` atau `WebCache.jsonl` dengan `-t json` atau
`-t jsonl`), beserta `BrowserActivity/WebCache-logs.txt`. Timeline gabungan `-j` menyertakan setiap baris,
dengan `Comp` dan pengenal `Artifact` `webcache`. `-Z` mengonversi setiap waktu: field yang namanya
berakhiran `Time`, `LastModified`, dan `Expires`.

## Detail lebih lanjut { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
