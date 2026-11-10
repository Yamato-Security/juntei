# Linimasa gabungan { #super-timeline }

`super-timeline` mengubah artefak yang diurai menjadi peristiwa dengan skema umum. Output **tidak diurutkan secara default**: output mengikuti urutan file input yang ditemukan dan urutan proyeksi setiap parser, terlepas dari urutan penyelesaian pekerja. Tambahkan `--sort` untuk pengurutan kronologis dan konsolidasi duplikat. Catatan Prefetch dapat berisi beberapa waktu eksekusi; catatan MFT dapat berisi delapan atau lebih stempel waktu file. Masing-masing menjadi peristiwa yang diberi label terpisah. Hal ini memudahkan untuk membandingkan bukti dari berbagai sumber tanpa kehilangan arti sebenarnya dari setiap stempel waktu.

Mulailah dari hasil [`auto-parse`](auto-parse.md), bukan direktori bukti asli:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Simpan hasil parsing, manifes, dan bukti asli. Linimasa hanya proyeksi analisis, bukan penggantinya atau penyimpanan Plaso asli.

## Penggunaan { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` dapat dipakai sendiri atau berulang untuk mengimpor waktu dan konteks aturan tanpa menjalankan Hayabusa. Peristiwa informasional dengan `Details` berupa string didukung. `--artifact mft,prefetch` membatasi artefak. Batas waktu inklusif dan memerlukan offset UTC eksplisit atau `Z`.

### Memilih masukan { #choosing-an-input }

| Masukan | Cara identifikasi |
|---|---|
| `-d DIR` | `manifest.json` menyediakan tugas utama berhasil, ID artefak, komputer dan berkas tepat. Tugas sekunder `auto-parse -j` dikecualikan agar tidak diimpor ganda. |
| `-f FILE` | JSON/JSONL asli memakai `Artifact` dan `Comp` tertanam. Ekspor tanpa label memerlukan `--input-artifact`; CSV mandiri selalu memerlukannya. |
| `--computer NAME` | Nama cadangan jika komputer tidak tercatat; komputer tak diketahui ditandai `Unknown`. |
| `--hayabusa FILE` | JSONL dengan waktu Hayabusa, metadata aturan, komputer dan `EvtxFile` jika tersedia. |

Manifes kini mendukung pemindahan direktori: jalur absolut dipetakan ke lokasi baru. Masukan dan target symlink harus tetap di dalamnya. Direktori lama Amcache, SRUM, Jump Lists didukung; direktori kategori memerlukan `OutputFiles` eksplisit.

Jalur relatif terhadap keluaran dan ekspor banyak tabel yang diratakan didukung. Semua artefak yang dijadwalkan parsing otomatis dikenali, termasuk browser, WebCache, Outlook dan UAL.

Untuk citra VHDX/E01, referensi berisi citra dan jalur internal; nama citra tetap dalam konteks. Direktori staging sementara tidak diperlukan saat membaca hasil.

Sumber mentah diambil dari catatan atau perintah manifes yang tidak ambigu. Dengan `-d`, referensi sumber mentah bersifat relatif terhadap direktori bukti, dan `Details.ParsedFile` serta ringkasan `InputFiles` bersifat relatif terhadap direktori yang diurai. Nama keluaran/laporan ringkasan bersifat relatif terhadap direktori keluaran garis waktu. Tabel SRUM CSV ditetapkan ke database hanya ketika host memiliki satu SRUDB.dat yang dapat diidentifikasi secara unik. Jika tidak, `SourceFile` menunjuk ke file yang diurai dan `Details.RawSourceAvailable` adalah `false`.

Input mandiri direferensikan berdasarkan nama filenya. Input dengan nama file yang sama menggunakan referensi `./Inputs/<ID>/<filename>` yang berbeda. Jalur sumber mentah absolut di luar akar bukti yang diketahui menggunakan pengidentifikasi `./Sources/<ID>/<filename>`. Referensi buram ini mempertahankan perbedaan sumber tanpa mengekspor nama direktori absolut analis; lihat catatan asli menggunakan `ParsedFile` dan `InputRecord` untuk sumber aslinya. Jalur yang dicatat dalam bukti, seperti jalur yang dapat dieksekusi Windows, tetap tidak berubah.

## Bidang peristiwa bersama { #common-event-fields }

