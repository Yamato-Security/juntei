# SRUM { #srum }

## Apa itu SRUM { #what-srum-is }

System Resource Usage Monitor (SRUM) menyimpan database ESE berisi penggunaan sumber daya aplikasi dan
layanan. Bergantung pada versi Windows, SRUM mencatat byte jaringan, pengenal antarmuka/profil,
koneksi, waktu CPU, aktivitas foreground/background, aktivitas disk, penggunaan energi, push
notification, dan data timeline aplikasi. Entri dikaitkan dengan ID aplikasi dan SID pengguna
melalui `SruDbIdMapTable`.

SRUM sangat berguna ketika sebuah executable telah dihapus: ID aplikasi, pengguna, dan
baris penggunaan sumber dayanya dapat bertahan lebih lama daripada berkasnya. Total jaringan dapat mendukung hipotesis eksfiltrasi, tetapi
SRUM bukan packet capture dan umumnya tidak mengidentifikasi setiap endpoint jarak jauh atau berkas yang ditransfer.

<span class="evidence-strength">Kesimpulan terbaik:</span> Windows mengatribusikan penggunaan sumber daya
yang tercatat kepada aplikasi/akun selama periode pengukuran tabel.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM ada di Windows 8 dan yang lebih baru. Salin `SRUDB.dat` secara konsisten, idealnya dari snapshot atau dengan
collector forensik. Kumpulkan juga `SOFTWARE` dan berkas transaksi/log ESE di direktori
`sru`.

Hive SOFTWARE yang sesuai terpisah dari basis data ESE: jika hive berstatus dirty, Juntei otomatis memutar ulang log transaksi Registry di sebelahnya, yaitu `SOFTWARE.LOG`, `.LOG1`, dan `.LOG2`, di memori. Kumpulkan juga berkas tersebut. Rincian pemulihan dan peringatan ditampilkan; tanpa log, nama akun atau Wi-Fi dapat tidak lengkap atau kedaluwarsa. Ini tidak memutar ulang log basis data SRUM.

!!! warning "Database dirty"
    Juntei membaca `SRUDB.dat` yang dirty apa adanya saat dikumpulkan dan melaporkannya. Saat ini Juntei tidak memutar ulang
    `SRU*.log`, sehingga perubahan yang hanya ada di log tersebut tidak akan muncul. Jangan diam-diam mendeskripsikan
    database yang dirty sebagai lengkap.

## Peringatan bagi analis { #analyst-cautions }

- Banyak baris merepresentasikan agregasi berkala, umumnya di sekitar batas per jam—bukan waktu tindakan yang tepat.
- Total byte saja tidak mengidentifikasi konten, niat, protokol, atau tujuan.
- Identitas aplikasi dapat berupa path, layanan, aplikasi terpaket, atau objek; periksa `AppType`.
- Nama pengguna dan Wi-Fi merupakan pengayaan dari hive `SOFTWARE` yang bersesuaian. Nama yang tidak ada tidak berarti
  ID yang mendasarinya tidak ada.
