# Log transaksi NTFS (`$LogFile`) { #ntfs-transaction-log-logfile }

## Apa itu dan mengapa penting { #what-it-is-and-why-it-matters }

NTFS adalah sistem berkas yang biasanya digunakan Windows untuk volume sistemnya. Untuk pulih dari crash, NTFS
mencatat perubahan yang hendak dilakukannya pada metadata sistem berkas di dalam berkas volume tersembunyi bernama
`$LogFile`. Setiap pembaruan dapat menyimpan byte **redo** (status baru) dan byte **undo** (status
sebelumnya yang diperlukan untuk membatalkan perubahan). Ini terpisah dari Windows Event Logs, berkas registry `.LOG1`,
dan [Jurnal USN](usn-journal.md).

Bagi investigator, status sebelumnya sangat berguna. Catatan undo dapat mempertahankan
nama berkas sebelum penggantian nama, entri direktori yang telah dihapus, atau timestamp sebelum pembaruan metadata.
Beberapa catatan juga berisi byte yang ditulis NTFS ke Jurnal USN, termasuk catatan yang sudah tidak ada di
`$J` yang dikumpulkan. Namun, banyak transaksi berkaitan dengan aktivitas rutin sistem operasi dan tidak
mendeskripsikan serangan.

**Contoh:** sebuah entri indeks yang dihapus menyebutkan `report.txt`; entri yang ditambahkan kemudian menyebutkan
`report.txt.encrypted`. Mencocokkan referensi berkas lengkap, induk, rantai transaksi, dan artefak
lainnya dapat mendukung adanya penggantian nama. Kedua nama itu saja tidak mengidentifikasi ransomware, operatornya,
atau waktu enkripsi yang persis.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Setiap volume NTFS memiliki log-nya sendiri. Akuisisi dengan alat yang membaca metadata NTFS atau mengekspornya
dari image forensik atau snapshot; penyalinan berkas Windows biasa mungkin tidak dapat mengaksesnya. Kumpulkan
`$MFT` dan `$Extend\$UsnJrnl:$J` dari volume yang sama sedekat mungkin waktunya. Pertahankan berkas
asli dan hash-nya. Akuisisi yang dilakukan belakangan dapat berisi rentang aktivitas yang berbeda.

`$LogFile` adalah **log sirkular**: catatan baru menimpa catatan lama. Retensi bergantung pada ukuran log
dan beban kerja, dan bisa hanya beberapa menit atau jam. Ketiadaan sebuah nama berkas atau operasi bukan
bukti bahwa hal tersebut tidak pernah terjadi.

## Apa yang dibuktikan sebuah catatan, dan peringatan { #what-a-record-proves-and-cautions }

- Sebuah pembaruan mencatat metadata yang dicatat NTFS untuk pemulihan. Satu entri redo tidak membuktikan bahwa
  seluruh tindakan tingkat tinggi telah selesai atau bahwa orang tertentu yang memulainya. Korelasikan
  catatan terkait, penyelesaian transaksi, sistem berkas yang dihasilkan, dan bukti independen.
- **Tidak ada timestamp transaksi umum.** Timestamp yang tertanam mendeskripsikan metadata berkas, yang
  terkadang usang, disalin, atau dimodifikasi penyerang. Jangan menempatkan `CreatedOn` pada timeline insiden
  sebagai waktu penggantian nama atau perubahan timestamp. Catatan USN dapat menyediakan timestamp tersendiri.
- LSN (log sequence number) mengurutkan catatan **di dalam log satu volume**. LSN bukan waktu jam
  dan tidak boleh dibandingkan antar-host atau antar-volume. Catatan dari putaran circular buffer yang lebih lama
  dapat bertahan berdampingan dengan aktivitas yang lebih baru.
- Slot tabel transaksi digunakan ulang. Kelompokkan berdasarkan `TransactionFirstLsn`, `TransactionId`, identitas
  klien, dan sumber, alih-alih hanya `TransactionId`. Rantai dapat menunjuk ke luar data yang tersimpan.
- Referensi berkas berisi nomor entri **dan nomor urut (sequence number)**. Jika tidak, penggunaan ulang entri MFT dapat
  membuat berkas yang tidak berkaitan tampak seperti berkas yang sama.
