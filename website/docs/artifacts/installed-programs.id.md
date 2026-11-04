# Program terinstal { #installed-programs }

## Apa yang disimpan Windows { #what-windows-keeps }

Windows dan installer mencatat perangkat lunak yang terinstal dalam beberapa daftar registry. Setiap daftar memiliki satu kunci
per program, produk, distribusi, atau paket; Juntei memberikan satu catatan per kunci.

| Sumber | Hive dan kunci | Apa isinya |
|---|---|---|
| `uninstall` | `SOFTWARE` dan `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, serta `SOFTWARE\Wow6432Node\…` | Apa yang dicantumkan oleh Settings dan *Programs and Features*: nama, versi, penerbit, tanggal instalasi, folder, perintah uninstall, ukuran, apakah tersembunyi |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Produk Windows Installer: nama dan versi produk, nama berkas paket, dan folder asal instalasinya |
| `app_path` | `SOFTWARE` dan `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Program yang dijalankan Windows berdasarkan nama (`Win+R`, `start tool`): path program dan foldernya |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Distribusi Windows Subsystem for Linux: nama, lokasi berkas atau disk-nya, WSL 1 atau 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Aplikasi terpaket (Store): yang di-provision untuk setiap pengguna, milik Windows sendiri, staged, deprovisioned, dan milik masing-masing pengguna |

<span class="evidence-strength">Kesimpulan terbaik:</span> program, produk, atau paket ini
terinstal (atau terdaftar) di sistem, untuk setiap pengguna atau untuk pengguna ini. Tanggal instalasi adalah
apa yang ditulis oleh installer; last write kunci adalah waktu entri tersebut terakhir berubah.

## Lokasi { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Kumpulkan `SOFTWARE`, setiap `NTUSER.DAT`, beserta log transaksinya. `SYSTEM` menyebutkan nama host, dan
`ProfileList` di `SOFTWARE` memberikan SID setiap pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Tanggal instalasi dan waktu kunci.** `InstallDate` adalah tanggal (tanpa jam) yang ditulis oleh installer, dalam
  waktu lokal; sebagian installer tidak menuliskannya, dan sebagian lain menulis format berbeda (disimpan apa adanya di
  `InstallDateRaw`). Pembaruan dapat menulis ulang entri tanpa mengubah tanggal, atau mengubah tanggal
  tanpa banyak perubahan pada program: bandingkan `InstallDate` dengan `KeyLastWrittenUTC`.
- **Perangkat lunak yang telah di-uninstall.** Program yang dihapus secara bersih kehilangan kuncinya. Gunakan `parse-registry read
  --recover-deleted` untuk kunci `Uninstall` yang terhapus, serta Amcache, Prefetch, dan `$MFT` untuk jejak
  yang tertinggal.
- **Registrasi per pengguna.** Entri `NTUSER.DAT` mengidentifikasi perangkat lunak yang terdaftar untuk suatu profil.
  Cakupan tersebut tidak menetapkan siapa yang menjalankan installer atau apakah hak administrator digunakan.
- **Entri tersembunyi.** `SystemComponent` menyembunyikan entri dari *Programs and Features*; entri
  tanpa `DisplayName` (pembaruan, komponen) juga tidak ditampilkan. Semuanya dilaporkan.
- **Kode produk.** Kunci `Uninstall` berbentuk GUID dapat dicocokkan dengan kunci `msi_product`
  yang dipadatkan (packed) melalui `ProductCode`. Nama berbentuk GUID saja tidak membuktikan bahwa
  Windows Installer yang membuat entri tersebut.
- **App Paths.** Ini adalah registrasi yang digunakan untuk menemukan program, bukan catatan
  eksekusi. Nilai `Path` dapat berisi beberapa direktori pencarian dan belum tentu merupakan
  direktori instalasi.
- **Penerbit.** Ini adalah metadata registry yang diberikan oleh installer, bukan informasi
  tanda tangan yang telah diverifikasi.
- **Arsitektur.** `Wow6432Node` menyimpan program 32-bit pada Windows 64-bit (`Architecture` `x86`,
  `Wow64` `True`); entri lain menggunakan arsitektur sistem itu sendiri (kosong). Arsitektur
  paket adalah arsitekturnya sendiri (`x64`, `neutral`, …).
