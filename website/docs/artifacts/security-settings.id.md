# Pengaturan keamanan { #security-settings }

## Apa itu { #what-they-are }

Penyerang mengubah pengaturan keamanan Windows agar tetap tersembunyi dan untuk menjangkau kredensial. Mereka mematikan
Microsoft Defender atau mengecualikan alat mereka darinya, membuat Windows menyimpan kata sandi dalam teks biasa (cleartext),
membuka Remote Desktop, mematikan firewall, dan membungkam logging PowerShell. Registry menyimpan
pengaturan-pengaturan ini:

- **`SOFTWARE`:** Defender, pengecualian dan kebijakannya; UAC; kebijakan Remote Desktop dan
  firewall; execution policy PowerShell, kebijakan logging, dan provider AMSI; System Restore;
  channel log peristiwa.
- **`SYSTEM`** (per control set): `WDigest`, proteksi LSA, Credential Guard, Restricted Admin,
  NTLM; Remote Desktop; profil firewall; SMBv1; waktu akses terakhir; ukuran setiap log peristiwa.
- **`NTUSER.DAT`:** execution policy dan kebijakan logging PowerShell milik pengguna, serta AMSI untuk
  Windows Script Host.

`parse-registry security-settings` melaporkan setiap pengaturan yang ditemukannya sebagai sebuah catatan. Setiap catatan berisi
nilai pengaturan, makna nilai tersebut, dan apakah nilai itu **melemahkan default**: apakah nilai itu mematikan
suatu proteksi atau sumber bukti, atau menurunkannya di bawah apa yang dilakukan Windows saat ini (10 dan 11)
tanpa nilai tersebut.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengaturan ini memiliki nilai ini ketika hive
disimpan, dan kunci tersebut terakhir ditulis pada waktu ini. Pengaturan yang dilemahkan adalah petunjuk, bukan bukti
kompromi. Administrator, Group Policy, produk keamanan lain, dan versi Windows yang lebih lama juga menetapkan
sebagian nilai ini, dan bagian makna menyebutkannya bila hal itu umum terjadi.

## Lokasi { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Kumpulkan ketiga hive beserta log transaksinya. Pengaturan yang diubah tidak lama sebelum pengumpulan
mungkin hanya ada di log.

## Peringatan bagi analis { #analyst-cautions }

- **Konfigurasi yang tersimpan bukanlah status proteksi yang efektif.** Nilai registry yang dikumpulkan dan baris diagnostik untuk daftar nilai yang rusak
  dilaporkan. Data yang tidak ada dapat mencerminkan default, sumber yang tidak dikumpulkan, atau mekanisme
  manajemen lain; itu bukan bukti bahwa proteksi diaktifkan atau dinonaktifkan. `WeakensDefault` menandai
  pengaturan yang dapat menurunkan baseline yang dinyatakan bila pengaturan tersebut dipatuhi.