- Path yang disediakan oleh `$MFT` mendeskripsikan **status MFT yang dikumpulkan**, bukan path historis yang
  direkonstruksi. Juntei mencocokkan nomor urut dan membiarkan komponen yang tidak diketahui tetap eksplisit. Sebuah induk dapat
  telah diganti namanya setelah transaksi, dan hard link dapat memberikan lebih dari satu nama saat ini.
- `TargetInferred` berarti pemetaan open-attribute berasal dari rentang log yang berbeda.
  Perlakukan pemetaan tersebut dengan kehati-hatian ekstra. Entri indeks yang belum diresolusi dapat didekode hanya berdasarkan
  tata letaknya; validasi namanya dengan byte mentah dan sumber lain.
- Halaman yang robek (torn page) dilewati; catatan di sekitarnya yang masih dapat dibaca tetap tersedia. `Truncated` menandai
  catatan yang kelanjutannya hilang. Tinjau diagnostik parser sebelum memperlakukan keluaran sebagai
  lengkap. Masukan berukuran nol adalah error; log yang belum pernah ditulis dan seluruhnya `FF` memang secara sah tidak memiliki baris.
- Parser **tidak memutar ulang** transaksi untuk merekonstruksi MFT historis. Parser tidak
  memulihkan konten berkas non-resident sembarang atau mengeksekusi data apa pun yang dikumpulkan. Halaman DAX yang dilindungi CRC
  tidak didukung. Validasi dengan bukti nyata terutama mencakup halaman log 4 KiB.

## Mengurai dengan Juntei { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Lihat [referensi perintah](../commands/filesystem.md#parse-ntfs-logfile) untuk semua opsi. Penemuan
direktori menerima `$LogFile`, `LogFile`, bentuk `.bin`-nya, dan nama seperti `C_$LogFile` atau
`C_$LogFile.bin`, tanpa membedakan huruf besar/kecil. Satu berkas yang diberikan dengan `-f` boleh memiliki nama apa pun.

Pemasangan MFT otomatis mengikuti prefiks volume pada hasil ekspor: `C_$LogFile` menggunakan `C_$MFT`, dan
`D_$LogFile.bin` pertama-tama mencoba `D_$MFT.bin`, lalu `D_$MFT`. Log berprefiks tidak pernah menggunakan `$MFT` polos
sebagai fallback. `LogFile` tanpa prefiks juga menerima `MFT` atau `$MFT`. Gunakan `-m` bersama `-f` untuk memilih
MFT yang cocok secara eksplisit bila hasil ekspor Anda memiliki skema penamaan lain.

Keluaran berupa satu catatan per LSN, termasuk catatan update, checkpoint, dan klien yang tidak dikenal. JSON adalah
stream objek yang diformat rapi (pretty-printed); JSONL memiliki satu objek per baris. CSV memiliki nama field yang sama;
array dan field terstruktur berupa JSON ringkas di dalam sel CSV. Skalar yang tidak diketahui kosong di
CSV dan `null` di JSON. Array kosong berarti tidak ada item yang didekode, bukan bukti bahwa tidak ada item.

Satu log utuh beserta catatan hasil uraiannya disimpan di memori pada satu waktu. Meresolusi path MFT menambah
memori dan pekerjaan. `--raw-data` dapat sangat memperbesar ukuran keluaran. Di `auto-parse`, gunakan `-T` untuk
membatasi proses parser yang berjalan bersamaan pada mesin dengan memori terbatas.

Jika MFT yang berdampingan tidak dapat dibaca, Juntei memberikan peringatan dan mengekspor log tanpa path; `MftSource`
kemudian tetap null. Kegagalan pengayaan ini tidak membuang bukti primer yang dapat dibaca.
Kecocokan MFT yang tidak ada atau ambigu juga menghasilkan peringatan dan hitungan `Logs without a matching MFT`
di ringkasan; keduanya tidak dihitung sebagai kegagalan penguraian.

## Field keluaran { #output-fields }

### Identitas, integritas, dan asal-usul catatan { #record-identity-integrity-and-provenance }