- **WSL.** `Version` adalah versi WSL dari `Flags` distribusi (`0x8`: berjalan di
  mesin virtual WSL 2). Nilai `Version` milik distribusi itu sendiri (di `Details`) adalah versi sistem berkasnya,
  yaitu 2 juga pada distribusi WSL 1. `BasePath` berisi `ext4.vhdx` (WSL 2) atau `rootfs`
  (WSL 1) miliknya: kumpulkan untuk melihat isinya.
- **Paket.** Daftar `<SID>` adalah paket yang terdaftar untuk setiap pengguna; sebagian besar milik
  Windows sendiri. Paket `Deprovisioned` dihapus untuk pengguna baru, paket `EndOfLife\<SID>` untuk pengguna tersebut.
- **Nilai yang rusak.** Angka yang cacat, tipe nilai yang salah, dan UTF-16 yang tidak valid dilaporkan di
  `Problems` dan dipertahankan di `DataRaw`, termasuk string hex kosong untuk nilai yang ada tetapi berukuran
  nol byte. String kosong yang bersih pada field numerik opsional diperlakukan sebagai tidak ada. Versi
  paket harus berisi empat komponen desimal, masing-masing dalam rentang unsigned 16-bit.
  Peringatan tingkat hive tetap berada di stderr ketika `-t` mengalirkan catatan ke stdout.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

Ringkasan mencantumkan program yang terakhir diinstal (berdasarkan `InstallDate`), apa yang diinstal pengguna untuk
dirinya sendiri, dan distribusi WSL.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, berkas hive, dan `SOFTWARE` atau `NTUSER.DAT`. |
| `User`, `UserSID` | Pengguna dari hive pengguna (folder profilnya dan SID dari `ProfileList`), atau pengguna yang tercantum untuk sebuah paket. |
| `Source`, `Scope` | Daftar (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`), dan `machine` atau `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` dan `Version` sebuah produk; nama berkas program sebuah App Path; nama distribusi dan versi WSL; nama dan versi paket. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` sebagai tanggal bila ditulis dalam format `YYYYMMDD`; jika tidak, apa adanya. |
| `KeyLastWrittenUTC` | Waktu kunci entri terakhir berubah. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (`Path` sebuah App Path, `BasePath` distribusi, `Path` paket); `InstallSource` (folder sumber yang terakhir digunakan oleh produk). |
| `Program` | Program sebuah App Path. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` dalam KiB; apakah tersembunyi; apakah diinstal oleh Windows Installer. |
| `ProductCode` | Kode produk Windows Installer (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` di bawah `Wow6432Node`, atau arsitektur paket; apakah kunci berada di bawah `Wow6432Node`. |
| `Details` | Informasi tambahan: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (akun yang menjalankan installer Inno Setup); `PackageCode`, `PackageName` sebuah produk; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` sebuah distribusi; `List`, `ResourceId`, `PublisherId` sebuah paket. |
| `KeyPath` | Kunci entri. |
| `DataRaw` | Byte (hex) dari nilai yang tidak sesuai dengan yang dinyatakan namanya. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai yang tidak dapat dibaca atau tidak sesuai dengan yang dinyatakan namanya. |

## Strategi investigasi { #investigation-strategy }

1. Urutkan berdasarkan `InstallDate` dan `KeyLastWrittenUTC` di sekitar insiden; perhatikan alat akses
   jarak jauh, klien tunneling dan transfer berkas, pengarsip, runtime scripting, dan alat keamanan.
2. Tinjau catatan dengan `Scope` `user`: perangkat lunak yang diinstal tanpa hak administrator.
3. Periksa `InstallLocation`, `InstallSource`, dan `UninstallCommand` untuk folder yang dapat ditulisi pengguna
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Cari distribusi WSL, dan kumpulkan `BasePath`-nya.
5. Korelasikan dengan Amcache dan Prefetch (installer dan program yang berjalan), `$MFT` (pembuatan
   folder instalasi), dan log peristiwa Application (peristiwa MsiInstaller).

## Detail lebih lanjut { #further-details }

- [Microsoft: Uninstall registry key](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: App registration (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
