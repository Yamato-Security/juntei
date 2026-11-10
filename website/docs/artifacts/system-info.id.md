# Informasi sistem { #system-information }

## Apa itu { #what-it-is }

Empat hive mesin milik sebuah sistem menyatakan apa yang perlu diketahui analis sebelum membaca timeline-nya:

- nama-namanya (NetBIOS, nama host DNS, domain DNS) dan domain atau workgroup tempat sistem itu berada;
- versi, build, dan update revision Windows-nya, serta kapan Windows diinstal atau terakhir di-upgrade;
- zona waktunya, yang memberi tahu Anda waktu lokal yang digunakan sistem;
- kapan sistem terakhir kali dimatikan dengan bersih;
- arsitektur prosesor dan code page-nya;
- pengguna yang terakhir masuk, dan profil setiap pengguna yang pernah masuk;
- SID-nya: SID komputer (akun lokal adalah SID ini ditambah sebuah RID) dan SID domainnya;
- versi-versi Windows yang menjadi asal upgrade sistem tersebut.

`parse-registry system-info` menulis satu record per sistem yang berisi semua hal ini.

<span class="evidence-strength">Kesimpulan terbaik:</span> konfigurasi sebagaimana adanya saat hive-hive tersebut
dikumpulkan (atau terakhir ditulis). Waktu key menyatakan kapan bagian-bagiannya terakhir berubah.

## Lokasi { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Kumpulkan keempat hive beserta log transaksinya. Masing-masing hive saja sudah memberikan informasi yang disimpannya.

## Peringatan bagi analis { #analyst-cautions }

- **Waktu registry adalah UTC; zona waktu diperlukan untuk hal lainnya.** Waktu key, FILETIME, dan sebagian besar
  waktu peristiwa adalah UTC. Log teks, sebagian log aplikasi, dan tanggal yang ditulis di dalam nama atau string
  (seperti `Source OS (Updated on …)`) bisa berupa waktu lokal. `Bias` adalah selisih menit dari waktu standar lokal
  ke UTC (`UTC = local + Bias`). `UtcOffset` dan `DaylightUtcOffset` memberikan kedua offset tersebut,
  dan `TimeZoneDescription` menyatakan kapan daylight time dimulai dan berakhir.
- **Zona waktu dapat berubah.** `TimeZoneLastWrittenUTC` adalah waktu penulisan terakhir key. Windows juga
  menulis ulang key tersebut dengan sendirinya ketika memperbarui `ActiveTimeBias` (offset yang berlaku saat itu), sehingga
  waktu yang baru tidak membuktikan bahwa seseorang mengubah zona waktu. Bandingkan dengan log peristiwa: Security 4616
  (waktu sistem berubah) dan peristiwa `Microsoft-Windows-Kernel-General` di log System.
- **`StandardStart` dan `DaylightStart` adalah `TIME_FIELDS` milik kernel, bukan `SYSTEMTIME`.** Hari
  dalam seminggu adalah field terakhirnya, bukan field ketiga (seperti pada nilai `TZI` zona di `SOFTWARE`). Pada
  zona waktu AS, kedua tata letak terbaca sama, karena harinya adalah Minggu (0). Juntei mendekodenya
  sebagai `TIME_FIELDS`: sebuah sistem publik dengan zona waktu Pulau Paskah (Sabtu pertama pukul 22.00) menunjukkan
  perbedaannya. Bulan bernilai 0 berarti tidak ada daylight saving time.
- **Windows 11 menyebut dirinya "Windows 10".** `ProductName` tetap `Windows 10 …` di Windows 11. Build 22000
  dan yang lebih baru adalah Windows 11, dan ringkasan menyatakannya demikian. `CurrentVersion` bernilai `6.3` di Windows 8.1, 10,
  dan 11; `Version` menggunakan `CurrentMajorVersionNumber` (10) jika ada.
- **`InstallDate` berubah bersama feature upgrade.** Upgrade in-place ke rilis baru (misalnya 1809 ke
  21H2) mengatur ulang nilainya. Rilis yang digantikan oleh setiap upgrade, dengan `InstallDate`-nya sendiri, ada
  di `PreviousVersions` (`Setup\Source OS`). `InstallDate` paling awal di antaranya adalah yang paling mendekati
  instalasi asli. Tanggal di nama key `Source OS` tidak menyatakan zona waktunya.
- **`ShutdownTime` adalah shutdown bersih terakhir.** Crash atau kehilangan daya tidak memperbaruinya.
  Bandingkan dengan log peristiwa (6006, 6008, 1074) dan dengan waktu penulisan terakhir ke hive-hive tersebut.
- **`LastLoggedOnUser` adalah pengguna terakhir di layar sign-in,** bukan setiap logon. Remote Desktop,
  `runas`, layanan, dan tugas terjadwal tidak semuanya mengaturnya. `LogonUiLastWrittenUTC` kurang lebih menunjukkan kapan
  nilai itu terakhir diatur.
- **Profil:** `ProfileList` mencantumkan setiap pengguna (dan layanan) yang pernah memiliki profil di
  sistem, termasuk akun yang telah dihapus hingga profilnya dihapus. Windows 10 dan yang lebih baru menulis
  `LocalProfileLoadTime` dan `LocalProfileUnloadTime`: kurang lebih sign-in dan sign-out terakhir.