| Field | Makna dan interpretasi |
|---|---|
| `SourceFile`, `FileOffset` | Path masukan dan offset byte tempat header catatan ini dibaca; gunakan untuk memvalidasi byte. |
| `LSN` | Log sequence number catatan. Nilai besar adalah bilangan bulat, bukan timestamp. |
| `PreviousLSN`, `UndoNextLSN` | Tautan ke catatan sebelumnya dan catatan rollback. Nol berarti tidak ada tautan; tautan bukan nol dapat menunjuk ke luar data yang tersimpan. |
| `TransactionID`, `TransactionFirstLSN` | Slot tabel transaksi dan LSN paling awal yang tertaut ke transaksi ini; slot saja tidak unik. |
| `RecordType` | `ClientRecord` (update), `ClientRestart` (checkpoint), atau `Other`. |
| `ClientIndex`, `ClientSequence` | Identitas klien log; biasanya NTFS. Ini bukan akun pengguna atau ID proses. |
| `Flags`, `SpansPages` | Flag catatan mentah dan apakah data klien berlanjut ke halaman lain. Melintasi beberapa halaman adalah hal normal. |
| `Truncated` | True jika parser menemukan kelanjutan yang hilang untuk LSN ini. Ini tidak merangkum semua kerusakan di seluruh berkas. |
| `ClusterSize`, `ClusterSizeSource` | Byte per cluster yang digunakan untuk offset target; sumbernya `Caller`, `Checkpoint`, atau `Inferred`. Berikan nilai sebenarnya dengan `--cluster-size` jika diketahui. |
| `LogVersion` | Versi dari halaman restart terbaru yang dapat dibaca, biasanya `1.1` atau `2.0`; null jika tidak ada halaman restart yang dapat dibaca yang tersisa. |
| `MFT-Source` | MFT yang digunakan untuk pengayaan path saat ini, bila ada. MFT tersebut harus berasal dari volume yang sama. |

Pada JSON/JSONL `auto-parse`, `Comp` dan `Artifact` ditambahkan seperti pada parser lainnya. Log `.txt`
milik tugas tersebut menyimpan diagnostik; manifest menyatakan apakah tugas berhasil, gagal, atau dilewati.
Kerusakan yang masih dapat diselamatkan dapat menghasilkan tugas yang berhasil dengan hitungan `Parse problems` bukan nol di log-nya.

### Operasi dan target { #operations-and-target }

| Field | Makna dan interpretasi |
|---|---|
| `RedoOperation`, `UndoOperation` | Nama operasi NTFS, atau heksadesimal untuk kode yang tidak dikenal. Redo mendeskripsikan pembaruan yang dimaksud; undo mendeskripsikan kebalikannya. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Untuk operasi pada file record: nomor catatan MFT yang terdampak, tanpa nomor urut yang disimpulkan. Selain itu: referensi berkas yang memiliki open attribute yang diresolusi. Pembaruan indeks direktori inline menargetkan direktori, bukan child yang dideskripsikan oleh entrinya. |
| `TargetAttributeType`, `TargetAttributeName` | Tipe numerik dan nama open attribute yang digunakan oleh operasi. Untuk operasi file record, ini biasanya `$DATA` milik `$MFT`. Tipe umum: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Path MFT saat ini bila referensi lengkap target diketahui, atau komponen yang tidak diketahui. Kosong untuk operasi file record yang generasi historisnya tidak diketahui; nomor catatan saja tidak boleh diresolusi ke penghuninya saat ini. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | Berkas yang memiliki open attribute yang digunakan operasi untuk menulis, beserta path MFT saat ini. Untuk operasi file record, field ini mempertahankan konteks `$MFT` secara terpisah dari catatan yang terdampak. |
| `TargetAttribute` | Offset dalam tabel open-attribute log; ini berbeda dari tipe atribut atau pengenal MFT. |
| `TargetInferred` | True jika pemetaan target berasal dari rentang log lain. Target null tetap belum diresolusi meskipun flag ini false. |
| `TargetVCN`, `Lcns` | Virtual cluster dalam target dan nomor cluster fisik; berguna untuk validasi tingkat rendah, bukan untuk penentuan waktu. |
| `RecordOffset`, `AttributeOffset` | Offset atribut/entri dan pembaruan di dalamnya. |
| `ClusterBlockOffset`, `TargetBlockSize` | Offset dan ukuran target dalam satuan 512 byte. |
| `OffsetInTarget` | Offset byte terhitung di dalam file record atau stream non-resident yang terdampak. Bergantung pada ukuran cluster dan ukuran catatan MFT. |
| `RedoLength`, `UndoLength` | Panjang data yang dinyatakan. Operasi zeroing dapat menyatakan suatu panjang tanpa membawa byte payload sebanyak itu. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Byte payload yang benar-benar tersedia untuk didekode. Bandingkan dengan panjang yang dinyatakan dan semantik operasi. |

