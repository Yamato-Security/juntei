# MuiCache { #muicache }

## Apa itu MuiCache { #what-muicache-is }

Ketika pengguna menjalankan sebuah program, Windows membaca version resource program tersebut untuk mendapatkan nama yang
harus ditampilkan shell (`FileDescription`, misalnya "Windows Command Processor" untuk
`cmd.exe`) dan menyimpannya dalam cache per pengguna di `MuiCache`. Windows 8 dan yang lebih baru juga menyimpan nama perusahaan
(`CompanyName`). Cache ini diberi kunci berdasarkan path lengkap program:

- **Windows 8 dan yang lebih baru:** dua nilai per program, `<path>.FriendlyAppName` dan
  `<path>.AppCompany`.
- **Windows Vista dan 7:** satu nilai per program, `<path>`, yang berisi nama ramah (friendly name).
- **Windows XP:** bentuk yang sama, di `NTUSER.DAT`.

MuiCache dapat memberikan petunjuk eksekusi program untuk sesi shell seorang pengguna, termasuk program
di drive lepas pasang, network share, dan folder yang sudah tidak ada lagi, beserta nama dan perusahaan
yang diberikan oleh berkas itu sendiri.

<span class="evidence-strength">Kesimpulan terbaik:</span> profil shell ini menyimpan path,
deskripsi, dan perusahaan sebuah berkas dalam cache. Korelasikan dengan artefak eksekusi lainnya. Profil tersebut tidak mengidentifikasi
akun yang digunakan untuk menjalankan proses. MuiCache tidak memiliki waktu per program dan tidak memiliki hitungan eksekusi.

## Lokasi { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (bahasa string yang di-cache) bukan program dan tidak disertakan. Nilai bernama
`@<file>,-<number>` (`@shell32.dll,-22019`) adalah resource string yang di-cache shell untuk menu
dan tipe berkasnya sendiri, bukan program; nilai ini tidak disertakan dan dihitung dalam ringkasan. Pada Windows Vista
dan yang lebih baru, `NTUSER.DAT` tidak memiliki MuiCache.

Kumpulkan `UsrClass.dat` (dan `NTUSER.DAT`) beserta log transaksinya untuk setiap profil, serta hive
`SYSTEM` dan `SOFTWARE` sistem untuk nama host dan nama pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Pemilik profil belum tentu akun proses.** Program yang dijalankan dengan akun
  lain dapat menampilkan jendela di sesi pengguna ini dan meninggalkan metadata di cache pengguna ini.
  `User` dan `UserSid` mengidentifikasi pemilik hive, bukan konteks keamanan proses.
- **Tidak ada waktu per program.** `KeyLastWrittenUTC` adalah last write kunci: perubahan terbaru pada
  salah satu programnya, bukan waktu program tertentu dijalankan. Prefetch, BAM, UserAssist, dan log peristiwa memberikan
  waktu.
- **Tidak hanya program yang dijalankan pengguna secara manual.** DLL atau program yang dimuat oleh shell itu sendiri (untuk
  `Open with`, shell extension, child dari installer) dapat muncul, seperti `explorerframe.dll` atau
  `shell32.dll`.
- **Nama berasal dari berkas.** `FriendlyAppName` dan `AppCompany` adalah apa yang dinyatakan oleh
  version resource berkas saat dijalankan. Malware dapat mengklaim deskripsi dan perusahaan apa pun; alat yang diganti namanya
  tetap mempertahankan deskripsi aslinya, yang dapat mengungkapkannya (berkas bernama `svchost.exe` yang
  friendly name-nya adalah alat akses jarak jauh).
- **Satu entri per path.** Menjalankan program yang sama lagi tidak menambah entri; salinan di path
  lain akan menambah entri. Entri sebuah program dapat tetap ada setelah berkasnya dihapus.
- **Kedua bentuk untuk satu path.** Bila sebuah path memiliki friendly name dalam kedua bentuk (profil yang
  di-upgrade), masing-masing mendapatkan catatannya sendiri, sehingga tidak ada nilai yang hilang.
- **Nilai yang rusak.** Nilai yang bukan string, bukan UTF-16 yang valid, atau tidak dapat dibaca utuh tetap mempertahankan
  byte-nya (`FriendlyAppNameRaw`, `AppCompanyRaw`) dan dilaporkan di `Problems`.
  UTF-16 yang tidak valid juga dapat menghasilkan karakter pengganti pada teks yang ditampilkan; gunakan kolom
  mentah untuk byte aslinya.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Ringkasan mencantumkan program di luar folder Windows dan Program Files (folder pengguna,
drive lain, network share) beserta penggunanya, dan jumlah resource string yang tidak disertakan.
Folder Program Files dengan nama lain (pada Windows dalam bahasa lain) juga dicantumkan.
Path relatif dan path yang keluar dari folder environment variable tetap dicantumkan. Path drive
biasa diperiksa setelah meresolusi komponen `.` dan `..` tanpa mengakses sistem berkas
milik pemeriksa.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nama folder profil, SID pengguna (disebutkan oleh root key `UsrClass.dat`; jika tidak, dari `ProfileList` sistem), dan hive. |
| `HiveType` | `UsrClass.dat` atau `NTUSER.DAT`. |
| `Path` | Path program: nama nilai tanpa `.FriendlyAppName` atau `.AppCompany`. |
| `FriendlyAppName` | Deskripsi program, dari `<path>.FriendlyAppName` atau `<path>`. |
| `AppCompany` | Perusahaan program, dari `<path>.AppCompany` (Windows 8 dan yang lebih baru). |
| `KeyPath`, `KeyLastWrittenUTC` | Kunci MuiCache, dan last write-nya: perubahan terbaru pada salah satu programnya. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Byte nilai (hex), bila byte tersebut bukan teks yang dapat didekode dengan bersih. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry: `replayed` bila log menulis salah satu nilai program tersebut. |
| `Problems` | Nilai yang bukan string, bukan UTF-16 yang valid, atau tidak dapat dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Perhatikan program di lokasi yang dapat ditulisi pengguna (`\Users\`, `Downloads`, `Temp`, `ProgramData`), di
   drive lain dan di network share, serta alat administrasi, akses jarak jauh, dan pengarsipan.
2. Bandingkan nama berkas dengan friendly name dan perusahaannya: ketidakcocokan dapat mengungkap alat yang
   diganti namanya atau berkas yang menyamar.
3. Korelasikan dengan Prefetch, BAM, dan UserAssist (kapan program dijalankan), Amcache dan ShimCache (hash
   dan keberadaan berkas), serta `$MFT` (apakah berkas masih ada).
4. Gunakan `parse-registry read --recover-deleted` pada `UsrClass.dat` untuk nilai MuiCache yang terhapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: VERSIONINFO resource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: eksperimen MuiCache, sesi GUI, dan keterbatasan akun proses](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
