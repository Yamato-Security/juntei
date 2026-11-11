# Keranjang Sampah (Recycle Bin) { #recycle-bin }

## Apa yang dicatat Recycle Bin { #what-the-recycle-bin-records }

Saat pengguna menghapus berkas atau folder di File Explorer, Windows memindahkannya ke Recycle Bin
pada volume tempat item itu berada dan menulis record indeks kecil di sebelahnya: path item tersebut,
ukurannya, dan kapan item itu dihapus. Setiap pengguna memiliki folder Recycle Bin di setiap volume,
yang dinamai sesuai SID pengguna, sehingga folder itu juga menunjukkan akun mana yang menghapus item
tersebut.

| Versi Windows | Indeks | Konten yang dihapus |
|---|---|---|
| Vista dan yang lebih baru | Satu berkas `$I…` per item (format versi 1 pada Vista hingga 8.1, versi 2 pada Windows 10 dan yang lebih baru) | `$R…`, dengan nama yang sama setelah `$R`: berkas itu sendiri, atau folder beserta isinya |
| NT 4.0, 2000, XP, Server 2003 | Satu berkas `INFO2` per Recycle Bin: record 800 byte dengan path ANSI dan path Unicode | `D<drive letter><index><extension>`, misalnya `Dc1.txt` |
| 95, 98, ME | Satu berkas `INFO2` (`INFO` pada Windows 95): record 280 byte yang hanya berisi path ANSI | Nama `D…` yang sama |

<span class="evidence-strength">Kesimpulan terbaik:</span> akun yang Recycle Bin-nya menyimpan record
tersebut memindahkan item di path asli ke Recycle Bin pada waktu yang tercatat. Penghapusan dengan
Shift+Delete, dari command line, atau dari sebagian besar program tidak melalui Recycle Bin dan tidak
meninggalkan record.

## Lokasi dan pengumpulan { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Kumpulkan berkas `$I` atau INFO2 dari Recycle Bin setiap volume. Jika berkas `$R` dan `D…` juga
dikumpulkan, terlihat item terhapus mana yang masih ada; tanpa berkas tersebut, `RecycledFileFound`
bernilai `false` untuk setiap item. Konten yang dihapus merupakan bukti tersendiri dan bisa berupa
malware: tangani sebagaimana mestinya.

## Peringatan bagi analis { #analyst-cautions }

- `DeletedTime` adalah waktu item dipindahkan ke Recycle Bin, menurut jam sistem, dalam bentuk
  FILETIME UTC. Ini bukan waktu Recycle Bin dikosongkan. Waktu pembuatan dan modifikasi milik item
  itu sendiri ada di `$MFT` dan berkas `$R`.
- SID menunjukkan akun yang menghapus item, belum tentu pemilik berkas. SID yang tidak memiliki profil di
  `ProfileList` hive SOFTWARE (akun yang telah dihapus, atau akun domain yang tidak pernah logon)
  tidak memiliki `User`.
- Saat Recycle Bin dikosongkan, atau sebuah item dipulihkan atau dihapus dari Recycle Bin, Windows
  Vista dan yang lebih baru menghapus berkas `$I` dan `$R` milik item tersebut. Namanya dapat tersisa
  di Jurnal USN, `$MFT`, dan slack `$I30`; Juntei tidak memulihkan record `$I` yang terhapus dari
  sumber-sumber itu. INFO2 tetap menyimpan record semacam itu dan mengosongkan byte pertama path
  ANSI-nya: `RemovedFromBin` bernilai `true`, dan Juntei mengembalikan karakter tersebut berdasarkan
  nomor drive.
- `FileSize` dalam berkas `$I` adalah ukuran item; untuk folder, total ukuran isinya. Dalam INFO2,
  nilainya adalah ruang yang dipakai item di disk, dalam cluster utuh (4.096 byte untuk berkas kecil).
- Path ANSI di INFO2 sering berbentuk 8.3 dan memerlukan code page sistem: gunakan `--code-page 932`
  untuk Windows berbahasa Jepang, 936 atau 950 untuk bahasa Tionghoa. Path Unicode, bila ada, dimuat
  di `OriginalPath`.
- Beberapa sistem Windows Vista menulis berkas `$I` berukuran 543 byte yang field ukurannya kurang
  satu byte. Juntei membaca waktu dan path-nya, lalu membiarkan `FileSize` kosong dan melaporkan
  masalahnya.
- `RecycledFileFound` menggambarkan hasil pengumpulan, bukan endpoint: `false` dapat berarti berkas
  `$R` tidak ikut dikumpulkan, atau item tersebut sudah tidak ada di Recycle Bin.
