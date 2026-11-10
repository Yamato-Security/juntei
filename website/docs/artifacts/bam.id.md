# BAM / DAM { #bam-and-dam }

## Apa itu BAM dan DAM { #what-bam-and-dam-are }

Background Activity Moderator (BAM) dan Desktop Activity Moderator (DAM) adalah layanan Windows
(Windows 10 1709 dan yang lebih baru) yang mengatur apa yang boleh dilakukan aplikasi di latar belakang.
Keduanya menyimpan, per pengguna, waktu terakhir setiap program berjalan, di hive `SYSTEM`:

- satu key per pengguna, dinamai sesuai SID pengguna;
- satu nilai per program: namanya adalah path program pada perangkatnya
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) atau ID aplikasi terpaket
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- 8 byte pertama data nilai adalah waktu terakhir program tersebut berjalan (FILETIME).

BAM mengaitkan sebuah program dengan pengguna dan waktu tanpa memerlukan hive milik pengguna itu sendiri, dan BAM mencantumkan program yang dijalankan
dari perangkat lepas pasang dan perangkat jaringan.

<span class="evidence-strength">Kesimpulan terbaik:</span> sebuah proses dari program ini berjalan dalam sesi
pengguna ini; terakhir berjalan pada waktu ini. BAM tidak menghitung jumlah eksekusi, dan sebuah entri dapat tetap ada setelah
program dihapus.

## Lokasi { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

Key setiap pengguna juga memiliki nilai `Version` dan `SequenceNumber`; keduanya bukan program dan
diabaikan. Sistem yang di-upgrade dari 1709 atau 1803 dapat memiliki kedua tata letak.

Kumpulkan `SYSTEM` beserta log transaksinya, serta `SAM` dan `SOFTWARE` milik sistem: keduanya menyebutkan nama
pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Satu waktu per program dan pengguna.** Hanya eksekusi terbaru yang disimpan. Prefetch, Amcache, dan
  log peristiwa memberikan eksekusi sebelumnya.
- **Aktivitas terbaru.** Entri program yang sudah lama tidak berjalan dilaporkan
  dihapus ketika Windows dimulai, sehingga BAM sebagian besar menunjukkan aktivitas terbaru: program yang tidak ada di dalamnya
  mungkin saja pernah berjalan jauh sebelumnya.
- **Path perangkat, bukan huruf drive.** `\Device\HarddiskVolume3` adalah volume ketiga yang dinomori
  Windows, bukan `C:`. `DevicePath` dan `Path` memisahkan perangkat dari path di dalamnya; cocokkan
  volume dengan `MountedDevices`, `$MFT`, atau artefak lain sebelum menuliskan huruf drive.
- **Tidak semua program.** BAM dan DAM melacak program dalam sesi pengguna; layanan umumnya tidak
  muncul. Entri `S-1-5-18` biasanya merupakan proses setup dan layar sign-in
  (`FirstLogonAnim.exe`), dan entri `S-1-5-90-0-<n>` adalah Desktop Window Manager (`DWM-<n>`).
- **Control set.** Secara default, control set yang terakhir digunakan Windows yang dibaca. `--all-control-sets`
  membaca semuanya: control set yang lebih lama dapat menyimpan entri yang sudah tidak ada di control set saat ini.
- **Nama pengguna.** Nama akun lokal berasal dari `SAM` (SID komputer dan RID
  akun); nama akun domain berasal dari folder profilnya di `ProfileList`. `UserSource`
  menunjukkan sumbernya (`sam`, `profile_list`, `well_known`). Tanpa keduanya, hanya SID yang diberikan. RID
  yang sudah tidak ada di `SAM` (akun yang dihapus) tetap tanpa nama.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Ringkasan mencantumkan setiap pengguna beserta SID, nama, dan jumlah entrinya, serta program yang terakhir dijalankan
(terbaru terlebih dahulu).

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Host, dan hive `SYSTEM`. |
| `ControlSet`, `IsCurrentControlSet` | Control set tempat entri berada, dan apakah itu control set yang terakhir digunakan Windows. |
| `Moderator` | `bam` atau `dam`. |
| `UserSID`, `User`, `UserSource` | SID yang menjadi nama key, nama pengguna, dan asal nama tersebut (`sam`, `profile_list`, `well_known`). |
| `Program` | Nama nilai sebagaimana ditulis Windows: path perangkat atau ID aplikasi terpaket. |
| `DevicePath`, `Path` | Untuk path perangkat, perangkatnya (`\Device\HarddiskVolume3`) dan path di dalamnya (`\Windows\System32\cmd.exe`); kosong untuk aplikasi terpaket. |
| `LastRunUTC` | Kapan program terakhir berjalan (hingga ketelitian 100 ns). |
| `KeyPath`, `KeyLastWrittenUTC` | Key pengguna dan last write-nya: pembaruan terbaru pada salah satu entrinya. |
| `DataRaw` | Byte nilai (hex), bila tidak berisi waktu: bukan `REG_BINARY`, atau kurang dari 8 byte. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Data yang tidak berisi waktu, dan nilai yang tidak dapat dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Urutkan berdasarkan `LastRunUTC` dan periksa program di lokasi yang dapat ditulis pengguna (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), di perangkat selain volume sistem, serta alat administrasi dan
   akses jarak jauh.
2. Periksa pengguna mana yang menjalankannya, dan akun yang tidak cocok dengan profil saat ini.
3. Korelasikan dengan Prefetch (jumlah eksekusi dan eksekusi sebelumnya), UserAssist (peluncuran oleh Explorer), Amcache
   dan ShimCache (keberadaan), `$MFT` (berkas program), serta log peristiwa.
4. Gunakan `--all-control-sets`, dan `parse-registry read --recover-deleted` untuk nilai BAM yang telah dihapus.

## Detail lebih lanjut { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifier (SID well-known)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

FILETIME last-run yang berada di luar rentang membuat `LastRunUTC` kosong dan mempertahankan nilai aslinya di `DataRaw`, dengan penjelasan di `Problems`. Ringkasan aktivitas terbaru mengidentifikasi setiap host dan menjaga agar observasi hive/pengguna yang terpisah tetap berbeda.
