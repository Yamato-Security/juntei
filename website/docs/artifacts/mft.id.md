# Master File Table (`$MFT`) { #master-file-table-mft }

## Apa itu MFT { #what-the-mft-is }

NTFS merepresentasikan setiap berkas dan direktori dengan sebuah record di Master File Table. Record berisi
atribut seperti nama, referensi parent, timestamp, ukuran, flag, pengenal keamanan,
informasi reparse, dan metadata data stream. Konten berkas berukuran kecil dan alternate data stream dapat
bersifat **resident**, artinya byte-nya disimpan di dalam record MFT itu sendiri.

MFT bersifat fundamental karena mencakup record yang teralokasi maupun yang dihapus di seluruh volume.
MFT dapat menunjukkan bahwa sebuah berkas pernah ada, di mana berkas itu ditautkan, apakah berkas itu dihapus, dan bagaimana metadatanya
berubah—bahkan ketika penjelajahan direktori biasa tidak lagi menampilkannya.

<span class="evidence-strength">Kesimpulan terbaik:</span> sebuah record berkas NTFS dengan metadata
yang ditampilkan ada pada saat pengumpulan atau bertahan di record yang tidak teralokasi. MFT saja tidak
membuktikan eksekusi atau niat pengguna.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\$MFT
```

`$MFT` adalah metadata NTFS yang dilindungi. Kumpulkan dengan alat akuisisi forensik atau alat triase yang mampu
mengakses NTFS secara raw; penyalinan biasa dalam user mode umumnya tidak dapat membacanya. Pasangkan dengan `$J`, `$LogFile`,
dan metadata volume bila memungkinkan.

## Peringatan bagi analis { #analyst-cautions }

- Record yang dihapus dapat tertimpa sebagian atau nomor record-nya dapat digunakan kembali.
- `SN` membedakan generasi dari nomor entri yang sama. Cocokkan entri **dan** sequence.
- Timestamp `$STANDARD_INFORMATION` (`0x10`) dan `$FILE_NAME` (`0x30`) memiliki perilaku pembaruan
  yang berbeda. Tidak ada satu pun yang kebal terhadap manipulasi.
- `SI<FN`, `uSecZeros`, dan `Copied` adalah petunjuk, bukan vonis timestomping otomatis.
- Hard link menghasilkan beberapa atribut `$FILE_NAME` dan path untuk satu record berkas.
- NTFS dapat menyimpan beberapa nama (termasuk DOS 8.3), alternate data stream, dan extension record.
- Pembaruan waktu akses dapat dinonaktifkan, ditunda, atau digabungkan.
- Path lengkap direkonstruksi dari record parent. `[Orphaned]`, `[Unknown]`, atau `PathUnknown`
  menandakan hierarki induk yang hilang/digunakan kembali/rusak—bukan nama direktori secara harfiah.
- **Slack** dari sebuah record FILE (byte-nya setelah ukuran yang digunakan) dapat menyimpan nama yang pernah dimilikinya:
  entri indeks dari direktori kecil yang indeksnya dipindahkan ke index allocation (sebagian besar masih
  digunakan di sana), nama sebelumnya, atau penghuni sebelumnya dari record yang digunakan kembali. Nama yang dipulihkan
  tidak memiliki `$STANDARD_INFORMATION`, dan header entri sebuah nama mungkin telah tertimpa. Bila
  ukuran yang digunakan atau atribut sebuah record tidak menunjukkan di mana slack-nya dimulai, tidak ada yang dipulihkan darinya
  dan `ParseErrors` pada baris tersebut menyatakan `slack not read` beserta alasannya.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
      --reconstruct-filesystem <DIR>  Recreate names and resident content in a new tree
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

Jangan pernah mengeksekusi resident stream yang diekstrak dengan `-e`; Juntei memberinya nama `.dontrun` dengan alasan yang jelas.
Kolom resident data dari `-i` berisi byte yang sama: perlakukan sebagai konten yang tidak tepercaya.

`-D` mencetak satu entri: header-nya, setiap atribut (beserta data run) dan nama-nama di slack-nya sebagaimana
ditulis oleh `-t json -x`, lalu byte record-nya. `-L` mencantumkan setiap nama di sebuah direktori, baik yang digunakan maupun
yang dihapus, beserta nomor entri dan sequence, tipe, ukuran, dan waktu modifikasinya. Direktori ditentukan dengan
entri (`30`, `0x1E`, `30-1`) atau dengan path (`\Windows\System32`, `C:/Windows`; tidak peka huruf besar/kecil).

## Field keluaran CSV { #csv-output-fields }

| Field | Interpretasi |
|---|---|
| `EntryNum` (Entry Number) | Nomor record MFT. Gabungkan dengan `SN`; nomor record digunakan kembali. |
| `SN` (Sequence Number) | Penghitung generasi yang digunakan oleh referensi NTFS. Ketidakcocokan dapat mengungkap hubungan yang usang/dihapus. |
| `InUse` | Flag alokasi pada saat pengumpulan. `false` berarti record bebas/dihapus, bukan berarti semua byte masih utuh. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | Referensi direktori parent dari `$FILE_NAME`. Validasi kedua bagian. |
| `ParentPath` | Path parent yang direkonstruksi. Penanda unknown/orphan memperlihatkan hierarki induk yang tidak terselesaikan. |
| `File` (Filename) | Nama dari `$FILE_NAME` atau stream pada baris ini. Satu record dapat memiliki beberapa baris. |
| `Ext` (Extension) | Ekstensi yang diurai dari nama demi kemudahan; bukan penentuan tipe berkas yang tepercaya. |
| `Size` (File Size) | Ukuran logis yang terkait dengan baris/stream. Alokasi sparse/terkompresi dapat berbeda. |
| `RefCnt` (Reference Count) | Jumlah referensi hard link NTFS; bandingkan dengan jumlah nama yang tersisa secara hati-hati. |
| `ReparseTarget` | Target yang didekode untuk reparse point yang didukung seperti junction/symlink. Path dapat mengalihkan. |
| `Dir` (Directory) | Record memiliki flag direktori. |
| `HasADS` (Has Alternate Data Streams) | Record berisi alternate data stream bernama. Hal ini tidak membuatnya berbahaya. |
| `IsADS` (Is Alternate Data Stream) | Baris ini merepresentasikan alternate data stream bernama, bukan data berkas tanpa nama. |
| `SI<FN` (Standard Information Before File Name) | Waktu pembuatan SI mendahului waktu pembuatan FN yang berbeda. Investigasi penyalinan/timestomping dan penyebab yang sah. |
| `uSecZeros` (Microsecond Zeros) | Waktu pembuatan/modifikasi SI jatuh tepat pada batas milidetik. Hanya heuristik timestomping yang lemah. |
| `Copied` | Waktu modifikasi mendahului waktu pembuatan. Sering konsisten dengan penyalinan berkas; bukan bukti tindakan penyerang. |
| `SI-Flags` (Standard Information Flags) | Atribut/flag berkas `$STANDARD_INFORMATION`. Flag hidden/system dapat menjadi titik pivot yang berguna. |
| `Type` (Name Type) | Namespace `$FILE_NAME` (Win32, DOS, POSIX, atau gabungan). Gunakan untuk menghindari penghitungan nama ganda. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | Waktu SI. Field timeline yang umum, tetapi alat user mode dapat mengubah beberapa di antaranya. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | Waktu FN. Diperbarui secara berbeda dan umumnya lebih sulit diubah oleh API biasa; tetap tidak kebal perubahan. |
| `USN` (Update Sequence Number) | USN terakhir yang terkait dengan atribut SI record. Lakukan pivot ke `$J` jika USN tersebut masih ada. |
| `LogfileSN` (Logfile Sequence Number) | Nomor urut `$LogFile` NTFS dari header record FILE; berguna dalam analisis transaksi tingkat rendah. |
| `SID` (Security ID) | Indeks ke `$Secure` NTFS; bukan SID Windows dengan sendirinya. Gabungkan dengan `SecurityID` dari [`parse-ntfs secure-sds`](secure-sds.md) untuk pemilik dan ACL berkas. |
| `ObjectID-FileDroid` | Object ID / pengenal distributed-link-tracking bila ada; korelasikan dengan droid LNK/Jump List. |
| `LoggedUtilStream` | Konteks `$LOGGED_UTILITY_STREAM` yang didekode bila didukung. Tinjau nilai yang tidak lazim. |
| `ZoneID-Contents` | Konten `Zone.Identifier` resident, seperti ZoneId dan URL unduhan. Konten ini mungkin tidak ada, telah dihapus, atau non-resident. |
| `SourceFile` | `$MFT` input, penting saat menggabungkan beberapa host atau volume. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | Dengan `-i`: konten `$DATA` resident pada baris (milik berkas, atau milik alternate data stream), sebagai Base64, sebagai byte (`41-42`), dan sebagai teks bila berupa ASCII yang dapat dicetak. Kosong bila data non-resident atau tersaring oleh `-E`/`-M`. |
| `ParseErrors` | Kerusakan atau hubungan yang tidak didukung yang memengaruhi baris ini. Jangan pernah membuang kolom ini saat memfilter. |
| `FromSlack` | Dengan `-x`: `True` untuk nama yang dipulihkan dari slack sebuah record FILE. `EntryNum` dan `SN`-nya adalah berkas yang ditunjuk oleh nama tersebut (kosong bila header entrinya tertimpa), timestamp yang dipulihkan berada di kolom `(FN)`, dan `InUse` bernilai `False`. |
| `SlackRecord`, `SlackOffset` | Dengan `-x`: record FILE yang slack-nya menyimpan nama tersebut, berdasarkan posisinya di `$MFT` dan bukan nomor record yang tertulis di dalamnya (milik extension record itu sendiri, meskipun nama-namanya dicantumkan bersama base record-nya), serta offset byte nama tersebut di `$MFT`. `ParseErrors` pada baris slack adalah masalah record tersebut, seperti `fixup values do not match`: byte yang dipulihkan mungkin terpengaruh. |
| `CopyOfEntryInUse` | Dengan `-x`: berkas yang sedang digunakan masih memiliki nama ini di direktori yang sama (dan merupakan berkas yang sama, bila referensinya masih ada). Nama yang tidak dimiliki oleh berkas mana pun yang sedang digunakan adalah yang menarik: dihapus, diubah namanya, atau penghuni sebelumnya dari record yang digunakan kembali. |

Kolom timestamp mengikuti urutan MACB (Modified, Accessed, Changed, Birth/creation):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, lalu `Created` SI/FN.

`(SI)` berarti `$STANDARD_INFORMATION`; `(FN)` berarti `$FILE_NAME`. Nama-nama ini mendeskripsikan
sumber yang biasa untuk record yang lengkap. Untuk baris nama berkas normal tanpa atribut SI,
fallback yang ada menempatkan waktu pembuatan FN di `Created (FN)` dan nilai modifikasi/perubahan record/akses FN
masing-masing di `DataModified (SI)`, `MetadataChanged (SI)`, dan `Accessed (SI)`.
Baris slack yang dipulihkan menempatkan keempat waktu di kolom `(FN)` dan membiarkan kolom SI kosong.
Nilai fallback baris normal **bukan** bukti SI yang independen. Gunakan atribut JSON asli
dan asal-usul slack saat menentukan sumber timestamp.

`-t ys-ntfs-parser-csv` (sebelumnya `ys-mft-parser-csv`, masih diterima) menggunakan skema berorientasi entri
dari parser yang dipelihara. Nama kolom yang mudah dibaca ini berlaku untuk keluaran default `-t csv`,
termasuk `MFT.csv` milik `auto-parse`. JSON/JSONL mempertahankan setiap
atribut (termasuk `$DATA` resident, dalam heksadesimal) dan `FullPath`; gunakan format ini ketika CSV
investigatif yang datar menghilangkan struktur tingkat rendah. Dengan `-x`, setiap entri JSON juga memiliki
`SlackFileNames`.

## Strategi investigasi { #investigation-strategy }

1. Filter jendela waktu, tetapi pertahankan record yang dihapus dan kedua set timestamp.
2. Cari path lengkap, ekstensi ganda yang mencurigakan, ADS, direktori staging, dan hash/nama yang dikenal.
   Dengan `-x`, sertakan nama slack yang tidak dimiliki oleh berkas mana pun yang sedang digunakan (`CopyOfEntryInUse` `False`); dengan
   `-i -E ps1,bat,vbs,js`, baca skrip kecil langsung dari CSV.
3. Lakukan pivot entri/sequence dan `USN` ke Jurnal USN.
4. Bandingkan referensi MFT dari Prefetch/LNK/Jump List dengan record target.
5. Korelasikan penghapusan dengan karantina Defender atau aktivitas USN `FILE_DELETE`/rename.

## Detail lebih lanjut { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Crate mft milik ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Parser mft oleh Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [Dokumentasi format libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)

Contoh berikut merekonstruksi jalur yang dapat ditelusuri. Konten nonresiden menjadi berkas kosong. Rincian: [Rekonstruksi sistem berkas](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' --reconstruct-filesystem reconstructed
```
