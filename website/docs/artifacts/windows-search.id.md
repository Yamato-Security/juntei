# Indeks Windows Search { #windows-search-index }

## Apa yang dicatat indeks { #what-the-index-records }

Windows Search (layanan `WSearch`) merayapi (crawl) profil pengguna, menu Start, kotak surat Outlook,
riwayat Internet Explorer, berkas offline, dan apa pun lainnya yang berada dalam cakupannya, lalu menyimpan
apa yang dibacanya di `Windows.edb`, sebuah database ESE. Dua bagiannya penting bagi investigasi:

- **property store** (`SystemIndex_PropertyStore`; `SystemIndex_0A` di Windows 7) dengan satu baris
  per item yang diindeks: path, nama, tipe, dan ukurannya, pemiliknya, waktu pembuatan, modifikasi, dan
  akses yang disalin pengindeks (indexer) dari berkas, kapan pengindeks membacanya, judul dan penulis
  sebuah dokumen, awal teksnya, nama komputer, dan ratusan properti lainnya;
- tabel **gatherer** (`SystemIndex_Gthr`, `SystemIndex_GthrPth`) dengan catatan milik crawler sendiri
  tentang setiap item: path URL-nya, waktu modifikasinya saat dimasukkan ke antrean, nomor dan flag
  perayapan (crawl), serta penghitung penghapusan.

Sebuah item tetap berada di indeks setelah berkas, pesan, atau halamannya hilang, sampai crawler menyadari
penghapusan itu. Karena itu, indeks menyebutkan berkas-berkas yang sudah tidak ada lagi, beserta metadata
dan cuplikan teksnya sebagaimana adanya saat diindeks.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengindeks membaca item tersebut pada path
yang tercatat, dengan metadata yang tercatat, pada `GatherTime`, di komputer yang disebutkan. Indeks
menunjukkan bahwa berkas itu ada dan apa isinya. Indeks tidak menunjukkan bahwa seorang pengguna membukanya.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 hingga Windows 11 21H2, serta rilis Windows Server pada tahun-tahun yang sama, menyimpan indeks
di `Windows.edb`. Windows 11 22H2 dan yang lebih baru menyimpannya dalam database SQLite di folder yang sama
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); Juntei menemukan dan menyebutkan namanya, dan
melaporkannya sebagai format yang lebih baru yang tidak didukung.

Kumpulkan seluruh folder `Applications\Windows` dari image disk, volume shadow copy, atau setelah
menghentikan layanan `WSearch`: database ini terbuka selama layanan berjalan, dan perubahan terbarunya hanya
ada di log transaksi `MSS*.log`, yang tidak diputar ulang oleh Juntei. Indeks yang disalin dari sistem yang
sedang berjalan dibiarkan dirty; keluaran menyatakannya, dan membaca apa yang ada di disk. Masukan direktori
dan `auto-parse` membaca setiap `Windows.edb` pada kedalaman berapa pun; gunakan `-f` untuk satu berkas.

## Peringatan bagi analis { #analyst-cautions }

- Parse salinan bukti yang tidak berubah. Juntei memetakan berkas ke memori untuk akses efisien; jangan mengubah atau memotong basis data selama pembacaan.
- `GatherTime` adalah saat pengindeks membaca item, bukan saat pengguna membuat, mengubah, atau membukanya.
  Pengindeksan berjalan di latar belakang, sering kali beberapa menit hingga beberapa jam setelah perubahan,
  dan berhenti sementara selagi komputer sibuk atau memakai baterai.
- `DateCreated`, `DateModified`, dan `DateAccessed` adalah salinan yang diambil pengindeks dari berkas.
  Nilai-nilai ini diperbarui saat item diindeks ulang, sehingga menggambarkan berkas pada saat `GatherTime`.
- `ItemPath` adalah path tampilan. `ItemURL` dan `GatherPath` adalah URL milik crawler: `file:` untuk
  berkas (dengan volume sebagai `[GUID]` di Windows 8 dan yang lebih baru), `iehistory://{SID}/` untuk
  riwayat Internet Explorer, `mapi://` untuk item Outlook, `csc://{SID}/` untuk berkas offline, dan
  `winrt://{SID}/` untuk konten aplikasi Windows.
- `User` adalah profil yang disebutkan dalam path item (`C:\Users\<name>\…`). Ini adalah tempat item
  berada, bukan siapa yang membuatnya; `FileOwner` adalah pemilik berkas sebagaimana dilihat pengindeks.
