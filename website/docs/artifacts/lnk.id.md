# Pintasan LNK { #lnk-shortcuts }

## Apa itu berkas LNK { #what-lnk-files-are }

Berkas Windows Shell Link (`.lnk`) mendeskripsikan sebuah target beserta argumen peluncuran, direktori kerja, ikon,
status jendela, metadata target, informasi volume/jaringan, item shell namespace, dan pengenal
distributed-link-tracking opsional. Windows dan aplikasi membuatnya di Recent Items,
Desktop, Start Menu, dan lokasi lain; penyerang juga mengirimkan pintasan berbahaya secara langsung.

LNK di folder Recent dapat mendukung adanya interaksi pengguna dengan target yang sudah tidak ada lagi, termasuk
drive lepas pasang atau network share. LNK sembarang dapat ada tanpa pernah diklik.

<span class="evidence-strength">Kesimpulan terbaik:</span> pintasan dibuat/diperbarui dengan
konteks target yang tercatat. Lokasinya dan artefak di sekitarnya menentukan apakah pintasan tersebut mendukung adanya penggunaan.

## Lokasi dan pengumpulan { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Telusuri seluruh profil pengguna serta arsip dan konten email yang dikumpulkan, bukan hanya `Recent`. Pertahankan
waktu dan path milik berkas sumber itu sendiri karena lokasi mengubah interpretasi.

## Peringatan bagi analis { #analyst-cautions }

- Waktu target, ukuran, referensi MFT, dan data volume adalah snapshot yang disimpan di dalam LNK; semuanya bukan
  metadata milik berkas pintasan itu sendiri dan bisa jadi sudah usang.
- `MachineID`, MAC, dan waktu pembuatan tracker mendeskripsikan konteks distributed-link-tracking, umumnya
  mesin yang membuat/memperbarui tautan—belum tentu mesin tempat Anda menemukannya.
- Node UUID yang menyerupai MAC dapat berasal dari virtualisasi, randomisasi, atau pengenal non-jaringan.
- `Args` dapat membuat executable yang tidak berbahaya (`powershell.exe`, `rundll32.exe`, `mshta.exe`) menjadi berbahaya.
- String ANSI memerlukan code page sistem sumber; gunakan `--code-page 932` untuk banyak tautan berbahasa Jepang.
- Pintasan dapat direkayasa atau disalin. Jangan menyamakan field-fieldnya dengan eksekusi tanpa dukungan dari Prefetch,
  Jump List, RecentDocs, log peristiwa, atau sumber lain.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## Field keluaran CSV { #csv-output-fields }

| Kelompok field | Field dan interpretasi |
|---|---|
| Sumber | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: path pintasan dan waktu dari sistem berkas host. Waktu-waktu ini terpisah dari waktu target. |
| Waktu target | `TargetCreated`, `TargetModified`, `TargetAccessed`: snapshot yang dicatat di dalam LNK; petunjuk historis yang berguna, bukan metadata target saat ini. |
| Dasar target | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: konteks target/peluncuran yang tercatat. Argumen dapat mengubah makna. |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: mengidentifikasi konteks tetap/lepas pasang/jaringan dan mengorelasikan pintasan lain. |
| Path | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: representasi path yang saling melengkapi. Tinjau perbedaan di antaranya serta path shell namespace. |
| Perilaku peluncuran | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: cara shell meluncurkan atau mengonfigurasi target. `Args` sangat penting. |
| Identitas NTFS | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: referensi target saat tautan dibuat; cocokkan keduanya dengan `$MFT`. |
| Pelacakan | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: distributed-link tracking. Gunakan sebagai korelasi, bukan satu-satunya dasar atribusi. |
| Blok tambahan | `ExtraBlocksPresent`: daftar struktur opsional yang tersedia; menjelaskan field lain mana yang didukung. |
| Tampilan | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: data tampilan dan data target yang variabel lingkungannya telah diekspansi. Penyerang dapat menggunakan nama/ikon yang menyesatkan. |
| Folder shell | `KnownFolder`, `SpecialFolder`: pengenal folder shell Windows yang telah didekode. |
| Jaringan | `NetworkDevice`, `NetworkProvider`: konteks network share/perangkat dan penyedia. |
| Encoding | `ConsoleCodePage`: code page konsol yang tercatat bila ada; terpisah dari `--code-page` milik parser. |
| Data di bagian akhir | `AppendedBytes`: byte setelah akhir logis. Data yang tidak terduga dapat mengindikasikan penyelundupan atau berkas yang cacat. |
| Validasi | `Problems`: kerusakan, pemotongan, blok yang tidak konsisten, atau keterbatasan dekode. Sertakan dalam peninjauan. |

JSON/JSONL juga mempertahankan setiap item target-ID (jenis, nama, GUID, waktu FAT, dan referensi
MFT), semua blok extra-data, properti property-store, pengaturan konsol, dan hubungan struktural
mentah. Gunakan format ini untuk pintasan yang mencurigakan alih-alih hanya mengandalkan baris CSV datar.
Waktu `TargetItems` yang bersarang berasal dari jam shell-item FAT tanpa offset yang tersimpan.
Tampilan `Z` lawasnya tidak membuktikan UTC; `auto-parse --timezone` mempertahankannya apa adanya.
FILETIME target dan waktu tracker di header LNK tetap merupakan timestamp absolut yang dapat dikonversi.

## Strategi investigasi { #investigation-strategy }

1. Periksa lokasi sumber dan waktu sumber sebelum menginterpretasikan field target.
2. Rekonstruksi peluncuran efektif dari path target, `Args`, `WorkingDirectory`, dan blok lingkungan.
3. Lakukan pivot pada referensi MFT target, nomor seri volume, path jaringan, droid, MachineID, dan MAC di seluruh bukti.
4. Periksa Prefetch dan log peristiwa untuk eksekusi; periksa Jump Lists/RecentDocs untuk interaksi pengguna.
5. Untuk lampiran berbahaya, pertahankan data yang ditambahkan dan lakukan hash/analisis terhadap LNK itu sendiri.

## Detail lebih lanjut { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [LECmd oleh Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