| Bidang | Arti |
|---|---|
| `Timestamp` | UTC RFC 3339 dengan sembilan digit pecahan untuk pengurutan konsisten; padding tidak menambah presisi. |
| `Comp` | Komputer yang terkait dengan rekaman masukan. |
| `Artifact` | ID artefak stabil seperti `mft`, `prefetch`, `srum` atau `hayabusa`. |
| `TimestampType` | Arti waktu, misalnya `MFT SI Data modified`. |
| `User` | Akun atau SID yang tercatat jika tersedia; dapat kosong. |
| `Path` | Berkas, kunci registry atau objek khusus artefak. |
| `Message` | Deskripsi singkat untuk penampil linimasa. |
| `SourceFile` | Referensi sumber mentah relatif bukti, atau pengidentifikasi sumber relatif ketika akarnya tidak diketahui; jika tidak, fallback file yang diurai dijelaskan di atas. |
| `SourceField` | Nama bidang atau lokasi bertingkat persis yang menyediakan waktu. |
| `RecordID` | Identitas rekaman artefak, tidak harus ID rekaman EVTX ID. |
| `Details` | Konteks terstruktur: waktu asli, presisi, dasar/peran waktu, berkas, rekaman/lokasi dan metadata pemulihan/aturan. |

Dalam CSV, `Details` adalah sel JSON berpetik; dalam JSONL berupa objek. Profil Timesketch mengganti `Timestamp`, `TimestampType`, `Message` menjadi `datetime`, `timestamp_desc`, `message` sambil mempertahankan asal lainnya. Ini [tiga bidang wajib Timesketch](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` menyimpan konteks terpilih, bukan seluruh data biner atau bidang asli. Struktur panjang dibatasi dan ditandai `ProjectionOmitted`. Cari `Details.ParsedFile`, `InputRecord`, `Location` untuk rekaman lengkap. Nomor CSV menghitung rekaman CSV setelah header, bukan baris fisik di sel. Untuk JSONL berupa nomor baris; JSON berupa nomor dokumen dan lokasi bertingkat.

## Memahami waktu { #what-the-times-mean }

Waktu menggambarkan objek tercatat, bukan bukti eksekusi atau serangan dengan sendirinya. Perbedaannya:

| Artefak | Interpretasi linimasa |
|---|---|
| MFT | Pisahkan modifikasi, akses, perubahan metadata, dan waktu pembuatan SI dan FN; atribut bernama FN tetap berbeda. Dengan `--sort`, bidang SI yang berulang pada baris nama file dikonsolidasikan. Waktu file dasar bersama baris ADS dikecualikan di kedua mode. Nama slack rekaman FILE yang dipulihkan menyimpan peristiwa FN yang dipulihkan secara terpisah. |
| I30 | Metadata FN/indeks dengan konteks slack/pemulihan, bukan waktu penulisan indeks direktori. |
| Jurnal USN | Waktu jurnal dan alasan pembaruan; satu rekaman dapat berisi beberapa perubahan tanpa awal yang tepat. |
| LogFile | Snapshot waktu redo/undo dan nama berkas tetap terpisah. LSN dan offset berkas adalah ID, bukan tanggal. |
| Prefetch | Waktu eksekusi yang dicatat. Dengan `--sort`, ini digabungkan dengan ekspor garis waktu Prefetch yang cocok. Metadata pembuatan volume yang direferensikan diberi label secara terpisah. |
| Database SQLite browser | Kunjungan, kunjungan terakhir yang dipulihkan, awal/akhir unduhan, waktu cookie dan penggunaan/penghapusan formulir. Akhir unduhan tidak membuktikan sukses/eksekusi; kedaluwarsa cookie adalah konfigurasi. Status pemulihan, keyakinan dan asal disimpan. |
| Internet Explorer / Edge lama WebCache | Akses, pembuatan/modifikasi, kedaluwarsa, sinkronisasi, unduhan, cookie dan pemeliharaan kontainer. Administrasi cache tidak otomatis membuktikan kunjungan pengguna. |
| Outlook PST, OST, MSG dan OFT | Pengiriman/penerimaan, pembuatan/modifikasi folder/item/lampiran, jadwal kalender/tugas/pengingat dan properti waktu bertipe eksplisit. Jadwal tidak membuktikan rapat, pengingat atau eksekusi; penerimaan tidak membuktikan pembacaan. |
| Pencatatan Akses Pengguna (UAL) | Akses pertama/terakhir, penyisipan/pembuatan database, kemunculan pertama/terakhir dan boot sistem. Peran, akun, alamat sumber dan penghitung harian adalah konteks, bukan login tengah malam buatan. Identitas dan baris DNS tanpa waktu tetap dalam laporan pendamping. |
| Amcache | Waktu tulis kunci inventaris, volume induk, pemasangan dan metadata tautan PE. Kehadiran inventaris atau LinkDate tidak membuktikan eksekusi. |
| ShimCache | Modifikasi berkas, pembaruan cache dan penulisan kunci bukan otomatis waktu eksekusi. |
| SRUM | Waktu/bucket penyedia, awal koneksi, interval eksplisit dan ID aplikasi. Byte, durasi dan penghitung bukan tanggal. |
| LNK | Waktu target MAC, pembuatan ID pelacakan dan shell item adalah metadata target, bukan pasti pembukaan shortcut. |
| Jump Lists | Penggunaan terakhir, ID pelacakan dan waktu target LNK tertanam. Waktu kontainer pada sistem koleksi diabaikan secara default. |
| Karantina Defender | Waktu entri dan pembuatan/akses/penulisan sumber daya. Waktu ekstraksi bukan aktivitas endpoint. |
| BAM, UserAssist, FeatureUsage | Aktivitas tercatat dipisahkan dari penulisan kunci. Penghitung tanpa waktu tidak menghasilkan peristiwa. |
| RecentDocs, MRU, Buka/Simpan MRU, ShellBags, Office | Waktu penggunaan/kepercayaan/shell dan konteks kunci. Waktu kunci daftar bukan penggunaan terakhir setiap item. |
| SAM dan rahasia LSA | Metadata login/kata sandi/kedaluwarsa/rahasia dan kunci tidak menggantikan log autentikasi. |
| Layanan, TaskCache, persistensi registry, BCD | Metadata kunci/konfigurasi dan waktu tugas eksplisit. Penulisan kunci layanan tidak membuktikan eksekusinya. |
| Jaringan dan USB | Profil/sewa jaringan, pemasangan/kedatangan/pelepasan perangkat dan kunci. Waktu pengguna USB MountPoints2 asli tetap merujuk hive pengguna terkait. |
| Informasi sistem, program terpasang, AppCompat, riwayat aplikasi | Metadata pemasangan, shutdown, inventaris/aktivitas dan kunci. Array profil/versi asli mempertahankan waktu bertingkat. |
| Kebijakan audit, keamanan, MuiCache, registry umum | Penulisan terakhir kunci jika ada; nilai/penghitung tanpa waktu tetap dalam masukan asli. |
| Persistence Hunter | Penulisan kunci dan perubahan terkait. Temuan bukan otomatis instalasi bertanggal atau putusan kompromi. |
| Hayabusa | Waktu peristiwa/alert dengan aturan, tingkat, kanal, ID dan konteks. Aturan serta korelasi berbeda tidak digabung. |

Ekspor default mempertahankan semua proyeksi stempel waktu, termasuk metadata berulang. Ia tidak melakukan penyortiran global atau konsolidasi duplikat. Dengan `--sort`, proyeksi duplikat dikonsolidasikan; ini dapat menghasilkan lebih sedikit baris dibandingkan ekspor input yang sama yang tidak disortir. Beberapa stempel waktu yang sama dengan arti atau sumber bukti berbeda dipertahankan di kedua mode. Identitas duplikat mencakup komputer, sumber, artefak, catatan, arti waktu, dan status pemulihan. Baris terkonsolidasi mencatat `DuplicateCount` dan jalur alternatif/bidang sumber yang dibatasi. Sumber yang berbeda tidak pernah digabungkan hanya karena jalur dan stempel waktunya mirip. Ketika sumber mentah tidak dapat diselesaikan, identitas rekaman masukan dipertahankan untuk menghindari runtuhnya tangkapan yang berbeda.

### Presisi, waktu lokal dan asumsi { #precision-local-times-and-assumptions }

Bidang UTC diketahui dan offset eksplisit dinormalisasi ke UTC. Nilai dan presisi asli, termasuk FILETIME 100 ns, disimpan; pembulatan terdahulu tidak bisa dipulihkan.

Sebagian bidang shell/FAT bermakna waktu lokal meski memakai `Z`. Nilai dan asal disimpan, tetapi zona endpoint yang hilang atau konversi sebelumnya tidak dapat direkonstruksi. Periksa artefak asli dan konfigurasi sebelum korelasi waktu presisi.

Waktu tanpa zona yang ditentukan artefak dilaporkan, tidak diam-diam dianggap UTC. Jika bukti mendukung, berikan offset tetap bertanda:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Tanggal saja juga memerlukan opsi itu; tengah malam adalah asumsi **presisi harian**, bukan peristiwa teramati. Nama zona dan DST tidak diterima. Nilai hilang/nol/rusak/di luar rentang tidak membuat peristiwa epoch palsu; tanggal 1970 yang sah dipertahankan.

Dua kategori dikecualikan secara default:

- `--include-collection-times`: metadata berkas LNK/Jump List di sistem koleksi/analisis, ditandai `collection_filesystem` dan dibedakan dari aktivitas endpoint.
- `--include-interpreted-times`: nilai registry umum QWORD ditafsir sebagai FILETIME dan berlabel `interpreted_value` karena banyak sebenarnya penghitung.

## Laporan pendamping { #companion-reports }

Untuk `SuperTimeline.csv`, perintahnya juga menulis:

- `SuperTimeline.csv-Untimed.jsonl`: lokasi rekaman asli tanpa waktu peristiwa yang dapat dipetakan.
- `SuperTimeline.csv-Diagnostics.jsonl`: waktu rusak/ambigu, metadata bersama sengaja dilewati dan batasan yang memerlukan rekaman asli.
- `SuperTimeline.csv-Summary.json`: masukan, hitungan artefak/peristiwa, penyaringan/kandidat/konsolidasi, keluaran, pekerja, anggaran/chunk/pass pengurutan dan durasi.

Nama pendamping menyertakan nama file keluaran lengkap. Misalnya, `SuperTimeline.jsonl` memiliki `SuperTimeline.jsonl-Summary.json` sendiri dan laporan tanpa batas waktu/diagnostik terkait. Ekspor CSV dan JSONL dengan batang yang sama dapat hidup berdampingan; `-C` diperlukan hanya untuk menggantikan kumpulan hasil yang ada.

Tanggal valid menunggu kenaikan asumsi zona waktu yang dibenarkan `UnresolvedTimeZones`, bukan `InvalidTimestamps`. Keduanya tetap dapat dilacak dalam diagnosis; tidak ada stempel waktu yang ditemukan.

Catatan dengan waktu valid di luar rentang yang diminta dihitung sebagai difilter, bukan tanpa batas waktu. Penghitungan acara kandidat mendahului konsolidasi duplikat. `Sorted` dan `DuplicateConsolidation` mencatat apakah `--sort` diaktifkan. Tanpanya, `DuplicateEvents`, `InitialSortChunks` dan `MergePasses` adalah nol, dan `SortMemoryBudgetMiB` adalah `null` karena tidak ada buffer pengurutan yang dialokasikan. `Complete: true` berarti semua input parsing yang dipilih berhasil diproses; ini tidak berarti setiap artefak dikumpulkan, setiap bidang diberi stempel waktu, atau setiap serangan dapat dideteksi.

CSV bisa meratakan profil sistem, mount point pengguna USB dan shell item; JSON/JSONL asli lebih kaya. Data yang hilang dari CSV tidak bisa dipulihkan. Artefak tak didukung atau masukan rusak menghasilkan kesalahan, bukan sukses parsial.

## Performa dan keamanan keluaran { #performance-and-output-safety }

Input dialirkan dan diproses secara paralel. Secara default, setiap pekerja menulis fragmen peristiwa yang diformat; fragmen digabungkan dalam urutan file masukan tanpa mengurutkan atau membaca ulang struktur acaranya. CSV memiliki satu header meskipun banyak file diproses. Urutan keluaran bersifat deterministik di seluruh jumlah pekerja; itu tidak menyiratkan kronologi.

Dengan `--sort`, buffer tumpah ke direktori sementara pribadi, lalu jalur gabungan yang dibatasi menghasilkan keluaran kronologis. Penyortiran menggunakan bilangan bulat penuh detik dan nanodetik, termasuk tanggal sebelum 1970. Pemesanan waktu yang sama bersifat deterministik dalam jumlah pekerja dan anggaran pengurutan. Bilah kemajuan maju untuk setiap peristiwa yang diproses selama penggabungan perantara dan akhir, termasuk lintasan berulang. Itu muncul di terminal dan disembunyikan ketika output dialihkan.

`--memory-limit` hanya berlaku dengan `--sort`: ini adalah **anggaran buffer pengurutan** agregat, bukan batas proses RSS. Catatan masukan, overhead objek/string JSON, buffer CSV, dan catatan gabungan memerlukan memori tambahan. Jumlah pekerja dikurangi untuk anggaran kecil. `--max-record-size` membatasi catatan input yang diuraikan (default 16 MiB); array/laporan JSON yang cantik dihitung sebagai satu dokumen masukan tingkat atas. Dokumen yang lebih besar mungkin memerlukan batas yang lebih tinggi atau, lebih disukai, JSONL. CSV parser read-ahead juga berkontribusi terhadap batas ini.

Ruang disk sementara harus mengakomodasi fragmen peristiwa yang diformat (default) atau catatan pengurutan yang diperluas dan lintasan gabungan (`--sort`), selain hasil akhir. Kedua mode mempertahankan staging keluaran sementara dan perlindungan terhadap penerbitan masukan yang gagal. Sebuah catatan dapat menghasilkan banyak peristiwa, sehingga garis waktunya bisa jauh lebih besar daripada masukannya. Filter rentang waktu/artefak bila perlu dan pilih volume sementara dengan ruang kosong yang memadai.

Ctrl+C membatalkan secara kooperatif, menghapus direktori pengurutan/pementasan pribadi dan keluar dengan kode `130`. Pada Linux dan macOS, hal ini juga berlaku untuk SIGTERM dan SIGHUP. Pembatalan diperiksa saat membaca, normalisasi, menggabungkan dan menulis laporan, sebelum publikasi dimulai. Pengakhiran yang tidak dapat dicegah atau hilangnya daya masih dapat meninggalkan file-file sementara.

Keluaran lama memerlukan `-C`. Masukan, manifes dan bukti terlindungi tidak boleh ditimpa. Kesalahan input/pengurutan menjaga keluaran lama. Berkas disiapkan di samping tujuan dan diganti satu per satu, linimasa terakhir. Ini **bukan** transaksi empat berkas; kegagalan I/O terakhir bisa meninggalkan sebagian laporan baru. Gunakan nama dasar baru untuk mempertahankan ekspor lengkap sebelumnya.

## Rincian lanjutan { #further-details }

- [Impor Timesketch CSV/JSONL](https://timesketch.org/guides/user/import-from-json-csv/)
- [Perluasan waktu Plaso dan CSV lama](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Keluaran dan format Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Bukti dan interpretasi](../overview/evidence-model.md)
- [Format keluaran](../getting-started/output-formats.md)

Masukan Juntei memakai singkatan `Comp`, `Filename`, `Args`, `Attr`. MFT CSV juga menerima `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN`, `HasADS` sambil menjaga generasi sequence dan perbedaan DOS/ADS. Nama lama didukung; `SourceField` menyimpan ejaan asli. Hayabusa mempertahankan `Computer`. Boot sector dan deskriptor keamanan masuk laporan tanpa waktu karena bukan peristiwa bertanggal.

## Parser banyak tabel terbaru { #newer-multi-table-parsers }

JSON/JSONL asli membawa nama tabel. Dalam CSV mandiri, browser, WebCache dan Outlook memakai skema kolom, sehingga nama CSV atau akhiran sumber tidak mengubah interpretasi. UAL memakai bidang waktu dikenal; penghitung dan teks mirip tanggal tidak ditebak.

Properti Outlook hanya diproyeksikan jika jenisnya waktu atau banyak waktu. Tanggal dalam string tetap string. Pesan tertanam mempertahankan ID/induk; ekstraksi tidak mengganti waktu lampiran tersimpan. Baris tanpa waktu tetap terlacak dalam hasil asli dan laporan pendamping.

Skema CSV/JSONL tetap sama; artefak baru tidak menambah ratusan kolom CSV. Konteks khusus ada di `Details`.
