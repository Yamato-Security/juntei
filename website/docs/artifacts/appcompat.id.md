# Kompatibilitas aplikasi { #application-compatibility }

## Apa yang disimpan Windows { #what-windows-keeps }

Selain ShimCache (`parse-registry shimcache`) dan Amcache (`parse-registry amcache`),
fitur kompatibilitas aplikasi Windows menyimpan hal-hal berikut di bawah
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| Sumber | Hive dan kunci | Apa yang ditunjukkan |
|---|---|---|
| `layers` | `SOFTWARE` (dan `Wow6432Node`) serta `NTUSER.DAT` `Layers` | Program yang diatur untuk berjalan dengan lapisan kompatibilitas: `RUNASADMIN` (selalu sebagai administrator), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 dan yang lebih baru) | Program yang terlihat dijalankan oleh pengguna menurut Program Compatibility Assistant |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Hal yang sama, pada Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Program yang memuat modul yang dilacak, dan kapan terakhir kali melakukannya |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 hingga Windows 10 awal) | Path program dan command line dalam database CIT yang terkompresi |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Database shim kustom yang diinstal dengan `sdbinst`: berkas, deskripsi, waktu instalasi |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Database shim kustom yang diterapkan pada setiap program, beserta waktu instalasinya |

<span class="evidence-strength">Kesimpulan terbaik:</span> lapisan dan database shim adalah
konfigurasi yang mengubah cara program berjalan; entri Program Compatibility Assistant dan CIT
menunjukkan bahwa sebuah program berjalan (atau memuat modul) sebagai pengguna ini atau pada sistem ini.

Database shim kustom adalah teknik persistensi dan injeksi (MITRE ATT&CK T1546.011): sebuah shim
dapat memuat DLL ke dalam program atau menjalankan program lain setiap kali program tersebut berjalan.

## Lokasi { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Kumpulkan `SOFTWARE`, setiap `NTUSER.DAT` beserta log transaksinya, dan berkas `.sdb` yang disebutkan oleh path
database shim (biasanya `C:\Windows\AppPatch\Custom\`).

## Peringatan bagi analis { #analyst-cautions }

- **Ekspansi CIT dibatasi.** Nilai yang menyatakan ukuran tidak terkompresi lebih dari 64 MiB dipertahankan
  sebagai byte mentah dengan sebuah diagnostik. Kompresi yang cacat dan ketidaksesuaian ukuran dilaporkan.


- **Tidak ada waktu eksekusi untuk entri Program Compatibility Assistant.** Data `Store` (`SACP`, disimpan di
  `DataRaw`) memiliki satu waktu, tetapi pada Windows 8.1, 10, dan 11 nilainya sama untuk setiap program dalam satu
  hive, begitu pula pada sistem lain: itu bukan waktu program dijalankan. Hanya last write kunci `Store` yang
  menandai waktu perubahan terbaru.
- **Program pada media lepas pasang** diberi nama `SIGN.MEDIA=<id> <path>`: `Path` adalah path pada
  media tersebut, dan `Details` berisi id media.
- **Database CIT** dikompresi dengan LZNT1 dan strukturnya tidak didokumentasikan secara publik: Juntei
  mencantumkan path program dan command line yang ditemukan di masing-masing database, satu kali per database, tanpa hitungan atau
  waktu. Sebuah command line adalah catatan tersendiri, tidak terikat pada programnya. Windows 10 dan yang lebih baru hanya menyimpan
  sedikit data di sini.
- **Database shim.** Periksa setiap catatan `custom_sdb` dan `installed_sdb`: perangkat lunak yang sah
  jarang menginstalnya. Berkas `.sdb` itu sendiri menunjukkan apa yang dilakukan shim tersebut.
- **Lapisan** yang diatur oleh pengguna hanya berlaku untuk pengguna tersebut; lapisan di `SOFTWARE` berlaku untuk semua orang. `RUNASADMIN` membuat
  program meminta elevasi setiap kali dijalankan.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

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
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

Ringkasan mencantumkan database shim kustom, program yang memiliki lapisan, dan penggunaan terakhir
modul-modul CIT.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, berkas hive, dan `SOFTWARE` atau `NTUSER.DAT`. |
| `User`, `UserSID` | Nama folder profil dari hive pengguna, dan SID-nya dari `ProfileList` sistem. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` atau `user`. |
| `Path` | Program (nama nilai), path atau command line dalam database CIT, atau program yang menjadi sasaran penerapan database shim. |
| `Layers` | Lapisan kompatibilitas, sebagaimana tertulis (`~ RUNASADMIN`). |
| `LastSeenUTC` | Waktu terakhir sebuah program memuat modul CIT. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, berkas, dan waktu instalasi database shim. |
| `Details` | Nama modul CIT, nama nilai database CIT, id media lepas pasang sebuah program (`SignMedia`), `DatabaseDescription` dan `DatabaseType` database shim. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Asal catatan, kapan kunci tersebut terakhir berubah, dan apakah berada di bawah `Wow6432Node`. |
| `DataRaw` | Data `SACP` dari nilai `Store`, atau data yang tidak sesuai dengan yang seharusnya (hex). |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Data yang tidak dapat didekode atau dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Tinjau setiap database shim kustom: berkas `.sdb`-nya, waktu instalasinya, dan program yang menjadi sasaran
   penerapannya; korelasikan dengan `sdbinst.exe` di Prefetch, Amcache, dan peristiwa pembuatan proses.
2. Cari program di folder yang dapat ditulisi pengguna di antara entri Program Compatibility Assistant, serta
   program dari media lepas pasang.
3. Periksa `RUNASADMIN` dan lapisan lain pada program yang tidak biasa.
4. Korelasikan dengan ShimCache, Amcache, BAM, Prefetch, dan UserAssist untuk waktu eksekusi.

## Detail lebih lanjut { #further-details }

- [Microsoft: Application compatibility database](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