- Recycle Bin pada volume non-sistem hanya memakai profil dari komputer hasil koleksi yang sama, berdasarkan metadata atau struktur folder drive/volume yang dikenali. Jika atribusi ambigu, pengguna tetap tidak diketahui; profil host lain tidak dipakai.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Dengan `-d`, Juntei membaca berkas bernama `$I` yang diikuti enam huruf atau angka (dengan ekstensi
apa pun), `INFO2`, dan `INFO` di dalam folder `RECYCLED`. Dengan `-f`, Juntei membaca berkas tersebut
apa pun namanya.

## Field keluaran CSV { #csv-output-fields }

| Field | Arti |
|---|---|
| `SourceFile` | Berkas `$I` atau INFO2. |
| `DeletedTime` | Waktu item dipindahkan ke Recycle Bin (UTC). |
| `OriginalPath` | Lokasi item sebelumnya. Untuk INFO2, path Unicode bila record memilikinya, jika tidak, path ANSI. |
| `FileSize` | `$I`: ukuran item dalam byte (total untuk folder). INFO2: ruang yang dipakainya di disk, dalam cluster utuh. Kosong bila tidak dapat dibaca. |
| `User`, `SID` | SID folder Recycle Bin, dan pengguna yang namanya menjadi nama profil SID tersebut di `ProfileList`. |
| `RecycledFile`, `RecycledFileFound` | Path konten yang dihapus di sebelah indeks (`$R…`, `D<drive><index><extension>`), dan apakah konten tersebut ada dalam hasil pengumpulan. |
| `Format`, `FormatVersion` | `$I` 1 (Vista hingga 8.1) atau 2 (Windows 10 dan yang lebih baru); `INFO2` 0 (95), 2 (NT 4.0), 4 (98), atau 5 (ME, 2000 hingga Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: nomor record (angka dalam nama berkas `D…`-nya), drive, dan posisi awal record di dalam INFO2. Path UNC tidak memiliki drive. |
| `RemovedFromBin` | INFO2: `true` bila item telah dipulihkan atau dihapus dari Recycle Bin. |
| `ShortPath` | INFO2: path ANSI, bila berbeda dari `OriginalPath` (sering berupa nama 8.3). |
| `Problems` | Kerusakan: berkas yang terpotong atau terlalu panjang, nomor drive yang tidak valid, path yang bukan UTF-16 valid, waktu penghapusan yang bukan waktu yang valid. |

JSON dan JSONL memiliki field yang sama, dengan angka dan boolean yang bertipe, `null` untuk nilai
yang tidak dimiliki record, dan `Problems` sebagai daftar.

Jika direktori tidak dapat didaftar atau nama konten ambigu, `RecycledFileFound` tidak diketahui (sel CSV kosong atau JSON `null`), dengan penyebab di `Problems`. Output tidak boleh menimpa konten Recycle Bin, termasuk melalui hard link, atau ditulis di dalam folder yang dihapus. Hasil lama dipertahankan jika tidak ada indeks yang dapat dibaca.

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis `FileExistence/RecycleBin.csv` (`RecycleBin.json` atau `RecycleBin.jsonl` dengan
`-t json` atau `-t jsonl`), beserta `FileExistence/RecycleBin-logs.txt`. Opsi `--code-page`-nya
berlaku untuk INFO2. Dari image disk, berkas `$I` dan INFO2 disalin; berkas `$R` dan `D…` menjadi
placeholder kosong, sehingga `RecycledFileFound` tetap terisi. Timeline gabungan `-j` menyertakan
setiap baris dengan pengenal `Artifact` `recycle-bin`, `-Z` mengonversi `DeletedTime`, dan
[`super-timeline`](../commands/super-timeline.md) membuat satu peristiwa
`Recycle Bin deletion (moved to the bin)` per item.

## Strategi investigasi { #investigation-strategy }

1. Urutkan berdasarkan `DeletedTime` per pengguna, lalu bandingkan dengan waktu aktivitas yang sedang
   Anda investigasi.
2. Cari `OriginalPath` di LNK, Jump Lists, ShellBags, dan RecentDocs untuk melihat apakah item
   tersebut digunakan sebelum dihapus.
3. Di Jurnal USN, penghapusan tampak sebagai penggantian nama ke dalam `$Recycle.Bin` sekitar
   `DeletedTime`; `$MFT` memberikan waktu milik item itu sendiri.
4. Bila `RecycledFileFound` bernilai `true`, hitung hash dan periksa konten `$R` atau `D…`.
5. Jika Recycle Bin telah dikosongkan, cari nama `$I` dan `$R` di Jurnal USN dan `$MFT`.

## Detail lebih lanjut { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