- `AutoSummary` adalah awal teks item. Nilai ini dapat mengutip dokumen yang sejak itu telah dihapus atau
  dienkripsi, dan merupakan cuplikan yang terpotong, bukan dokumennya.
- Database yang dirty dapat memuat halaman (page) yang belum dijangkau oleh halaman cabangnya (branch
  page). Juntei mengikuti rantai halaman daun (leaf page) itu sendiri dan menyebutkan dalam ringkasan
  berapa banyak yang dibacanya dengan cara itu. Perubahan terbarunya masih berada di log transaksi.
- Item dalam tabel gatherer tanpa baris property store (`InTables` bernilai `Gthr`) pernah dimasukkan ke
  antrean atau dirayapi, dan propertinya belum ditulis atau telah dihapus. Path, nama, dan waktu
  gatherer-nya masih ada.
- Windows 7 menyimpan string dalam pengodean terkompresi dan teracak (obfuscated) miliknya sendiri. Juntei
  mendekodenya; nilai yang tidak dapat didekode tetap dalam heksadesimal, dengan catatan di `Problems`.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV adalah format berkas bawaan. Dengan `-t csv`, `-t json`, atau `-t jsonl` tanpa `-o`, catatan ditulis ke
stdout. Tanpa kedua opsi tersebut, hanya ringkasan yang ditampilkan. Tambahkan `-C` untuk menimpa keluaran
yang sudah ada. Keluaran tidak pernah ditulis menimpa salah satu berkas masukan, bahkan dengan `-C`.

Juntei menulis satu catatan per item yang diindeks, dengan menggabungkan (join) property store dan tabel
gatherer berdasarkan pengenal item. Ringkasan menghitung database menurut skema, database yang dirty, item
yang ada di kedua tabel, hanya di gatherer, dan hanya di property store, folder, item yang ditandai
terhapus, item per store, item tanpa waktu tersimpan, dan masalah.

Kerusakan tidak menghentikan proses. Database yang terpotong tetap mempertahankan item yang dimuat
halaman-halamannya yang tersisa, dan ringkasan menyebutkan apa yang tidak dapat dibaca. Berkas bernama
`Windows.edb` yang bukan indeks Windows Search dilaporkan dan dilewati. Database lainnya tetap dibaca.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `DocumentID` | Pengenal item dalam indeks: `WorkID` milik property store (`DocID` di Windows 7) dan `DocumentID` milik baris gatherer. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (item yang diantrekan atau dirayapi tanpa properti), atau `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: path sebagaimana ditampilkan Windows. Kosong bila property store tidak memiliki barisnya. |
| `ItemURL` | `System_ItemUrl`: URL milik crawler untuk item tersebut (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | Path milik gatherer, digabungkan dari pohon cakupan (scope) dan nama berkas pada baris tersebut. |
| `Name` | `System_ItemName`, atau nama berkas pada baris gatherer. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (ekstensinya, atau `Directory`) dan nama tampilannya. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; beberapa nilai digabungkan dengan `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (berkas offline), atau `winrt` (aplikasi Windows). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` dan `System_IsDeleted`, sebagaimana tersimpan. |
| `DeletedCount` | Penghitung penghapusan pada baris gatherer. |
| `Size` | `System_Size` dalam byte. Kosong untuk folder, yang memuat placeholder. |
| `FileAttributes` | `System_FileAttributes` sebagai angka (16 direktori, 32 arsip, 2 tersembunyi, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) sebagaimana dibaca pengindeks. |
| `DateCreated`, `DateModified`, `DateAccessed` | Waktu berkas sebagaimana disalin pengindeks (UTC). |
| `ItemDate` | `System_ItemDate`: tanggal milik item itu sendiri, seperti waktu kirim sebuah pesan atau tanggal pengambilan sebuah foto; untuk berkas biasanya waktu modifikasinya hingga ke detik. |
| `GatherTime` | `System_Search_GatherTime`: saat pengindeks membaca item (UTC). |
| `GatherLastModified` | `LastModified` pada baris gatherer: waktu modifikasi item saat crawler memasukkannya ke antrean (UTC). |
| `Title`, `Author` | `System_Title` dan `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: awal teks item. |
| `CompName` | `System_ComputerName`: komputer pemilik indeks. |
| `TargetURL`, `DateVisited` | URL dan waktu kunjungan sebuah item riwayat Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Item riwayat aktivitas Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | Profil yang disebutkan dalam path item, dan dari path mana nilai itu dibaca (`ItemPath`, `ItemURL`, atau `GatherPath`). Kosong di luar profil. |
| `ScopeID`, `SDID` | Pengenal cakupan (scope) dan deskriptor keamanan pada baris gatherer. |
| `CrawlNumber` | `CrawlNumberCrawled` pada baris gatherer. |
| `GatherFlags` | `TransactionFlags` pada baris gatherer. |
| `Priority` | Prioritas perayapan item yang masih dalam antrean. ESE menandainya NULL setelah item dirayapi, sehingga kosong untuk sebagian besar item. |
| `PropertyCount` | Berapa banyak properti yang dimuat property store untuk item tersebut. |
| `OtherProperties` | Setiap properti lainnya, sebagai `Name=Value` yang dipisahkan dengan `; `. |
| `SourceFile` | Database yang dibaca, relatif terhadap direktori masukan, atau namanya bila memakai `-f`. |
| `Problems` | Nilai yang tidak dapat didekode, waktu yang bukan waktu, baris duplikat, dan kerusakan. Jangan disembunyikan. |

