# Rahasia LSA { #lsa-secrets }

## Apa itu { #what-they-are }

Local Security Authority (LSA) menyimpan rahasia di hive `SECURITY`. Rahasia ini mencakup
kata sandi autologon, kata sandi akun yang digunakan untuk menjalankan layanan, kata sandi domain akun
komputer, kunci sistem DPAPI, dan kunci yang digunakan untuk mengenkripsi logon domain yang di-cache. LSA
juga menyimpan logon domain yang di-cache, sehingga pengguna domain dapat masuk tanpa domain controller.

`parse-registry lsa-secrets` mencantumkan rahasia apa saja yang disimpan sistem dan kapan rahasia tersebut ditetapkan, serta berapa
banyak logon yang di-cache yang disimpan dan digunakan. **Perintah ini hanya membaca nama dan waktu: tidak ada yang didekripsi, dan
tidak ada byte rahasia yang dibaca atau ditulis.** Juntei membaca bukti; Juntei tidak mengekstrak kredensial.

<span class="evidence-strength">Kesimpulan terbaik:</span> sistem menyimpan rahasia dengan nama ini, yang ditetapkan
pada waktu ini. Rahasia layanan (`_SC_<service>`) berarti LSA menyimpan kata sandi akun
yang digunakan untuk menjalankan layanan tersebut.

## Lokasi { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Nama | Jenis | Apa itu |
|---|---|---|
| `DefaultPassword` | `autologon` | Kata sandi yang digunakan Winlogon untuk masuk secara otomatis. |
| `_SC_<service>` | `service_account` | Kata sandi akun yang digunakan untuk menjalankan `<service>`. |
| `$MACHINE.ACC` | `machine_account` | Kata sandi akun komputer di domainnya. |
| `DPAPI_SYSTEM` | `dpapi_system` | Kunci DPAPI sistem. |
| `NL$KM` | `cached_logon_key` | Kunci yang digunakan untuk mengenkripsi logon yang di-cache. |
| `L$_SQSA_<SID>` | `security_questions` | Pertanyaan keamanan akun lokal (Windows 10 dan yang lebih baru). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Kredensial dial-up dan VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Data privat lainnya, berdasarkan prefiks yang diberikan oleh `LsaStorePrivateData`. |

Kumpulkan `SECURITY` beserta log transaksinya, dan hive `SYSTEM` sistem, yang menyebutkan nama
host.

## Peringatan bagi analis { #analyst-cautions }

- **Nama dan waktu, bukan nilai.** Apakah sebuah nilai berisi data ditentukan dari ukurannya. Kontennya
  tidak dibaca.
- **`DefaultPassword` tidak berarti autologon.** Rahasia ini ada di sebagian besar sistem, termasuk
  sistem yang tidak pernah masuk secara otomatis. `Microsoft\Windows NT\CurrentVersion\Winlogon` di
  `SOFTWARE` (`AutoAdminLogon`, `DefaultUserName`) menyatakan
  apakah autologon dikonfigurasi. `CurrentSetUTC` yang baru-baru ini merupakan perubahan yang perlu dijelaskan.
- **Rahasia layanan:** `_SC_<service>` muncul ketika sebuah layanan diatur untuk berjalan sebagai akun dengan
  kata sandi. Bandingkan dengan `parse-registry services` (`ObjectName`), dan cari layanan
  yang baru-baru ini diinstal atau tidak dikenal. Rahasia tersebut dapat tetap ada setelah layanannya hilang.
- **Waktu:** `CurrentSetUTC` dan `OldSetUTC` adalah waktu Windows menetapkan nilai saat ini dan nilai
  sebelumnya. Kata sandi `$MACHINE.ACC` berubah setiap 30 hari secara default, sehingga waktunya bergeser.
  `KeyLastWrittenUTC` serta waktu kunci `CurrVal`/`OldVal` menunjukkan kapan rahasia tersebut terakhir
  berubah.
- **Logon yang di-cache:** sebuah slot sedang digunakan bila catatannya menyebutkan seorang pengguna. Header catatan tidak
  dienkripsi; hanya panjang nama penggunanya yang dibaca. `CachedLogonsKept` adalah hitungan milik `NL$Control`,
  yang mengikuti `CachedLogonsCount` (10 secara default). Slot yang sedang digunakan berarti pengguna domain masuk
  secara interaktif, dan kredensial mereka dapat di-crack secara offline dari hive. Nama pengguna
  dienkripsi dan tidak didekode. Hanya catatan biner yang tervalidasi dengan header lengkap dan panjang nama pengguna yang konsisten yang dihitung sebagai sedang digunakan. Jika `Problems` melaporkan slot yang rusak, hitungan slot yang digunakan hanya mencakup slot yang tervalidasi; jangan menginterpretasikan slot yang rusak sebagai kosong.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Untuk setiap hive, ringkasan mencantumkan setiap rahasia beserta jenisnya dan kapan rahasia itu ditetapkan. Ringkasan kemudian menyebutkan
layanan yang kata sandi akunnya disimpan oleh LSA, dan menghitung slot logon yang di-cache yang sedang digunakan.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Host (dari hive `SYSTEM` sistem), dan hive `SECURITY`. |
| `Secret`, `Kind` | Nama rahasia, dan apa itu (lihat tabel di atas); `Cache` dan `cached_logons` untuk catatan logon yang di-cache. |
| `ServiceName`, `UserSID` | Layanan yang menjadi pemilik rahasia `_SC_`, dan SID pengguna yang diberikan oleh sebuah nama (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` dan `OupdTime`: kapan nilai saat ini dan nilai sebelumnya ditetapkan. |
| `HasCurrentValue`, `HasOldValue` | Apakah `CurrVal` dan `OldVal` berisi data. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | Last write kunci `CurrVal`, `OldVal`, dan kunci rahasia. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Catatan logon yang di-cache: slot, slot yang sedang digunakan, dan hitungan milik `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Waktu yang cacat atau di luar rentang (hanya metadata waktu yang ditampilkan), header cache yang rusak, tipe nilai cache yang salah, dan kunci yang tidak dapat dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Cari rahasia `_SC_` milik layanan yang tidak dikenal atau baru diinstal, dan cocokkan dengan
   `parse-registry services` serta event 7045 (sebuah layanan diinstal).
2. Periksa waktu `DefaultPassword` terhadap pengaturan autologon `Winlogon`.
3. Pada anggota domain, catat slot logon yang di-cache yang sedang digunakan, dan kapan `Cache` terakhir ditulis.
4. Bandingkan waktu rahasia dengan timeline: rahasia yang ditetapkan selama jendela intrusi dapat
   menandai akun layanan baru, perubahan autologon, atau perubahan akun mesin.

## Detail lebih lanjut { #further-details }

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Cached domain logon information](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
