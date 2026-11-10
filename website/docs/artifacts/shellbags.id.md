# ShellBags { #shellbags }

## Apa itu ShellBags { #what-shellbags-are }

Explorer mengingat bagaimana setiap folder yang dibuka pengguna ditampilkan: tampilannya, ukuran ikon, urutan pengurutan, dan
posisi jendela. Untuk menemukan kembali pengaturan sebuah folder, Explorer menyimpan identitas folder tersebut di registry pengguna
sebagai pohon shell item, yaitu item yang sama dengan yang disimpan dalam target ID list sebuah tautan: sebuah drive, lalu setiap
folder hingga folder yang dibuka. Pohon-pohon inilah yang disebut ShellBags.

ShellBags mencakup folder yang dibuka di Explorer dan di dialog buka dan simpan, termasuk folder di drive lepas pasang,
network share, perangkat portabel (ponsel, kamera), arsip ZIP dan server FTP, serta halaman
Control Panel. ShellBags tetap ada setelah folder, drive, atau network share tersebut tidak ada lagi.

<span class="evidence-strength">Kesimpulan terbaik:</span> Explorer milik pengguna ini menampilkan folder ini
(atau folder di bawahnya). ShellBag tidak menunjukkan apa yang dilakukan di dalam folder, dan tidak memberi tanggal
pada setiap kunjungan: lihat waktu-waktu di bawah ini.

## Lokasi { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Key `BagMRU` menyimpan nilai-nilai bernomor, masing-masing satu shell item, dan nilai `MRUListEx` yang memberikan
urutannya, yang terakhir dibuka terlebih dahulu. Setiap nilai memiliki subkey dengan nama yang sama untuk item-item
di bawahnya, sehingga sebuah path adalah item-item dari `BagMRU` hingga sebuah nilai. `NodeSlot` sebuah subkey adalah
nomor key `Bags\<N>`-nya, tempat pengaturan tampilan folder berada.

Kumpulkan `UsrClass.dat` dan `NTUSER.DAT` beserta log transaksinya (`.LOG1`, `.LOG2`) untuk setiap
profil, serta hive `SOFTWARE` milik sistem, yang memberikan SID pengguna sebuah `NTUSER.DAT`.

## Waktu mana yang menyatakan apa { #which-times-say-what }

- **Waktu shell item** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) adalah
  waktu sistem berkas milik folder itu sendiri (presisi FAT: 2 detik) ketika Explorer menulis item tersebut. Waktu ini
  memberi tanggal pada folder, bukan kunjungan, dan tidak diperbarui pada kunjungan berikutnya.
  Jam FAT tidak memiliki offset UTC yang tersimpan. Judul `UTC` lawas dan akhiran `Z` yang ditampilkan adalah
  konvensi tampilan, bukan bukti zona waktu sumber. `auto-parse --timezone` mempertahankan
  field-field ini; interpretasikan dengan konteks sistem sumber.
- **Waktu last-write sebuah key** berubah ketika `MRUListEx`-nya ditulis ulang, yaitu ketika salah satu
  itemnya dibuka dan dipindahkan ke depan. Jadi, waktu key tempat nilai sebuah item berada
  (`ParentKeyLastWrittenUTC`) adalah saat item tersebut terakhir dibuka **hanya jika item itu adalah item terbaru
  di key tersebut** (`IsMostRecentInParent` bernilai true, `MruPosition` 0). Untuk item lainnya, itu adalah
  waktu yang lebih belakangan, ketika item saudaranya dibuka.
- Key milik item itu sendiri (`KeyLastWrittenUTC`) memberi tanggal pada pembukaan child terbarunya dengan cara
  yang sama.

## Peringatan bagi analis { #analyst-cautions }

- **Ketiadaan membuktikan sedikit.** Explorer menulis ShellBags ketika tampilan sebuah folder disimpan; folder yang
  hanya dilihat sebentar, atau dibuka dengan cara lain (baris perintah, skrip, program), mungkin tidak memilikinya.
  Pengguna dapat menghapus key-key tersebut.
- **Referensi NTFS adalah milik folder** (`MftEntry`, `MftSequence`) ketika item ditulis.
  Cari di `$MFT` untuk melihat apakah folder yang sama masih ada, atau berkas lain telah menggunakan kembali
  entri tersebut.
- **Drive lepas pasang** dinamai berdasarkan huruf drive-nya. Huruf tersebut adalah huruf yang dimiliki drive pada saat itu;
  cocokkan dengan riwayat USB dan perangkat yang di-mount untuk mengetahui perangkatnya.
