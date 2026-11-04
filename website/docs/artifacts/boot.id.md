# Sektor boot (`$Boot`) { #boot-sector-boot }

## Apa itu `$Boot` { #what-boot-is }

`$Boot` adalah entri MFT 7 pada setiap volume NTFS: 8 KiB pertama volume. Sektor pertamanya, yaitu
**volume boot record**, menyimpan BIOS parameter block yang mendeskripsikan sistem berkas:

- ukuran sektor dan cluster, jumlah sektor dalam volume, serta **hidden sectors**
  (sektor-sektor pada disk sebelum volume, yaitu tempat partisinya dimulai);
- tempat `$MFT` dan `$MFTMirr` dimulai, serta ukuran record FILE dan INDX;
- **nomor seri volume** 64-bit, yang ditetapkan saat volume diformat.

Sisa sektor pertama dan 15 sektor setelahnya menyimpan **boot code** yang memuat
Windows.

32 bit bawah dari nomor seri adalah yang ditampilkan Windows sebagai `XXXX-XXXX` (`vol`, `dir`) dan yang
dicatat oleh [pintasan LNK](lnk.md), [Jump Lists](jump-lists.md), dan [Prefetch](prefetch.md) tentang
volume tempat sebuah berkas berada. `$Boot` mengaitkan record-record tersebut dengan volume dari host yang dikumpulkan.

<span class="evidence-strength">Kesimpulan terbaik:</span> nomor seri volume pada LNK, Jump List, atau Prefetch
yang sama dengan `VolumeSerialNumber` milik `$Boot` menunjuk ke volume tersebut, atau ke volume lain yang
diberi, atau disalin dengan, nomor seri yang sama. Nomor seri dapat diubah dan tidak
unik di antara host.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Kumpulkan dengan alat forensik yang mendukung akses raw, atau baca sektor pertama dari image volume: keduanya adalah
byte yang sama. Juntei membaca berkas apa pun yang diawali dengan sektor boot NTFS, sehingga image volume dapat digunakan
sebagai input. Image disk justru diawali dengan tabel partisi; berikan volume partisi tersebut kepada Juntei.

## Peringatan bagi analis { #analyst-cautions }

- Nomor seri berubah ketika volume diformat. Berkas LNK yang nomor serinya tidak cocok dengan
  volume mana pun saat ini mungkin menunjuk ke media lepas pasang, server network share, atau volume yang diformat
  setelahnya.
- Boot code sama pada volume yang diformat oleh Windows yang sama: boot code identik pada berkas `$Boot`
  Windows 10, 11, dan Server 2012 R2 dari DFIR Artifact Museum. Bootkit volume boot record
  mengubahnya, atau mengubah field hidden sectors, yang digunakan boot code untuk menemukan sisa
  `$Boot`. Bandingkan `BootCodeSHA256`, `JumpInstruction`, dan `HiddenSectors` dengan host lain dan dengan tabel
  partisi sebelum menyimpulkan apa pun: boot manager atau perangkat lunak enkripsi juga dapat mengubah boot
  code.
- Image volume yang dienkripsi BitLocker diawali dengan sektor boot milik BitLocker sendiri (OEM ID
  `-FVE-FS-`). Bila diberikan dengan `-f`, Juntei melaporkannya, dan `Problems` menyatakan bahwa itu bukan NTFS; sektor boot
  NTFS berada di dalam data terenkripsi. `$Boot` yang dikumpulkan dari volume yang telah dibuka kuncinya adalah sektor boot NTFS.
- `Problems` mencantumkan inkonsistensi struktural dan pemotongan yang dikenali, bukan menyediakan pengujian lengkap apakah Windows dapat me-mount volume. Gunakan untuk menilai sektor hasil carving dan salinan, bukan sebagai temuan serangan dengan sendirinya. Hash boot code tidak mencakup jump awal, BIOS parameter block, dan signature akhir; periksa juga field-field tersebut.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

Ringkasan mencantumkan nomor seri volume setiap sektor boot (sebagaimana ditampilkan Windows, dan sebagaimana ditampilkan field LNK dan
Prefetch), ukuran, ukuran cluster, dan cluster `$MFT`. Bila beberapa host diurai, ringkasan
mengelompokkannya berdasarkan hash boot code, sehingga volume yang boot code-nya berbeda dari yang lain akan menonjol.

## Field keluaran CSV { #csv-output-fields }

| Field | Interpretasi |
|---|---|
| `SourcePath` | Berkas input. |
| `VolumeSerialNumber` | Nomor seri 32-bit, sebagaimana ditampilkan keluaran LNK, Jump List, dan Prefetch (`180B1E67`). Windows menampilkannya sebagai `180B-1E67`. |
| `VolumeSerialNumber64` | Nomor seri 64-bit secara utuh. |
| `OEM-ID` | `NTFS` pada volume NTFS. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Geometri volume. Sektor per cluster di atas 128 dicatat sebagai pangkat dua, yang didekode di sini. Kosong bila tidak valid. |
| `TotalSectors`, `VolumeSize` | Sektor dalam volume dan ukurannya dalam byte, tanpa salinan sektor boot di akhirnya. |
| `HiddenSectors` | Sektor pada disk sebelum volume: tempat partisinya dimulai. |
| `MFT-Cluster`, `MFT-Offset` | Tempat `$MFT` dimulai, dalam cluster dan dalam byte dari awal volume. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | Tempat `$MFTMirr` dimulai. |
| `FileRecordSize`, `IndexRecordSize` | Byte per record FILE (biasanya 1.024) dan per record INDX (biasanya 4.096). |
| `MediaDescriptor` | `F8` untuk hard disk. |
| `JumpInstruction` | Tiga byte pertama (`EB 52 90`): jump melewati BIOS parameter block menuju boot code. |
| `EndSignature` | `55 AA` di akhir sektor boot yang valid. |
| `BootCodeSHA256` | SHA-256 dari boot code: byte 0x54 hingga 0x1FD dari sektor pertama dan sisa 8 KiB. |
| `Problems` | Hal-hal yang membuat Windows tidak akan me-mount volume: OEM ID selain `NTFS`, tidak ada `55 AA`, ukuran tidak valid, field FAT yang dibiarkan nol oleh NTFS ternyata terisi, `$MFT` melewati akhir volume; serta berkas yang lebih pendek dari 8 KiB. |

JSON/JSONL memiliki field dan nama yang sama, dengan angka sebagai angka, `null` untuk nilai yang tidak diketahui, dan
`Problems` sebagai list.

## Strategi investigasi { #investigation-strategy }

1. Sandingkan `VolumeSerialNumber` setiap host dengan nomor seri volume pada berkas LNK, Jump
   Lists, dan berkas Prefetch-nya: yang tidak cocok dengan volume mana pun yang dikumpulkan adalah volume lain (drive
   USB, share, volume yang diformat setelahnya).
2. Cari nomor seri lainnya di keluaran [perangkat USB](usb.md), yang mencantumkan nomor seri volume
   dari volume lepas pasang yang pernah di-mount.
3. Bandingkan `BootCodeSHA256` dan `HiddenSectors` antar host. Investigasi boot code yang berbeda
   dengan disassembler dan tabel partisi disk, serta cari entri log peristiwa yang terkait dengan boot.
4. Gunakan `MFT-Offset` dan `ClusterSize` untuk menemukan `$MFT` di dalam image volume.

## Detail lebih lanjut { #further-details }

- [Microsoft: Ikhtisar NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: Dokumentasi format NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (header volume)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `boot`-nya)