### Kolom metadata berkas praktis { #convenient-file-metadata-columns }

Kolom-kolom ini menampilkan satu nama berkas non-DOS yang diutamakan dari sisi redo, dengan fallback ke undo.
Jika kedua sisi tidak membawa nama berkas, field pembaruan timestamp mengutamakan timestamp redo, lalu undo.
Pembaruan entri direktori juga menampilkan waktu, ukuran, dan flag-nya secara langsung, dengan mengutamakan redo daripada
undo. Atribut `$DATA` yang didekode menampilkan ukurannya bahkan ketika tidak ada nama berkas. Ukuran `$DATA` tanpa nama
dari file record yang didekode diutamakan daripada ukuran nama berkasnya yang mungkin sudah usang.
Kolom-kolom ini adalah alat bantu penelusuran; gunakan field sisi yang terpisah untuk perbandingan sebelum/sesudah yang lengkap.

| Field | Makna dan interpretasi |
|---|---|
| `MetadataSide` | `Redo` atau `Undo`, mengidentifikasi sisi yang menyuplai kolom nama berkas/timestamp praktis. |
| `EntryNumber`, `SequenceNumber` | Berkas yang dideskripsikan oleh metadata nama berkas/file record/atribut yang didekode atau oleh pembaruan timestamp `$STANDARD_INFORMATION` parsial, beserta nomor urut bila diketahui. Kosong untuk pembaruan entri direktori yang tidak mengidentifikasi child-nya, dan untuk pembaruan ukuran/run-list mentah. Gunakan `TargetEntryNumber` untuk catatan yang terdampak. |
| `Filename` | Nama terdekode yang diutamakan; semua nama tetap ada di array sisi, termasuk alias DOS dan hard link. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Referensi induk dan path MFT saat ini; bukan rekonstruksi path historis. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | Waktu metadata dalam UTC, hingga 100 ns. Ini bukan timestamp transaksi. Waktu yang tidak ada/tidak valid tetap kosong/null. |
| `FileSize`, `FileAttributes` | Ukuran dan flag Windows numerik dari metadata yang dipilih; nilai entri direktori bisa sudah usang. Atribut `$DATA` dan file record yang didekode dapat menyuplai ukuran yang lebih langsung. |
| `AttrType` | Tipe numerik dari catatan atribut yang didekode; 48 untuk pembaruan metadata entri direktori yang didekode. Terpisah dari open attribute yang ditargetkan oleh operasi. |

### Metadata dan payload redo/undo lengkap { #complete-redoundo-metadata-and-payloads }

| Field | Makna dan interpretasi |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Array berisi setiap `$FILE_NAME` yang didekode pada sisi tersebut, mempertahankan nama yang ditambahkan/baru dibandingkan dengan yang dihapus/lama. |
| `RedoTimes`, `UndoTimes` | Timestamp `$STANDARD_INFORMATION` yang ditulis oleh masing-masing sisi; pembaruan parsial membiarkan timestamp yang tidak berubah tetap tidak ada alih-alih menebaknya. |
| `RedoDecoded`, `UndoDecoded` | Informasi file record, atribut, entri indeks, atau entri direktori yang terstruktur, diidentifikasi oleh `Kind`. Null dapat berarti operasi yang tidak didukung atau byte yang tidak mencukupi. |
| `JournalWriteOffset`, `JournalWriteLength` | Offset dan jumlah byte data redo yang menulis `$DATA:$J`. Ini mengidentifikasi kemungkinan sumber catatan USN yang jika tidak demikian akan hilang; ini tidak mengekspor jurnal yang direkonstruksi. |
| `Checkpoint` | Versi checkpoint, LSN awal, ukuran cluster, serta LSN/panjang dump open-attribute, attribute-name, dirty-page, dan transaction-table. |
| `OtherRecordType` | Tipe catatan klien mentah untuk badan catatan yang tidak dikenali atau tidak dapat didekode. |
| `RedoData`, `UndoData`, `OtherData` | Payload hex hanya dengan `--raw-data`; selain itu null/kosong. Simpan ini untuk validasi struktur yang tidak didukung atau rusak. |

