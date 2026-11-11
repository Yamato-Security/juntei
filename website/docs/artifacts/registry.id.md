# Hive Registry Windows { #windows-registry-hives }

## Apa itu hive registry { #what-registry-hives-are }

Windows Registry menyimpan konfigurasi sistem dan pengguna dalam berkas hive biner. Key membentuk sebuah pohon;
nilai di bawah sebuah key berisi data bertipe. Artefak registry mendeskripsikan persistensi, perangkat lunak terinstal,
layanan, akun, konfigurasi jaringan, aktivitas pengguna, pengaturan keamanan, dan hampir setiap bagian
dari kondisi Windows.

Pembaca generik Juntei mendekode hive primer apa pun, memutar ulang log transaksi, melaporkan kerusakan, dan dapat
memulihkan record deleted/free-cell, cell-slack, remnant, pre-replay, dan yang hanya ada di log transaksi.

<span class="evidence-strength">Kesimpulan terbaik:</span> hive berisi, atau menyimpan secara dapat dipulihkan,
kondisi key/value tersebut. Waktu penulisan terakhir key menunjukkan bahwa key berubah, bukan nilai mana yang berubah atau alasannya.

## Hive umum dan lokasinya { #common-hives-and-locations }

| Hive | Path umum | Contoh bukti |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Layanan, driver, control set, nama host, mounted device, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Perangkat lunak terinstal, TaskCache, profil, konfigurasi Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | Akun lokal dan metadata akun |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Kebijakan keamanan lokal/metadata rahasia (bergantung pada akses) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Konfigurasi profil sistem default |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Preferensi pengguna, Run key, RecentDocs, aktivitas aplikasi |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Shell class, aktivitas COM/shell pengguna |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Inventaris aplikasi (gunakan `parse-registry amcache` untuk keluaran semantik) |

Kumpulkan berkas `.LOG`, `.LOG1`, dan `.LOG2` yang berdampingan. Hive yang disalin saat Windows sedang berjalan bisa saja
dirty; log transaksi berisi kondisi yang telah di-commit yang belum tercermin pada berkas primer saja.

## Peringatan bagi analis { #analyst-cautions }

- Timestamp penulisan terakhir key berlaku untuk **key**, bukan untuk nilai individual.
- Nilai registry biasanya tidak memiliki timestamp tersendiri.
- `CurrentControlSet` adalah alias analis/Windows yang di-resolve melalui `SYSTEM\Select`; ini bukan
  key yang benar-benar tersimpan.
- Record yang terhapus dapat bersifat parsial, usang, terduplikasi di berbagai sumber pemulihan, atau tertimpa.
- Record yang hanya ada di log transaksi menunjukkan kondisi historis/antara. Jelaskan `recovered_from`.
- Hasil `absent_value` berarti key yang di-query ada tetapi tidak memiliki nilai tersebut dalam kondisi yang diurai;
  ini tidak membuktikan bahwa nilai tersebut tidak pernah ada.
- String, path, dan command line merupakan konfigurasi, bukan bukti bahwa tindakan yang dikonfigurasi telah dijalankan.
- Hive yang dirty tanpa log yang dapat diterapkan dapat menghilangkan atau menampilkan kondisi yang usang.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host` | Nama host yang diturunkan dari konteks sistem yang tersedia; kosong jika Juntei tidak dapat mengidentifikasinya dengan aman. |
| `HiveType` | Kategori hive yang terdeteksi (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, dll.). Gunakan untuk menginterpretasikan path. |
| `SourcePath` | Hive input yang tepat. Penting ketika mode direktori menggabungkan beberapa host/pengguna. |
| `RecordType` | `key`, `value`, `default_value`, atau `absent_value`. Ketiadaan hanya berlaku untuk query/kondisi tersebut. |
| `KeyPath` | Path di bawah root hive. `[unknown]` menandai silsilah terhapus yang tidak dapat di-resolve. |
| `KeyLastWrittenUTC` | Modifikasi terakhir key secara keseluruhan. Ini tidak dapat mengidentifikasi nilai mana yang berubah. |
| `ValueName` | Nilai bernama; kosong untuk nilai default/tanpa nama. |
| `ValueType` | Tipe registry (`REG_SZ`, `REG_DWORD`, `REG_BINARY`, dll.). Tipe memengaruhi interpretasi. |
| `ValueData` | Nilai yang di-render sesuai tipenya. Multi-string tetap berupa list di JSON. |
| `DataSize` | Ukuran yang dicatat oleh nilai. Bandingkan dengan byte mentah dan `Problems`. |
| `ValueDataRaw` | Byte persis ketika dekode gagal atau `--raw-data` digunakan. Simpan untuk validasi. |
| `ValueU64` | Representasi numerik utuh untuk DWORD/QWORD tanpa kehilangan presisi JavaScript. |
| `ValueFiletimeUTC` | Render opsional QWORD sebagai FILETIME. Hanya bermakna jika skema key mendefinisikan FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only`, atau `transaction_log_recovered`. Menentukan tingkat keyakinan. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay`, atau entri log seperti `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged`, atau `not_applied` untuk konteks record/hive ini. |
| `LogsApplied` | Nama berkas log transaksi yang diputar ulang Juntei. Catat ini dalam laporan. |
| `Problems` | Kerusakan, cell yang hilang, ketidaksesuaian tipe, list/induk yang tidak dapat di-resolve, atau keterbatasan lainnya. |

## Interpretasi status pemulihan { #recovery-status-interpretation }

| Status | Arti |
|---|---|
| `deleted_complete` | Key/value yang terhapus berhasil dipulihkan dengan path/data yang diketahui secara lengkap. Tetap bersifat historis, bukan kondisi aktif. |
| `deleted_partial` | Sebagian path, list nilai, nilai, atau data telah tertimpa/tidak dapat dibaca. Jangan mengisi celah dari cell yang digunakan ulang. |
| `residual_value_only` | Nilai bertahan tanpa konteks key yang memadai; `KeyPath` bisa tidak ada. |
| `transaction_log_recovered` | Kondisi hanya ada di versi pre-replay/log dan tidak ada di record allocated/free saat ini. |

## Strategi investigasi { #investigation-strategy }

1. Mulailah dengan key semantik yang tepat jika diketahui; gunakan pencarian regex/data untuk penemuan.
2. Periksa `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom`, dan `Problems` sebelum konten.
3. Interpretasikan data sesuai skema key yang terdokumentasi; `REG_BINARY` tidak mendeskripsikan dirinya sendiri.
4. Korelasikan konfigurasi dengan bukti runtime (log peristiwa, Prefetch, SRUM, layanan/tugas).
5. Simpan data mentah untuk field yang baru atau diperdebatkan.

## Detail lebih lanjut { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Spesifikasi format registry karya Maxim Suhanov](https://github.com/msuhanov/regf)
- [Parser registry yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: Tipe nilai registry](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