JSON dan JSONL mempertahankan field yang sama dengan nilai bertipe: angka tetap angka dan nilai yang hilang
menjadi `null`. Sebagai ganti `OtherProperties`, keduanya memiliki `Properties`, yaitu setiap properti item
beserta nama, pengenal numerik, tipe kolom, dan nilainya, dan untuk waktu, jumlah tick FILETIME mentahnya
di samping waktu hasil pembacaannya. `Gather` memuat kolom-kolom baris gatherer.

## Perilaku timeline { #timeline-behavior }

`auto-parse` menyimpan catatan sebagai `FileExistence/WindowsSearch.csv` (atau `.json`/`.jsonl`) dan
menambahkannya ke timeline gabungan `-j`. `super-timeline` membuat peristiwa dari `DateCreated`,
`DateModified`, dan `DateAccessed` (waktu berkas sebagaimana dicatat Windows Search), dari `GatherTime`
(saat item diindeks), dari `DateVisited` milik item riwayat Internet Explorer, serta dari
`ActivityStartTime` dan `ActivityEndTime` milik sebuah aktivitas. `GatherLastModified` menjadi peristiwa
hanya untuk item yang tidak memiliki waktu modifikasi di property store. Item tanpa satu pun dari
waktu-waktu ini tetap berada di keluaran hasil penguraian dan dihitung sebagai item tanpa waktu.

## Strategi investigasi { #investigation-strategy }

1. Filter `Store` ke `file` dan cari berkas, nama, dan kata-kata yang terkait kasus di `ItemPath`, `Name`,
   dan `AutoSummary`. Indeks menyebutkan berkas-berkas yang sejak itu telah dihapus.
2. Baca `GatherTime` sebagai waktu terakhir berkas itu ada dengan metadata yang tercatat, dan
   `DateModified` sebagai waktu modifikasinya pada saat itu.
3. Gunakan `FileOwner` dan `User` untuk mengaitkan berkas dengan sebuah akun, lalu konfirmasi dengan `$MFT`,
   Jurnal USN, dan `$Secure:$SDS`.
4. Untuk e-mail (`Store` `mapi`) dan riwayat Internet Explorer (`iehistory`), lakukan pivot ke berkas
   Outlook dan WebCache untuk mendapatkan catatan lengkapnya.
5. Bandingkan indeks antar-host: `CompName` menyebutkan komputer pemilik indeks, dan indeks yang
   dikumpulkan dari sebuah image menyebutkan berkas-berkas pada image itu.

## Versi yang diuji { #tested-versions }

Pengujian menggunakan indeks publik yang hasilnya dibaca secara independen oleh alat lain:

- Indeks Windows 7 milik plaso (`SystemIndex_0A`), diperiksa terhadap `esedbexport` milik libyal,
  termasuk pengodean string Windows 7.
- Indeks Windows 10 milik DFIRArtifactMuseum, yang dibiarkan dirty, diperiksa terhadap SIDR milik Stroz
  Friedberg, dan indeks Windows 11-nya (revisi format ESE 200), diperiksa terhadap `esedbexport`.
- Indeks SQLite Windows 11 22H2 milik DFIRArtifactMuseum, dilaporkan sebagai format yang lebih baru yang
  tidak didukung.
- Database yang terpotong, kosong, atau berjenis lain.

## Detail lebih lanjut { #further-details }

- [Microsoft — Ikhtisar Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — Format berkas database ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Kode sumber Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
