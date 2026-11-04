# FeatureUsage, RecentApps, dan item yang disematkan di taskbar { #featureusage-recentapps-and-taskbar-pins }

## Apa itu { #what-they-are }

Explorer menyimpan beberapa catatan tentang cara pengguna menggunakan aplikasi di `NTUSER.DAT` milik pengguna tersebut:

- **FeatureUsage** (Windows 10 dan yang lebih baru): penghitung per aplikasi, berdasarkan jenis penggunaan.
  - `AppSwitched`: berapa kali pengguna beralih ke jendela aplikasi (klik taskbar,
    Alt+Tab).
  - `AppLaunch`: berapa kali aplikasi dijalankan dari taskbar (item yang disematkan).
  - `ShowJumpView`: berapa kali jump list-nya dibuka (klik kanan pada taskbar).
  - `AppBadgeUpdated`: berapa kali badge taskbar-nya (jumlah yang belum dibaca) berubah.
  - `TrayButtonClicked`: berapa kali setiap tombol taskbar (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) diklik.

  Sebuah aplikasi berupa application user model ID (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), path yang bagian pertamanya bisa berupa GUID
  known folder (`{1AC14E77-…}\cmd.exe` adalah `%windir%\system32\cmd.exe`), atau ID proses
  (`*PID00001d74`) untuk jendela yang tidak dapat dicocokkan Explorer dengan suatu aplikasi.
- **RecentApps** (Windows 10 hingga 1709): aplikasi yang baru-baru ini dijalankan pengguna, dengan jumlah
  peluncuran dan waktu terakhir; di bawah masing-masing, berkas yang dibukanya (`RecentItems`).
- **Pin taskbar** (`Taskband`): program yang disematkan ke taskbar, sebagai shell item
  (`Favorites`) dan sebagai tautan dari setiap pintasan yang disematkan (`FavoritesResolve`). Pada Windows 10 dan
  yang lebih baru, setiap pin menyatakan apakah Windows yang menyematkannya (`SystemPinned`) atau pengguna (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP hingga 7): program yang di-cache oleh
  Start menu (`ProgramsCache`), dan pada Windows 7 program yang disematkan di Start menu dan taskbar
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna ini menggunakan aplikasi ini
sebanyak sekian kali dengan cara ini (FeatureUsage), menjalankannya pada waktu ini (RecentApps), atau menyematkannya.
Penghitung tidak memiliki waktu sendiri; hanya last write kunci dan waktu-waktu RecentApps yang menandai
waktu sesuatu.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`KeyCreationTime` milik `FeatureUsage` adalah waktu mulai penghitungan; ringkasan menampilkannya. Kumpulkan
`NTUSER.DAT` beserta log transaksinya untuk setiap profil, serta hive `SYSTEM` dan
`SOFTWARE` sistem untuk host dan SID pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Hitungan tanpa waktu.** Hitungan FeatureUsage menyatakan seberapa sering, bukan kapan. Waktu
  last-write kunci jenis tersebut (`KeyLastWrittenUTC`) adalah perubahan terbaru pada salah satu hitungannya.
- **Tidak setiap eksekusi.** `AppSwitched` menghitung peralihan jendela, bukan start; aplikasi yang berjalan
  tanpa jendela, atau yang tidak pernah dialihkan oleh pengguna, tidak dihitung. `AppLaunch` hanya menghitung
  start dari taskbar.
- **Aplikasi terpaket dan path.** Sebuah aplikasi diberi nama sesuai cara Windows mengelompokkan jendelanya: ID
  aplikasi terpaket, ID yang ditetapkan program untuk dirinya sendiri (`Chrome`, `VMware.Workstation.vmui`), atau
  path. `AppPath` memberikan path dengan GUID known folder diganti oleh path default
  folder tersebut.
- **Pin default.** Profil baru mendapatkan pin default (File Explorer, Edge, Store); pada Windows 10
  dan yang lebih baru pin tersebut bertanda `SystemPinned`. Pin `UserPinned` adalah pilihan pengguna.
- **Pin dalam dua daftar.** `Favorites` dan `FavoritesResolve` mendeskripsikan pin yang sama dalam urutan
  yang sama (`Index`), tetapi keduanya diberikan sebagai catatan terpisah.
- **Path ProgramsCache** bersifat relatif: sebuah folder (`Programs\Accessories`) diikuti oleh
  pintasan di dalamnya (`Notepad.lnk`).
- **Nilai yang rusak.** Hitungan yang bukan `REG_DWORD`, waktu yang bukan `REG_QWORD`, pin
  yang tidak dipahami, atau tautan yang ternyata bukan tautan tetap mempertahankan byte-nya (`DataRaw`) dan dilaporkan di
  `Problems`.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

Ringkasan mencantumkan catatan per sumber, kapan FeatureUsage setiap hive mulai menghitung,
aplikasi yang paling sering menjadi tujuan peralihan, aplikasi dan item terbaru (terbaru lebih dulu), serta pin taskbar.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nama folder profil, SID pengguna dari `ProfileList` sistem, dan hive. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` atau `ProgramsCache`. |
| `App` | Nama nilai FeatureUsage, `AppId` aplikasi terbaru, atau application user model ID sebuah pin. |
| `AppPath` | `App` sebagai path, dengan GUID known folder diganti oleh path default-nya; kosong untuk ID. |
| `Count` | Hitungan FeatureUsage, atau `LaunchCount` aplikasi terbaru. |
| `LastAccessedUTC` | `LastAccessedTime` aplikasi atau item terbaru. |
| `ItemPath` | `Path` item terbaru (atau `AppPath` aplikasi terbaru); path shell item sebuah pin atau program yang di-cache (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); path tautan `FavoritesResolve`. |
| `DisplayName` | `DisplayName` item terbaru. |
| `PinType` | `SystemPinned` atau `UserPinned`, untuk pin `Favorites`. |
| `Index` | Posisi pin atau program yang di-cache dalam daftarnya. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | Nilai dan kunci asalnya, serta last write kunci tersebut. |
| `DataRaw` | Byte (hex) yang tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai dengan tipe yang salah, pin atau daftar yang tidak dipahami, tautan yang bukan tautan. |

## Strategi investigasi { #investigation-strategy }

1. Urutkan `AppSwitched` berdasarkan `Count` untuk aplikasi yang paling banyak digunakan pengguna, dan cari
   alat administrasi, akses jarak jauh, pengarsipan, dan transfer, serta path di lokasi yang dapat ditulisi pengguna
   (`\Users\`, `Downloads`, `Temp`).
2. Periksa pin: alat dengan `UserPinned`, atau pin yang pintasannya berada di lokasi yang tidak biasa.
3. Gunakan waktu-waktu RecentApps, bersama UserAssist, Prefetch, BAM, dan Jump Lists, untuk menentukan waktu penggunaan.
4. Bandingkan `KeyCreationTime` dengan waktu pembuatan profil: penghitungan dimulai sejak saat itu.
5. Gunakan `parse-registry read --recover-deleted` untuk penghitung dan pin yang terhapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