- **Nama berasal dari item.** Folder yang kemudian diubah namanya tetap memiliki nama lamanya di sini. Folder root dan known
  folder dinamai sebagaimana alat forensik umumnya menamainya (`This PC`, `Quick Access`, `Downloads`);
  `Guid` memberikan pengenalnya.
- **Hasil pencarian, library, dan halaman Control Panel** adalah tampilan, bukan folder di disk
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Item yang tidak dikenal parser mana pun** menyimpan byte-nya di `DataRaw`, dengan `ItemType` `unknown` dan
  byte kelasnya di `ItemClass`. Item yang rusak atau tidak biasa dilaporkan di `Problems`, dan byte-nya
  juga disimpan.
- **Hive yang dirty** mungkin tidak memiliki item terbarunya tanpa log-nya. Juntei me-replay log yang berada di samping
  hive kecuali `--no-logs`; `TransactionLogStatus` menunjukkan apakah sebuah record berasal dari log tersebut.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell item didekode dengan ys-lnk-parser, parser shell item yang digunakan oleh `parse-file lnk`.
Ringkasan mencantumkan folder yang terakhir dibuka (terbaru terlebih dahulu), berdasarkan waktu last-write key parent-nya,
beserta penggunanya.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host, nama folder profil, SID pengguna (nama root key `UsrClass.dat` menyebutkannya; untuk `NTUSER.DAT`, `ProfileList` milik sistem), hive, dan tipenya. |
| `BagKey` | Key item, di bawah root hive (`…\BagMRU\4\3\0`): nilainya berada di key di atasnya. |
| `MRU-Position`, `IsMostRecentInParent` | Posisinya dalam `MRUListEx` milik key parent-nya (0: yang terakhir dibuka); kosong bila daftar tidak memilikinya. |
| `NodeSlot` | Nomor key `Bags\<N>`-nya (pengaturan tampilan folder). |
| `Path` | Item-item dari puncak pohonnya hingga item tersebut, digabungkan dengan `\`. `[unknown]` mewakili item yang nilainya hilang. |
| `ItemType`, `ItemClass` | Jenis item (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`), dan byte kelasnya. |
| `ItemName`, `GUID` | Namanya (nama panjang folder), dan pengenal kelas dari root folder, item Control Panel, atau delegate folder. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Dari shell item:** waktu folder ketika item ditulis (FAT, presisi 2 detik). |
| `MFT-Entry`, `MFT-Sequence` | **Dari shell item:** referensi berkas NTFS milik folder. |
| `ExtensionBlocks` | Jumlah extension block yang dimiliki item. |
| `KeyLastWrittenUTC` | **Dari registry:** last write dari key milik item itu sendiri: saat child terbarunya dibuka. |
| `ParentKeyLastWrittenUTC` | **Dari registry:** last write dari key tempat nilainya berada: saat item terakhir dibuka, jika `IsMostRecentInParent` bernilai true. |
| `DataRaw` | Byte nilai (hex), untuk item yang tidak sepenuhnya dipahami atau memiliki masalah. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai yang disebutkan oleh `MRUListEx` key-nya tetapi tidak dimiliki key tersebut, nilai yang tidak disebutkan oleh daftar, subkey tanpa nilainya, nilai yang bukan biner, `NodeSlot` dengan tipe yang salah, daftar shell item yang rusak. |

## Strategi investigasi { #investigation-strategy }

1. Daftarkan drive lepas pasang (`volume` dengan `Guid` Removable Drives), lokasi jaringan, folder
   FTP, dan perangkat portabel, serta folder-folder di bawahnya.
2. Cari folder yang menarik: lokasi staging yang dapat ditulis pengguna (`Temp`, `ProgramData`, `Public`),
   folder alat dan arsip, profil pengguna lain, dan folder yang sudah tidak ada (periksa
   referensi NTFS di `$MFT`).
3. Beri tanggal pada kunjungan hanya dengan `ParentKeyLastWrittenUTC` dari item yang `IsMostRecentInParent`-nya bernilai true;
   korelasikan sisanya dengan berkas LNK, Jump Lists, dan RecentDocs, yang memberi tanggal pada pembukaan berkas.
4. Gunakan `parse-registry read --recover-deleted` untuk key dan nilai `BagMRU` yang dihapus.

## Detail lebih lanjut { #further-details }

- [libyal libfwsi: Format Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

Pencarian SID dari `ProfileList` memerlukan satu SID yang unik dan valid secara sintaksis. Pemetaan folder profil yang ambigu dan key cadangan seperti `S-1-...bak` membuat `UserSid` tidak diketahui.