Setiap objek nama berkas mencakup referensi berkas dan induk (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, `ParentPath` saat ini, empat timestamp,
`Size`, `AllocatedSize`, `Attr`, dan `ReparseValue`. `NameEscaped` mengidentifikasi UTF-16 tidak valid
yang direpresentasikan dengan escape. Nilai reparse dapat berupa reparse tag atau ukuran extended attribute.

Objek timestamp mempertahankan `Utc` sekaligus bilangan bulat `Filetime`; nilai sentinel atau tidak valid dapat
memiliki representasi UTC null sambil tetap mempertahankan bilangan bulat aslinya. `RedoTimes` dan `UndoTimes`
berisi `Created`, `Modified`, `Changed`, dan `Accessed`; nilai anggota yang tidak ada adalah null.

Objek `FileRecord` yang didekode mencakup nomor, nomor urut, jumlah hard link, flag, referensi dasar,
apakah hanya header yang tersisa, dan atribut yang ada. Setiap atribut yang didekode mempertahankan
tipe, nama, status resident, flag, ID, ukuran, dan ukuran yang dialokasikan. `$STANDARD_INFORMATION` menambahkan
empat waktu, atribut, ID pemilik/keamanan, kuota, dan USN; ID ini memerlukan resolusi terpisah dan
bukan bukti pengguna yang mengubah berkas. Entri indeks mencakup flag, panjang kunci, dan
nama berkas opsional. Pembaruan entri direktori mempertahankan empat waktu, ukuran, atribut, dan nilai
reparse bahkan ketika nama itu sendiri tidak tercatat di log.

## Alur kerja investigasi { #investigation-workflow }

1. Tinjau diagnostik untuk halaman restart yang hilang, halaman yang robek, dan catatan yang terpotong.
2. Telusuri `RedoFileNames` dan `UndoFileNames` untuk nama berkas atau ekstensi yang mencurigakan.
3. Cocokkan nomor entri **dan nomor urut**, lalu periksa rantai LSN dan operasi transaksi tersebut.
4. Bandingkan timestamp redo dan undo untuk melihat perubahan. Penyalinan, ekstraksi, dan pemulihan rutin juga
   dapat mengubah timestamp; perbedaan merupakan petunjuk, bukan putusan timestomping otomatis.
5. Korelasikan dengan [Jurnal USN](usn-journal.md), [$MFT](mft.md), dan [$I30](i30.md). Gunakan log
   peristiwa, [Prefetch](prefetch.md), dan artefak aktivitas pengguna untuk menetapkan eksekusi atau atribusi.

Sebagai contoh, temukan semua catatan yang membawa nama berkas terenkripsi di salah satu sisi:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Ini adalah pencarian nama berkas, bukan detektor ransomware yang lengkap. Untuk memeriksa payload jurnal,
ekspor dengan `--raw-data` dan pilih catatan dengan `JournalWriteOffset` yang tidak null; hex `RedoData`
merepresentasikan byte yang ditulis pada offset tersebut. Mengurai dan menghapus duplikasi catatan USN yang tertanam itu
adalah langkah tersendiri. Juntei belum menggabungkannya dengan keluaran `parse-ntfs usn-journal`.

## Detail lebih lanjut { #further-details }

- [Crate logfile ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): struktur yang didukung, korpus validasi, dan keterbatasan yang diketahui.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): implementasi Rust upstream.
- [Dokumentasi format NTFS libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation): halaman log, fixup, dan struktur metadata.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): riset dan implementasi penguraian NTFS yang independen.
- [LogFileParser oleh Joakim Schicht](https://github.com/jschicht/LogFileParser): analisis log transaksi yang mendetail.
- [ntfs-linker oleh Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker): mengorelasikan sumber-sumber NTFS menjadi riwayat berkas.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): sampel NTFS Windows nyata.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): implementasi NTFS dan volume uji publik.