- **SID:** akun lokal adalah `MachineSid-<RID>`, akun domain `DomainSid-<RID>`.
  `MachineSid` berasal dari `SAM` (atau jika tidak ada, dari `PolAcDmS` milik `SECURITY`). Keduanya seharusnya sama;
  jika tidak, `Problems` menyatakannya, seperti yang terjadi pada hive dari sistem yang berbeda. `DomainSid` kosong
  di sebuah workgroup.
- **Satu direktori, satu sistem.** Dengan `-d`, hive-hive dalam satu direktori dianggap milik satu sistem.
  Salinan `RegBack` dan hive kedua dengan jenis yang sama (`SYSTEM.old`) menjadi record tersendiri.
  Dengan `-f`, hive-hive yang diberikan dianggap milik satu sistem, dan hive lainnya dicari di sebelahnya.
- **Control set:** hanya control set yang terakhir digunakan Windows yang dibaca. `parse-registry read` menampilkan
  control set lainnya.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Ringkasan memberikan, untuk setiap sistem: versi Windows-nya, kapan diinstal, zona waktunya dalam
kata-kata, offset yang berlaku saat key terakhir ditulis, shutdown terakhirnya, domain atau
workgroup-nya, pengguna terakhir yang masuk, SID-nya, berapa banyak profil yang dimilikinya, dan versi-versi yang menjadi
asal upgrade-nya. `auto-parse` juga mencetak zona waktu setiap sistem ketika mencantumkan sistem-sistem yang
ditemukannya.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host, dan hive yang dibaca (`SourcePath` adalah hive `SYSTEM`, atau jika tidak ada, hive pertama yang dibaca). |
| `CompName`, `Hostname`, `Domain` | Nama NetBIOS (`SYSTEM`, atau jika tidak ada, `PolAcDmN` milik `SECURITY`), nama host DNS, dan domain DNS (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | Domain tempat komputer bergabung, atau workgroup-nya (`PolPrDmN`), dan nama DNS domain tersebut (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: nama produk sebagaimana tertulis, `EditionID`, `DisplayVersion` (atau `ReleaseId`), versi mayor dan minor (atau `CurrentVersion`), build, update build revision, `CSDVersion`, serta `Client`, `Server`, atau `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (hingga detik) dan `InstallTime` (hingga 100 ns, Windows 10 dan yang lebih baru): instalasi atau feature upgrade terakhir. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Sebagaimana tertulis. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (atau `StandardName`), dan zona waktu dalam kata-kata beserta offset dan tanggal daylight saving-nya. |
| `UTC-Offset`, `DaylightUTC-Offset` | Offset waktu standar lokal dari UTC, dan offset daylight time (kosong jika tanpa daylight saving time). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (yang berlaku saat key ditulis), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Sebagaimana tertulis, sering kali berupa referensi resource (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Kapan waktu standar dan daylight time dimulai, dalam kata-kata (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dynamic daylight saving time dinonaktifkan; jam perangkat keras menyimpan UTC. |
| `TimeZoneLastWrittenUTC` | Waktu penulisan terakhir key zona waktu. |
| `LastShutdownUTC` | `ShutdownTime`: shutdown bersih terakhir. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | Control set yang dibaca, `PROCESSOR_ARCHITECTURE`, `ACP`, dan `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Pengguna terakhir di layar sign-in, dan waktu penulisan terakhir key-nya. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | SID komputer, SID domainnya (`PolPrDmS`), dan SID akun komputer di domain (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Kurang lebih kapan komputer terakhir diberi nama, terakhir bergabung ke domain atau workgroup, dan terakhir diperbarui (key `CurrentVersion`). |
| `PreviousVersions` | Setiap key `Source OS`: versi, tanggal instalasinya, dan tanggal di nama key (JSON: satu objek per key). |
| `Profiles` | Setiap profil: SID, folder, waktu key, load dan unload terakhir (JSON: satu objek per profil). |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry, untuk keempat hive. |
| `Problems` | Nilai dengan tipe atau ukuran yang salah (byte-nya dipertahankan), dan `SAM` serta `SECURITY` yang SID komputernya berbeda. |

## Strategi investigasi { #investigation-strategy }

1. Baca zona waktu terlebih dahulu: catat `UtcOffset`, tanggal daylight saving, dan apakah zona waktu
   baru saja diubah. Konversikan log berwaktu lokal dengannya.
2. Catat versi, build, dan tanggal instalasi. Periksa `PreviousVersions` untuk upgrade, dan bandingkan
   tanggal instalasi dengan waktu sistem berkas dan log peristiwa yang paling lama.
3. Catat `LastShutdownUTC`: aktivitas setelahnya tanpa waktu shutdown baru mengindikasikan crash, kehilangan
   daya, atau pengumpulan live.
4. Gunakan `MachineSid` dan `DomainSid` untuk membedakan akun lokal dari akun domain pada SID di tempat lain (BAM,
   `ProfileList`, log peristiwa), serta waktu load dan unload profil untuk sign-in terakhir.
5. Korelasikan nama komputer dan domain dengan log peristiwa, serta dengan `CompNameLastWrittenUTC`
   dan `PrimaryDomainLastWrittenUTC` untuk penggantian nama dan penggabungan ke domain.

## Detail lebih lanjut { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Security identifier](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Zona waktu](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Informasi produk Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
