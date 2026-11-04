# Autostart Registry { #registry-autostarts }

## Apa itu { #what-they-are }

Windows menjalankan program dan memuat DLL dari banyak lokasi registry: saat boot, saat logon, ketika sebuah
program dimulai atau berakhir, ketika `cmd.exe` dibuka, ketika sebuah kelas COM dibuat, atau ketika spooler,
LSA, netsh, atau layanan waktu memuat plug-in-nya. Penyerang menggunakan lokasi-lokasi ini untuk persistensi,
eskalasi hak akses, dan DLL injection, dan perangkat lunak yang sah juga menggunakannya. Juntei membacanya
dari `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, dan `UsrClass.dat` ke dalam satu tabel. Juntei menulis satu record per
program, DLL, atau paket, dan menandai masing-masing dengan teknik MITRE ATT&CK-nya.

<span class="evidence-strength">Kesimpulan terbaik:</span> hive mengonfigurasi program atau DLL ini
untuk dimulai atau dimuat. Hal ini tidak membuktikan bahwa program tersebut berjalan. Menjalankannya memerlukan kondisi yang
disebutkan oleh mekanismenya (logon, program yang di-debug dimulai, kelas dibuat), serta bukti
eksekusi lainnya.

## Lokasi { #locations }

| Hive | Yang dibaca |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` dan `Policies\Explorer\Run`, beserta salinan `Wow6432Node` dan mode instalasi Terminal Server-nya; `Explorer\StartupApproved\Run`, `Run32` dan `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … dan `Notify`); `Windows` `AppInit_DLLs` dengan `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (dengan `UseFilter`) dan `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, variabel profiler .NET di `Environment`); `Control\Lsa` `Authentication`, `Security` dan `Notification Packages` (+`OSConfig`); `SecurityProviders`; monitor, provider, dan print processor di `Control\Print`; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | key `Run` dan `StartupApproved` milik pengguna; `Windows NT\CurrentVersion\Windows` `Load` dan `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, profiler .NET); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (sebagai `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` dan `LocalServer32` (serta milik `Wow6432Node`); `shell\open\command` dari tipe program (`exefile`, `batfile`, `scrfile`, …) dan dari apa yang dijalankan oleh program yang melakukan auto-elevate (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Kumpulkan setiap hive beserta log transaksinya. Hive pengguna berada di `Users\<name>\NTUSER.DAT` dan
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Peringatan bagi analis { #analyst-cautions }

- **Nilai default juga dilaporkan.** Sebagian besar record pada sistem yang bersih adalah default Windows:
  - `Userinit` adalah `userinit.exe`;
  - `Shell` adalah `explorer.exe`;
  - `KnownDLLs`, paket LSA, monitor cetak, helper netsh, dan komponen Active Setup semuanya memiliki entri bawaan.

  Cari apa yang berbeda dari build bersih dengan versi yang sama.
- **`StartupApproved` hanya mencakup daftar Startup di Task Manager.** Key ini menunjukkan apakah sebuah entri `Run`
  atau berkas folder Startup dinonaktifkan di sana, dan kapan. Entri tanpa nilai `Run` yang cocok
  dilaporkan, dengan detail "only in StartupApproved". Hal ini dapat menunjukkan autostart yang
  telah dihapus.
- **`OverridesMachineRegistration` memerlukan hive `SOFTWARE` milik sistem.** Kelas COM atau
  perintah milik pengguna diperiksa terhadapnya, dan hive tersebut ditemukan dengan cara yang sama seperti hive `SYSTEM` milik host. Sebuah
  override adalah pola COM hijacking (T1546.015). OneDrive dan instalasi per pengguna lainnya juga
  mendaftarkan kelasnya sendiri yang tidak meng-override apa pun.
- **`ImagePath` dan `Args` dipisahkan dari baris perintah hanya jika hal itu dapat ditentukan.**
  - Program yang diberi tanda kutip dipisahkan pada tanda kutip penutupnya.
  - Kata pertama tanpa folder dianggap sebagai program (`cmd /c …`).
  - Selain itu, pemisahan dilakukan pada awalan terpendek yang diakhiri dengan ekstensi program.

  Path tanpa tanda kutip yang mengandung spasi dapat menjalankan program lain (`C:\Program.exe`) jika program tersebut ada.
- **`GlobalFlag` menentukan `Enabled` untuk `VerifierDlls` dan `MonitorProcess`.** `VerifierDlls` hanya dimuat
  dengan `0x100`. `MonitorProcess` hanya berjalan dengan `0x200` dan `0x1` dari `ReportingMode` milik `SilentProcessExit\<program>`.
  Mode `2` (dump) atau `4` (notifikasi) tidak menjalankan apa pun, dan
  `Details` menyebutkan flag mode tersebut.
- **`KeyLastWrittenUTC` adalah last write dari key**, bukan dari nilai. Beberapa nilai berbagi waktu yang sama.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Ringkasan menghitung record berdasarkan mekanisme. Ringkasan juga mencantumkan entri `StartupApproved\Run` dan `Run32`
yang tertinggal tanpa nilai `Run`-nya (berkas folder Startup tidak memiliki nilai yang bisa hilang), serta kelas COM dan
perintah milik pengguna yang meng-override milik mesin.

Seperti setiap perintah `parse-registry`, perintah ini menolak `-o` yang merupakan salah satu berkas yang dibacanya, bahkan
dengan `-C`. Hal itu mencakup hive, log transaksinya, serta hive SYSTEM dan SOFTWARE milik sistem,
berdasarkan path, symbolic link, atau hard link. Bila tidak dapat dipastikan, perintah ini tidak menulis.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Konteks host, hive sumber, dan tipenya. |
| `User`, `UserSID` | Untuk hive pengguna: folder profil, dan SID. SID berasal dari nama root key `UsrClass.dat`, atau jika tidak, dari `ProfileList` milik sistem. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Teknik MITRE ATT&CK, bila ada yang sesuai (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | Key di bawah root hive (nama sebagaimana tertulis) dan nilai (kosong: nilai default). |
| `Data` | Teks nilai, atau satu item dari sebuah list. Kosong bila tidak dapat dibaca, atau bila hanya `StartupApproved` yang menyebutkan entri tersebut. |
| `ImagePath`, `Args` | Program atau DLL, dan argumennya, bila keduanya dapat dibedakan (lihat di atas). |
| `Enabled`, `DisabledAtUTC` | Apa yang dinyatakan oleh `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (dengan `ReportingMode`), `UseFilter`, `IsInstalled`, `Enabled` milik time provider, atau `ScreenSaveActive`. Kosong bila tidak ada yang menyatakan. |
| `Details` | Informasi tambahan: apa yang dilakukan oleh rename yang tertunda, nama kelas COM, `ReportingMode`, `FilterFullPath`, `ShellComponent` milik Active Setup, … |
| `OverridesMachineRegistration` | Untuk kelas COM atau perintah milik pengguna: apakah mesin mendaftarkan yang sama. Kosong tanpa hive `SOFTWARE` milik sistem. |
| `WOW64` | Di bawah `Wow6432Node`: untuk program 32-bit. |
| `KeyLastWrittenUTC` | Last write dari key. |
| `DataRaw` | Byte nilai (hex), bila bukan teks atau bukan UTF-16 yang valid. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Tipe nilai yang salah, data yang tidak dapat dibaca, entri `StartupApproved` dengan ukuran yang salah. |

## Strategi investigasi { #investigation-strategy }

1. Mulailah dengan ringkasan:
   - kelas COM dan perintah yang meng-override milik mesin;
   - entri `StartupApproved` tanpa nilai `Run`-nya;
   - mekanisme yang jarang memiliki entri (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Bandingkan paket LSA, `BootExecute`, `KnownDLLs`, monitor cetak, dan helper netsh dengan build
   bersih dari versi Windows yang sama.
3. Cocokkan `ImagePath` dengan sistem berkas ($MFT, hash, dan signature), dan periksa path profil
   pengguna, `ProgramData`, dan `Temp`.
4. Korelasikan `KeyLastWrittenUTC` dengan peristiwa registry Security dan Sysmon, pembuatan proses,
   Prefetch, dan Amcache.
5. Gunakan `parse-registry read --recover-deleted` untuk menemukan autostart yang dihapus dari key-key ini.

## Detail lebih lanjut { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Key registry Run dan RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
