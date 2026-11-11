# Amcache { #amcache }

## Apa itu Amcache { #what-amcache-is }

Amcache adalah inventaris kompatibilitas aplikasi Windows yang disimpan dalam sebuah hive registry. Sistem modern
mencatat berkas executable di bawah `InventoryApplicationFile` dan perangkat Plug-and-Play di bawah
`InventoryDevicePnp`; versi Windows yang lebih lama menggunakan tata letak lawas `Root\File`.

Untuk investigasi executable, Amcache bernilai karena dapat mempertahankan path lengkap, SHA-1,
ukuran berkas, metadata PE, informasi versi, teks penerbit, tanggal link, dan timestamp registry
setelah berkas aslinya hilang. Catatan perangkat dapat mengidentifikasi perangkat keras yang terpasang, driver,
tanggal instalasi, dan pengenal.

<span class="evidence-strength">Kesimpulan terbaik:</span> Windows menginventarisasi berkas atau
perangkat yang tercatat. Catatan executable adalah **bukti keberadaan, bukan bukti eksekusi**.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Kumpulkan hive beserta berkas di sebelahnya: `Amcache.hve.LOG`, `.LOG1`, dan `.LOG2`. Jika hive berstatus dirty, `parse-registry amcache` otomatis memutar ulang log transaksi yang dapat digunakan di memori sebelum mengurai, tanpa mengubah bukti. Rincian pemulihan dicatat dalam log parser atau terminal; saat rekaman dikirim ke stdout, diagnostik dikirim ke stderr. Log yang hilang atau tidak dapat digunakan dapat menyebabkan rekaman tidak lengkap atau hive yang tidak konsisten tidak dapat dibaca.

## Peringatan bagi analis { #analyst-cautions }

- Entri executable Amcache tidak membuktikan bahwa pengguna meluncurkan berkas tersebut.
- `Publisher` adalah metadata PE/version-resource yang dicatat oleh Windows. Ini tidak membuktikan bahwa berkas tersebut memiliki
  tanda tangan Authenticode yang valid pada saat pengumpulan.
- Hash mungkin tidak ada, dan skema lawas/modern tidak menyajikan nilai yang identik.
- `LinkDate` berasal dari header PE dan dapat dipalsukan, disalin, atau dihasilkan oleh reproducible build.
- Waktu last-write kunci mendeskripsikan pembaruan catatan registry, belum tentu unduhan, pembuatan,
  eksekusi pertama, atau eksekusi terakhir.
- Path dapat merujuk ke volume lepas pasang atau volume yang sudah tidak ada.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