- `null`, nol, kolom yang tidak ada, tipe yang tidak didukung, dan nilai yang rusak merupakan kondisi yang berbeda.
- Skema provider bervariasi menurut versi Windows. Pertahankan tabel yang tidak dikenal dan nama kolom aslinya.
- Layanan Windows di latar belakang secara sah menghasilkan total sumber daya yang besar.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Tabel keluaran { #output-tables }

CSV membuat `SRUM-IdentityMap.csv` untuk `SruDbIdMapTable` dan satu tabel per provider. Nama keluaran yang dikenal antara lain:

| Provider/tabel | Tujuan investigasi |
|---|---|
| `NetworkUsages` | Byte yang dikirim/diterima per aplikasi, pengguna, antarmuka, dan profil. |
| `NetworkConnections` | Aktivitas berupa jumlah/durasi koneksi dan waktu mulai koneksi. |
| `AppResourceUseInfo` | Penggunaan CPU, disk, jaringan foreground/background, dan penggunaan aplikasi terkait. |
| `AppTimelineProvider` | Aktivitas timeline aplikasi dengan field waktu akhir dan kondisi penggunaan. |
| `EnergyUsage` | Pengukuran energi/provider jangka pendek dan jangka panjang; `IsLt` membedakannya. |
| `PushNotifications` | Aktivitas push notification. |
| `VfuProvider` | Provider `vfuprov`; tujuan terperincinya tidak terdokumentasi dengan baik, jadi interpretasikan field-fieldnya dengan hati-hati. |
| `EnergyEstimation`, `TaggedEnergy` | Provider energi dan penggunaan yang spesifik untuk versi tertentu. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Tabel provider Windows Server. |
| Nama tabel tidak dikenal | Juntei mempertahankan provider yang tidak dikenal dengan nama GUID/tabelnya dan mengeluarkan setiap kolom katalog. |

Karena SRUM digerakkan oleh katalog dan Windows menambahkan kolom provider, dengan sengaja tidak ada daftar
tetap field spesifik provider yang terbatas. Juntei mengeluarkan **setiap kolom yang ada di database dengan
label field PascalCase milik Juntei**. Field-field berikut adalah kontrak keluaran yang stabil/umum.

## Keluaran auto-parse yang dikategorikan { #categorized-auto-parse-output }

`auto-parse` menempatkan setiap tabel SRUM langsung di kategori investigasinya. CSV, JSON, dan JSONL
menggunakan prefiks nama berkas `SRUM-` yang sama; tidak ada direktori `SRUM/` pembungkus.

| Kategori | Berkas CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv`, dan tabel provider yang tidak dikenal |

Hanya tabel yang ada di hasil tangkapan yang diekspor. Penghitung jaringan dan record notifikasi
ditempatkan berdasarkan kegunaan investigasi utamanya; ini bukan klaim bahwa setiap baris mendeskripsikan
koneksi jarak jauh. Identity map mendukung beberapa kategori. VFU dan provider yang tidak dikenal tetap berada di
`SystemInformation` karena keberadaannya saja tidak menetapkan adanya eksekusi.

`SystemInformation/SRUM-logs.txt` berisi diagnostik parser dan pemetaan keluaran dari asli ke
akhir. Kedua manifest mencantumkan path tabel yang sebenarnya di `OutputFiles`; `Output` milik tugas SRUM
adalah direktori komputer dan `Category` mengidentifikasi lokasi log/metadata bersamanya.

Dengan beberapa database, nama berkas mendapat `-Source1`, `-Source2`, dst., sebelum ekstensinya.
Log mempertahankan alias tangkapan asli, dan baris JSON/JSONL mempertahankan provenans `SourceFile`,
`Table`, `Provider`, `Comp`, dan `Artifact`-nya. Tidak ada tabel yang disalin ke beberapa
kategori. Timeline SIEM gabungan `-j` menyertakan setiap record SRUM yang dipertahankan sebanyak satu kali.

`parse-ese srum` yang dijalankan secara mandiri mempertahankan lokasi keluaran yang diminta. Berkas tabel CSV-nya juga memiliki
prefiks `SRUM-`; nama berkas keluaran JSON/JSONL yang eksplisit tetap dipilih oleh analis.

## Field keluaran umum { #common-output-fields }

| Field | Interpretasi |
|---|---|
| `SourceFile` | Database input; ada di JSON/JSONL dan berguna lintas host. |
| `Table` | Nama/GUID tabel ESE asli. Pertahankan saat membandingkan versi atau provider yang tidak dikenal. |
| `Provider` | Nama provider/keluaran yang mudah dibaca, atau nama tabel asli jika tidak dikenal. |
| `ID` | `AutoIncId` baris; pengenal baris database, bukan ID proses. |
| `Timestamp` | Kapan SRUM mencatat/mengagregasi angka-angka tersebut, dibulatkan ke milidetik. Biasanya bukan waktu peristiwa yang tepat. |
| `EXE-Info` | Identitas aplikasi/layanan/paket yang di-resolve dari `AppID`. Titik pivot program utama. |
| `EXE-InfoDescription` | Deskripsi yang diekstrak dari nama aplikasi SRUM komposit. Hanya konteks. |
| `EXE-Timestamp` | Waktu yang tertanam dalam identitas aplikasi komposit jika ada; bukan waktu penggunaan baris tersebut. |
| `AppType` | `NormalApp`, `Service`, `ModernApp`, atau `Object`; menentukan bagaimana `EXE-Info` harus diinterpretasikan. |
| `SID` | SID akun yang di-resolve dari `UserID`. Titik pivot identitas pengguna/layanan yang kuat. |
| `User` | Nama profil yang mudah dibaca, diperkaya dari `SOFTWARE`; dapat bernilai null meskipun `SID` tetap valid. |
| `UserID`, `AppID` | Referensi numerik mentah ke `SruDbIdMapTable`; pertahankan untuk nama yang tidak ter-resolve atau duplikat. |
| `IsLt` | `true` untuk tabel EnergyUsage jangka panjang, `false` untuk jangka pendek. |
| `InterfaceLUID` | LUID antarmuka jaringan mentah dari baris provider. |
| `InterfaceType` | Kelas antarmuka yang didekode (misalnya `IF_TYPE_IEEE80211`); kelas, bukan adaptor yang unik. |
| `L2ProfileID` | Referensi profil jaringan numerik. |
| `ProfileName` | Nama Wi-Fi/profil yang diperkaya dari `SOFTWARE`; berpotensi sensitif dan bukan bukti koneksi saat ini. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Kolom FILETIME provider yang dikenal dan didekode ke UTC. Makna tepatnya bergantung pada provider, bukan waktu eksekusi universal. |
| Kolom spesifik provider | Kolom provider yang diekspor seperti `BytesSent`, `BytesReceived` (kolom database `BytesRecvd`), penghitung CPU/disk, durasi, jumlah, flag, atau estimasi. Interpretasikan satuannya sesuai provider dan bandingkan baris dalam skema yang sama. |

## Field diagnostik (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Field | Interpretasi |
|---|---|
| `ValueState` | Pemetaan field database ke `parsed`, `null`, `absent`, `unsupported`, atau `damaged`. Ini memisahkan ketiadaan bukti dari kegagalan parser. |
| `Raw` | Byte asli dalam heksadesimal untuk nilai yang memiliki byte. Gunakan untuk validasi dan dekode di masa mendatang. |

`SRUM-IdentityMap.csv` (dari `SruDbIdMapTable`) berisi `IdIndex`, `IdType`, dan `IdBlob`: indeks mentah, kategori identitas,
dan teks program/SID/objek yang didekode yang digunakan oleh baris provider.

## Strategi investigasi { #investigation-strategy }

1. Identifikasi program dengan `EXE-Info`, `AppType`, dan `AppID`; jangan menyamakan `Id` dengan PID.
2. Kelompokkan baris jaringan berdasarkan program, pengguna, profil, antarmuka, dan interval waktu.
3. Bandingkan byte yang dikirim/diterima dengan baseline normal dan host lain.
4. Korelasikan interval yang mencurigakan dengan Prefetch, log peristiwa, data DNS/proxy/firewall, dan akses berkas.
5. Laporkan status database yang dirty dan tidak adanya pemutaran ulang log transaksi sebagai keterbatasan pengumpulan.

## Detail lebih lanjut { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
