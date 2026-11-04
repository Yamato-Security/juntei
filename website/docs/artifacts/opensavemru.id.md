# Dialog Buka dan Simpan Sebagai (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Apa itu riwayat dialog { #what-the-dialog-history-is }

Ketika sebuah program menampilkan dialog umum Open atau Save As, Windows mencatat di
`NTUSER.DAT` milik pengguna:

- **berkas mana yang dibuka atau disimpan:** `OpenSavePidlMRU` memiliki satu subkey per ekstensi (`docx`,
  `ps1`, …) dan `*` untuk semua ekstensi, masing-masing berupa daftar most-recently-used;
- **program mana yang menggunakan dialog, dan folder tempat dialog itu berada:** `LastVisitedPidlMRU` (dan
  `LastVisitedPidlMRULegacy` untuk program yang menggunakan dialog versi lama);
- **program yang menampilkan dialog:** `CIDSizeMRU` (ukuran dialog per program), dan
  `FirstFolder` (path program dan sebuah folder);
- pada Windows 2000 dan XP, `OpenSaveMRU` (path per ekstensi) dan `LastVisitedMRU` (program dan
  foldernya sebagai path).

Sejak Windows Vista, berkas dan folder disimpan sebagai shell item list, struktur yang sama dengan target
sebuah pintasan: setiap item merupakan bagian dari path (This PC, sebuah drive, sebuah folder, berkasnya), dan
item berkas atau folder dapat membawa waktu dan referensi berkas NTFS-nya sendiri.

<span class="evidence-strength">Kesimpulan terbaik:</span> sebuah program menggunakan dialog Open atau Save As
untuk membuka atau menyimpan berkas ini, atau di folder ini, dalam sesi pengguna ini. Hanya entri
terbaru dari setiap daftar yang memiliki tanggal: waktu penulisan terakhir key-nya.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Kumpulkan `NTUSER.DAT` beserta log transaksinya untuk setiap profil.

## Peringatan bagi analis { #analyst-cautions }

- **Satu waktu per daftar.** Waktu penulisan terakhir sebuah key berubah ketika salah satu entrinya berpindah ke
  depan daftarnya, sehingga waktu itu hanya menandai entri di posisi 0 (`IsMostRecent`, `LastUsedUTC`).
  Entri lainnya berurutan, dari terbaru ke terlama menurun dalam daftar, tetapi tidak bertanggal.
- **Berkas yang sama di dua daftar.** Sebuah berkas ada di daftar ekstensinya dan di `*`; jangan menghitungnya
  sebagai dua kali penggunaan. `*` memberikan urutan lintas ekstensi.
- **Dibuka atau disimpan.** Daftar-daftar ini tidak menyatakan yang mana. Korelasikan dengan record `$MFT` dan Jurnal USN
  milik berkas, berkas LNK, dan jump list.
- **Waktu item adalah waktu berkas, bukan waktu dialog.** `ItemModifiedUTC`, `ItemCreatedUTC`, dan
  `ItemAccessedUTC` adalah waktu berkas saat item ditulis, dengan presisi 2 detik (FAT). Item dari
  known folder (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) sering kali tidak memilikinya. Timestamp FAT ini
  tidak memiliki offset zona waktu. Sufiks kolom `UTC` adalah konvensi tampilan, bukan
  bukti bahwa jam aslinya menggunakan UTC; korelasikan dengan zona waktu sistem sumber dan
  timestamp lain sebelum membandingkannya dengan FILETIME registry. Lihat
  [panduan file time](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) dari Microsoft.
- **Path adalah shell path.** Path dimulai dari desktop: `This PC\C:\Users\…`, sebuah known folder, sebuah
  library (`UsersLibraries\Documents\…`), atau lokasi jaringan. Path yang dimulai dengan folder
  atau berkas berada di desktop. Path kosong adalah desktop itu sendiri.
- **Item yang tidak didekode.** Item dengan tipe yang tidak dikenal oleh parser shell item ditampilkan berdasarkan
  kelasnya (`[class 0x00]`), tipenya `unknown`, dan byte daftarnya ada di `DataRaw`.
- **Program berupa nama.** `LastVisitedPidlMRU` dan `CIDSizeMRU` menyimpan nama berkas
  (`WINWORD.EXE`), bukan path. `FirstFolder` menyimpan path.
- **Daftar pendek dan dibersihkan.** Setiap daftar menyimpan entri dalam jumlah terbatas, dan alat pembersih
  menghapus key-nya. Gunakan `parse-registry read --recover-deleted` untuk nilai yang telah dihapus.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Ringkasan mencantumkan entri per daftar, program yang menggunakan dialog, dan berkas
terbaru setiap hive (entri pertama dari `*`).

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, pemilik profil (dari path atau shell folder), SID dari `ProfileList` sistem, dan hive. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU`, atau `LastVisitedMRU`. |
| `Extension` | Subkey ekstensi (`docx`, `*`); kosong untuk daftar lainnya. |
| `MRU-Position`, `IsMostRecent` | Posisi di daftar key (0 adalah yang terbaru); kosong jika daftar tidak menyebutkan nilai tersebut. |
| `ValueName` | Nomor nilai (`MRUListEx`) atau hurufnya (`MRUList`). |
| `Program` | Program yang menampilkan dialog. |
| `Path` | Berkas atau folder: nama-nama shell item yang digabungkan, atau path yang disimpan sebagai teks. |
| `ItemType` | Tipe shell item terakhir: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Waktu berkas atau folder, sebagaimana dicatat oleh shell item-nya. |
| `MFT-Entry`, `MFT-Sequence` | Referensi berkas NTFS yang dicatat oleh shell item. |
| `LastUsedUTC` | Waktu penulisan terakhir key, hanya untuk entri terbaru. |
| `KeyPath`, `KeyLastWrittenUTC` | Key dan waktu penulisan terakhirnya: waktu entri terbarunya, bukan waktu setiap baris. |
| `DataRaw` | Byte nilai (hex) jika tidak dapat didekode, atau shell item list jika tipe sebuah item tidak dikenal. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai yang disebutkan daftar tetapi tidak ada di key, nilai yang tidak disebutkan daftar, tipe yang salah, nama atau daftar yang terpotong. |

## Strategi investigasi { #investigation-strategy }

1. Baca `*` secara berurutan per pengguna, dan `LastUsedUTC` untuk berkas terbaru.
2. Cari skrip, arsip, executable, dan dokumen di lokasi yang dapat ditulisi pengguna, removable, atau
   jaringan, serta berkas yang namanya menyerupai kredensial atau hasil ekspor.
3. Gunakan `LastVisitedPidlMRU` untuk mengaitkan sebuah program dengan folder yang terakhir digunakannya, serta `CIDSizeMRU` dan
   `FirstFolder` untuk program yang pernah menggunakan dialog sama sekali.
4. Korelasikan path dan `MftEntry` dengan `$MFT`, Jurnal USN, berkas LNK, jump list,
   ShellBags, dan RecentDocs.

## Detail lebih lanjut { #further-details }

- [libyal winreg-kb: Key most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Format Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