- **Flag Defender lawas mungkin diabaikan.** `DisableAntiSpyware` dan `DisableAntivirus` tidak
  membuktikan bahwa Defender berhenti. Keberlakuannya bergantung pada versi platform, onboarding, dan tamper
  protection. [Microsoft menjelaskan pengaturan lawas ini](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Pengecualian terkelola modern bisa tidak tersedia dalam tangkapan registry.** Mulai platform
  4.18.25110.6, perangkat yang menggunakan manajemen konfigurasi Defender for Endpoint memerlukan cmdlet
  PowerShell Defender yang didukung untuk mengambil konfigurasi pengecualian. Oleh karena itu, ketiadaan di registry offline
  tidak menetapkan bahwa tidak ada pengecualian.
  [Panduan konfigurasi Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Dilemahkan bukan berarti berbahaya.** Contoh:
  - `DisableAntiSpyware` = 1 juga ditetapkan oleh Windows selama antivirus lain terdaftar.
  - Remote Desktop dibuka secara sengaja di server.
  - Windows 7 menyimpan log peristiwa yang lebih kecil dan tidak memperbarui waktu akses terakhir.
- **Default bergantung pada versi.** Beberapa nilai dahulu merupakan default:
  - `NtfsDisableLastAccessUpdate` = 1 adalah default sebelum Windows 10 1803. Nilai ini dilaporkan
    tanpa putusan, tetapi `0x80000001`, yang ditetapkan secara manual pada versi yang lebih baru, dianggap melemahkan.
  - `RunAsPPL`, `LsaCfgFlags`, dan `UseLogonCredential` dibandingkan dengan Windows saat ini.
- **Pengecualian Defender:** setiap path, ekstensi, proses, atau alamat adalah sebuah catatan. Setiap pengecualian
  melemahkan Defender. `TemporaryPaths`, yang ditambahkan Defender sendiri, tidak memiliki putusan. Cari folder yang dapat ditulisi pengguna,
  ekstensi `.ps1`/`.exe`, dan LOLBin (`rundll32.exe`, `powershell.exe`).
- **Kunci kebijakan** (`Policy` = True) ditulis oleh Group Policy, oleh alat, dan oleh penyerang yang
  tahu bahwa kebijakan mengalahkan pengaturan lokal. Korelasikan dengan GPO domain.
- **Channel log peristiwa:** hanya 25 channel yang relevan dengan keamanan yang dilaporkan, beserta apakah Windows
  mengaktifkan masing-masing secara default. `TaskScheduler/Operational` dan `DNS-Client/Operational`, misalnya,
  nonaktif secara default. Gunakan `parse-registry read` untuk channel lainnya.
- **Waktu kunci:** `KeyLastWrittenUTC` adalah milik kunci, sehingga belum tentu merupakan waktu nilai ini
  berubah. Seluruh kunci `Policies` yang ditulis sekaligus adalah hal yang umum.
- **Control set:** secara default, control set yang terakhir digunakan Windows yang dibaca. `--all-control-sets`
  membaca semuanya.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

Ringkasan menghitung pengaturan per area. Ringkasan mencantumkan pengaturan yang melemahkan default dan setiap
pengecualian Defender.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive, dan tipenya (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Untuk `SYSTEM`: control set, dan apakah control set itu yang terakhir digunakan Windows; tidak diketahui bila `Select\Current` tidak ada atau tidak valid. |
| `User`, `UserSID` | Untuk `NTUSER.DAT`: nama folder profil, dan SID pengguna dari `ProfileList` sistem. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Pengaturan apa itu: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Nilainya: angka dalam desimal, string, atau pengecualian itu sendiri. Kosong bila nilai tidak sesuai dengan tipe pengaturan. |
| `Meaning` | Makna nilai tersebut. |
| `WeakensDefault` | `True` bila nilai melemahkan apa yang dilakukan Windows saat ini tanpanya, `False` bila tidak, kosong bila hal itu tidak berlaku (ukuran, port, provider AMSI) atau tidak dapat ditentukan. |
| `Policy` | Apakah kunci berada di bawah kunci `Policies`. |
| `KeyPath`, `ValueName` | Kunci dan nilai. Nama nilai sebuah pengecualian adalah item yang dikecualikan; provider AMSI tidak memilikinya. |
| `KeyLastWrittenUTC` | Last write kunci. |
| `DataRaw` | Byte nilai (hex), bila tidak sesuai dengan tipe pengaturan. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai dengan tipe yang salah dan kunci yang tidak dapat dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Jalankan dengan `--weakened-only` pada setiap host, dan bandingkan host satu sama lain: pengaturan
   yang hanya dimiliki satu host akan menonjol.
2. Periksa setiap pengecualian Defender terhadap alat yang ditemukan di host dan di log Defender
   (Event 5007 mencatat perubahan konfigurasi).
3. Korelasikan waktu kunci dengan logon, instalasi layanan, dan pembuatan proses. Cari command line
   `reg.exe`, `Set-MpPreference`, `netsh advfirewall`, dan `powershell` (event
   4688, Sysmon 1 dan 13, PowerShell 4104).
4. Untuk pengaturan kredensial (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`), cari credential dumping dan pass-the-hash setelahnya.
5. Gunakan `parse-registry read --recover-deleted` untuk nilai yang dihapus guna memulihkan default, dan
   `--all-control-sets` untuk control set yang lebih lama.

## Detail lebih lanjut { #further-details }

- [Microsoft: Configure and validate exclusions for Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: User Account Control settings and configuration](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Configure added LSA protection](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
