# Jump Lists { #jump-lists }

## Apa itu Jump Lists { #what-jump-lists-are }

Jump Lists menjadi dasar menu yang ditampilkan dari entri aplikasi di taskbar dan Start menu. Automatic Jump
Lists adalah riwayat item yang digunakan oleh sebuah aplikasi yang dikelola Windows. Custom Jump Lists adalah
kategori dan tugas yang didefinisikan aplikasi. Keduanya dapat menyematkan struktur LNK dengan path target, waktu,
konteks volume/jaringan, referensi MFT, dan argumen peluncuran.

Record otomatis dapat mengungkap berkas atau folder yang dibuka oleh sebuah aplikasi, urutan/waktu penggunaan terakhir,
jumlah interaksi, dan status penyematan (pin). Daftar kustom dapat memperlihatkan halaman “baru saja ditutup” atau tugas
aplikasi—termasuk argumen perintah berbahaya.

<span class="evidence-strength">Kesimpulan terbaik:</span> aplikasi pemilik mencatat atau menampilkan
item/tugas tersebut di Jump List-nya. Perilakunya berbeda antar aplikasi; dukung eksekusi/akses yang sebenarnya dengan bukti lain.

## Lokasi dan pengumpulan { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Prefiks heksadesimal pada nama berkas adalah AppID. Kumpulkan kedua direktori dan pertahankan path profil
pengguna. Daftar otomatis adalah kontainer OLE Compound File dengan stream `DestList` dan stream LNK
bernomor; daftar kustom menggunakan format kategori/tautan tersendiri.

## Peringatan bagi analis { #analyst-cautions }

- Deskripsi AppID berasal dari pemetaan yang sudah dikenal; AppID yang tidak dikenal tidak mencurigakan dengan sendirinya.
- Field `LastModified` / last-used milik entri DestList; waktu target yang disematkan adalah hal yang berbeda.
- Jumlah interaksi hanya ada pada versi yang didukung dan bukan jumlah pembukaan secara lengkap.
- Penyematan mempertahankan item melampaui batas kebaruan normal; pembersihan atau pengaturan aplikasi dapat menghapus item lain.
- Sebuah daftar dapat berisi stream yang usang/tidak terhubung atau byte dari daftar kustom sebelumnya yang lebih panjang.
- Argumen dan path LNK yang disematkan dapat dikendalikan penyerang. Asosiasi aplikasi adalah konteks, bukan dasar kepercayaan.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Dengan `auto-parse`, tabel CSV-nya adalah `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
dan `FileAndFolderOpening/JumpLists-CustomDestinations.csv`, langsung di bawah setiap komputer.
Log bersama adalah `JumpLists-logs.txt` di kategori tersebut. Manifest mencantumkan path yang tepat
di `OutputFiles`. Ekspor CSV mandiri mempertahankan `AutomaticDestinations.csv` dan
`CustomDestinations.csv` di direktori yang Anda pilih. Ekspor otomatis JSON dan JSONL berupa
satu berkas `JumpLists.json` atau `JumpLists.jsonl`; struktur record-nya tidak berubah.

## Field CSV AutomaticDestinations { #automaticdestinations-csv-fields }

| Kelompok field | Field dan interpretasi |
|---|---|
| Sumber | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: kontainer Jump List dan waktu sistem berkasnya. |
| Aplikasi | `AppId`, `AppIdDescription`: AppID dari nama berkas dan pemetaan aplikasi yang dikenal. Jangan menyimpulkan identitas dari hash yang tidak dikenal. |
| Kontainer | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: struktur/versi DestList dan konteks header. |
| Urutan | `MRU`, `EntryNumber`: posisi penggunaan terbaru dan pengenal entri/stream heksadesimal. |
| Metadata penggunaan | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: siklus hidup/penggunaan/status pin DestList. Interpretasikan sesuai versi. |
| Host/path | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: lokasi target dan konteks pelacakan. Host/MAC bukan satu-satunya dasar atribusi. |
| ID pelacakan | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: korelasikan target yang dipindahkan dan pelacakan LNK yang disematkan. |
| Catatan | `Notes`: memberi tahu Anda tentang stream yang tidak terhubung, sisa data yang dipulihkan, atau konteks parser lainnya. |
| LNK yang disematkan | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: interpretasikan seperti yang dijelaskan di [Pintasan LNK](lnk.md). |
| LNK tambahan | `NetworkPath`, `Name`, `IconLocation`: field jaringan dan tampilan dari pintasan yang disematkan. |
| Validasi | `Problems`: stream yang hilang, pemotongan, ketidaksesuaian hitungan, atau kerusakan tautan. |

## Field CSV CustomDestinations { #customdestinations-csv-fields }

Keluaran kustom diawali dengan `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription`, dan `EntryName` (nama kategori atau `Tasks`), diikuti oleh semua field LNK yang disematkan
yang tercantum di atas, lalu `Notes`, `NetworkPath`, `Name`, `IconLocation`, dan `Problems`.

JSON/JSONL juga mempertahankan header DestList, score/checksum/property store per entri,
`DestListPropertyStore`, stream compound-file, kategori kustom, byte setelah kategori terakhir,
dan field bersarang dari setiap LNK yang disematkan.

## Strategi investigasi { #investigation-strategy }

1. Identifikasi pengguna dan profil sumber, lalu petakan AppID ke aplikasi dengan hati-hati.
2. Urutkan entri otomatis berdasarkan `LastModified` dan `MRU`; pisahkan item yang disematkan.
3. Tinjau path, network share, nomor seri volume lepas pasang, referensi MFT, dan argumen.
4. Periksa `Notes` dan tautan yang tidak terhubung/sisa—item yang dihapus atau sebelumnya dapat bertahan di sana.
5. Korelasikan dengan RecentDocs, LNK Recent yang berdiri sendiri, Prefetch, SRUM, `$MFT`, dan log peristiwa.

## Detail lebih lanjut { #further-details }

- [Microsoft: Format Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Format Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JLECmd oleh Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [Proyek uji/referensi parser JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