Keluaran CSV berisi tabel ringkasan, temuan, berkas, dan perangkat. Dengan `auto-parse`, tabel-tabel ini adalah
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv`, dan
`Registry-Amcache-Devices.csv` langsung di bawah direktori `FileExistence/` milik setiap komputer, dengan
`Registry-Amcache-logs.txt` di samping tabel-tabel tersebut. Beberapa hive menambahkan `-Source1`, `-Source2`, dst.
sebelum ekstensi; `OutputFiles` di manifest mencantumkan path yang persis. Ekspor CSV mandiri
mempertahankan nama tabel aslinya dan lokasi keluaran yang diminta. Catatan JSONL menggunakan
nilai `RecordType` `summary`, `finding`, `file`, dan `device`.

## Field keluaran berkas { #file-output-fields }

| Field | Cara analis sebaiknya menggunakannya |
|---|---|
| `KeyName`, `SourceKeyPath` | Mengidentifikasi catatan registry sumber yang persis dan cabang skemanya. Pertahankan untuk reprodusibilitas. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Waktu pembaruan catatan registry, dalam bentuk mudah dibaca dan mentah. Jangan memberinya label waktu eksekusi. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Waktu kunci volume induk pada skema lawas; bersifat kontekstual, bukan waktu tindakan terhadap berkas. |
| `FileID` | Pengenal berkas yang dicatat oleh Amcache. Dipertahankan persis apa adanya; jangan berasumsi bahwa nilainya selalu SHA-1. |
| `SHA1`, `SHA256` | Pivot yang kuat ke Defender, threat intelligence, EDR, dan repositori berkas. Null berarti tidak tercatat. |
| `Name`, `OriginalFileName`, `Description` | Nama PE/versi yang tercatat. Bandingkan dengan path sebenarnya untuk mendeteksi penyamaran (masquerading); metadata dikendalikan penyerang. |
| `FullPath` | Pivot lokasi yang paling berguna. Normalisasikan huruf besar/kecil dengan hati-hati tetapi pertahankan string bukti aslinya. |
| `LongPathHash` | Pengenal terkait path milik Windows; berguna untuk mencocokkan catatan, bukan hash konten. |
| `ProgramID` | Menautkan berkas ke catatan inventaris aplikasi bila tersedia. |
| `Publisher` | Teks penerbit yang tercatat. Perlakukan sebagai metadata, bukan validasi tanda tangan. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Konteks version-resource. Perbedaan atau nilai yang tidak masuk akal dapat mendukung analisis penyamaran. |
| `BinaryType` | Konteks arsitektur/tipe yang tercatat; bandingkan dengan analisis PE bila berkas berhasil dipulihkan. |
| `LinkDate` | Timestamp kompilasi/link PE sebagaimana tercatat. Hanya berguna jika disikapi dengan skeptis dan dikorroborasi. |
| `Size` | Ukuran yang tercatat; bandingkan dengan `$MFT`, konten Defender, atau berkas yang dipulihkan. |
| `Language` | Pengenal bahasa PE; petunjuk kontekstual yang lemah, bukan atribusi. |
| `IsPE-File`, `IsOS-Component` | Flag klasifikasi yang dicatat oleh Windows. Nilai false tidak membuktikan bahwa berkas tidak berbahaya. |
| `USN` | Nilai NTFS update sequence yang dicatat oleh Amcache; dapat membantu pivot ke catatan USN yang masih tersisa. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Mengaitkan aplikasi/komponen terpaket dengan berkas tersebut. |
| `RegistryValues` | Setiap nilai registry asli beserta tipe, nilai terdekode, dan hex mentah. Gunakan bila field bertipe tidak ada atau untuk field masa depan/lawas. |

## Field keluaran perangkat { #device-output-fields }

| Kelompok field | Field dan interpretasi |
|---|---|
| Sumber | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: catatan perangkat yang persis dan waktu pembaruan registry. |
| Identitas | `Description`, `BusDescription`, `Manufacturer`, `Model`: deskripsi perangkat yang ditampilkan kepada pengguna; string vendor bisa bersifat generik. |
| Pengenal perangkat keras | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: pivot keluarga perangkat dan instans yang paling kuat. |
| Status | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: konteks enumerasi dan hubungan di Windows. |
| Driver | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: asosiasi paket driver dan layanan; korelasikan dengan berkas dan layanan yang terinstal. |
| Instalasi | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: siklus hidup instalasi yang tercatat. Tanggal yang tidak ada bukan bukti bahwa perangkat tidak pernah terpasang. |
| Filter dan stack | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: konteks driver stack tingkat lanjut; filter driver yang tidak terduga perlu ditinjau. |
| Pelestarian data mentah | `RegistryValues`: nilai sumber bertipe/mentah yang lengkap untuk validasi dan field yang tidak didukung. |

Kolom array dan nilai mentah di CSV menggunakan sufiks `JSON` (misalnya, `HWID-JSON` dan
`RegistryValuesJSON`); selnya berisi JSON, sedangkan JSON/JSONL mempertahankan array asli.

## Strategi investigasi { #investigation-strategy }

1. Lakukan pivot pada `FullPath`, `SHA1`, `Name`, dan `USN`.
2. Cari Prefetch yang cocok untuk mendukung adanya eksekusi.
3. Gunakan `$MFT` dan Jurnal USN untuk menetapkan pembuatan, penggantian nama, penghapusan, dan ukuran berkas.
4. Bandingkan metadata penerbit/versi dengan PE yang dipulihkan secara independen dan validasi tanda tangan.
5. Periksa karantina Defender dan log peristiwa untuk deteksi atau penghapusan.

## Detail lebih lanjut { #further-details }

- [Skema keluaran ys-amcache-parser milik Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [AmcacheParser oleh Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Plugin Amcache Regipy](https://github.com/mkorman90/regipy)
