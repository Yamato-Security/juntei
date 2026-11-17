# Linimasa gabungan { #super-timeline }

`super-timeline` mengubah artefak yang sudah diparsing menjadi peristiwa berskema bersama. Keluaran **tidak diurutkan secara default**: urutannya mengikuti urutan berkas masukan yang ditemukan dan urutan proyeksi setiap parser, tanpa bergantung pada urutan selesainya pekerja. Tambahkan `--sort` untuk pengurutan kronologis dan konsolidasi duplikat. Setiap waktu eksekusi dalam Prefetch atau delapan waktu berkas atau lebih dalam MFT menjadi peristiwa tersendiri. Makna waktunya tetap jelas ketika membandingkan sumber.

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

Sumber mentah diambil dari catatan atau perintah manifes yang tidak ambigu. Dengan `-d`, referensi sumber mentah relatif terhadap direktori bukti, dan `Details.ParsedFile` relatif terhadap direktori yang diurai. Tabel SRUM CSV ditetapkan ke database hanya ketika host memiliki satu SRUDB.dat yang dapat diidentifikasi secara unik. Jika tidak, `SourceFile` menunjuk ke file yang diurai dan `Details.RawSourceAvailable` adalah `false`.

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
| MFT | Waktu modifikasi, akses, perubahan metadata dan pembuatan SI/FN terpisah. Atribut FN bernama tetap berbeda; dengan `--sort`, SI berulang digabung. Waktu dasar bersama dari baris ADS dikecualikan di kedua mode. Nama slack rekaman FILE yang dipulihkan memiliki peristiwa FN pemulihan tersendiri. |
| I30 | Metadata FN/indeks dengan konteks slack/pemulihan, bukan waktu penulisan indeks direktori. |
| Jurnal USN | Waktu jurnal dan alasan pembaruan; satu rekaman dapat berisi beberapa perubahan tanpa awal yang tepat. |
| LogFile | Snapshot waktu redo/undo dan nama berkas tetap terpisah. LSN dan offset berkas adalah ID, bukan tanggal. |
| Prefetch | Eksekusi tercatat, yang dengan `--sort` digabung dengan ekspor linimasa Prefetch yang cocok; waktu pembuatan volume diberi label terpisah. |
| Database SQLite browser | Kunjungan, kunjungan terakhir yang dipulihkan, awal/akhir unduhan, waktu cookie dan penggunaan/penghapusan formulir. Akhir unduhan tidak membuktikan sukses/eksekusi; kedaluwarsa cookie adalah konfigurasi. Status pemulihan, keyakinan dan asal disimpan. |
| Internet Explorer / Edge lama WebCache | Akses, pembuatan/modifikasi, kedaluwarsa, sinkronisasi, unduhan, cookie dan pemeliharaan kontainer. Administrasi cache tidak otomatis membuktikan kunjungan pengguna. |
| Outlook PST, OST, MSG dan OFT | Pengiriman/penerimaan, pembuatan/modifikasi folder/item/lampiran, jadwal kalender/tugas/pengingat dan properti waktu bertipe eksplisit. Jadwal tidak membuktikan rapat, pengingat atau eksekusi; penerimaan tidak membuktikan pembacaan. |
| Pencatatan Akses Pengguna (UAL) | Metadata akses pertama/terakhir, penyisipan/pembuatan basis data, waktu boot sistem operasi yang pertama/terakhir dilihat dan disimpan. Peran, akun, alamat sumber, dan penghitung harian tetap sesuai konteks. Tanggal/penghitungan harian tidak ditemukan saat masuk tengah malam; identitas tanpa batas waktu dan baris DNS tetap berada dalam hasil parsing asli. |
| Amcache | Waktu tulis kunci inventaris, volume induk, pemasangan dan metadata tautan PE. Kehadiran inventaris atau LinkDate tidak membuktikan eksekusi. |
| ShimCache | Modifikasi berkas, pembaruan cache dan penulisan kunci bukan otomatis waktu eksekusi. |
| SRUM | Waktu/bucket penyedia, awal koneksi, interval eksplisit dan ID aplikasi. Byte, durasi dan penghitung bukan tanggal. |
| LNK | Waktu target MAC, pembuatan ID pelacakan dan shell item adalah metadata target, bukan pasti pembukaan shortcut. |
| Jump Lists | Penggunaan terakhir, ID pelacakan dan waktu target LNK tertanam. Waktu kontainer pada sistem koleksi diabaikan secara default. |
| Cache bitmap RDP | Tidak ada peristiwa: tile yang di-cache tidak memiliki waktu. Record-nya tetap ada di input yang diurai. |
| Recycle Bin | Waktu setiap item dipindahkan ke Recycle Bin, beserta pengguna Recycle Bin, path asli, dan apakah konten yang dihapus ikut dikumpulkan. Bukan waktu Recycle Bin dikosongkan, dan bukan waktu berkas milik item itu sendiri. |
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

Ekspor default mempertahankan semua proyeksi waktu, termasuk metadata berulang, tanpa pengurutan global atau konsolidasi duplikat. Dengan `--sort`, proyeksi duplikat digabung sehingga jumlah barisnya bisa lebih sedikit daripada ekspor tak terurut dari masukan yang sama. Waktu sama dengan makna/sumber berbeda tetap dipertahankan di kedua mode. Identitas duplikat mencakup komputer, sumber, artefak, rekaman, makna waktu dan pemulihan. Hasil gabungan menyimpan `DuplicateCount` dan referensi alternatif terbatas. Sumber berbeda tidak digabung hanya karena jalur/waktu mirip. Tanpa sumber mentah, identitas masukan mencegah penggabungan koleksi berbeda.

### Presisi, waktu lokal dan asumsi { #precision-local-times-and-assumptions }

Bidang UTC diketahui dan offset eksplisit dinormalisasi ke UTC. Nilai dan presisi asli, termasuk FILETIME 100 ns, disimpan; pembulatan terdahulu tidak bisa dipulihkan.

Sebagian bidang shell/FAT bermakna waktu lokal meski memakai `Z`. Nilai dan asal disimpan, tetapi zona endpoint yang hilang atau konversi sebelumnya tidak dapat direkonstruksi. Periksa artefak asli dan konfigurasi sebelum korelasi waktu presisi.

Stempel waktu yang naif tanpa zona waktu yang ditentukan artefak dihitung sebagai belum terselesaikan, bukan diperlakukan secara diam-diam sebagai UTC. Jika bukti membenarkan hal tersebut, berikan offset tetap yang ditandatangani:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Tanggal saja juga memerlukan opsi itu; tengah malam adalah asumsi **presisi harian**, bukan peristiwa teramati. Nama zona dan DST tidak diterima. Nilai hilang/nol/rusak/di luar rentang tidak membuat peristiwa epoch palsu; tanggal 1970 yang sah dipertahankan.

Dua kategori dikecualikan secara default:

- `--include-collection-times`: metadata berkas LNK/Jump List di sistem koleksi/analisis, ditandai `collection_filesystem` dan dibedakan dari aktivitas endpoint.
- `--include-interpreted-times`: nilai registry umum QWORD ditafsir sebagai FILETIME dan berlabel `interpreted_value` karena banyak sebenarnya penghitung.

## Keluaran dan lewati catatan { #output-and-skipped-records }

Perintah ini menulis **hanya garis waktu** yang dipilih dengan `-o`, dalam format yang dipilih dengan `-t` (`csv` secara default, atau `jsonl`). Itu tidak membuat file pendamping tanpa batas waktu, diagnostik atau ringkasan, atau gulungan diagnostik per rekaman. File pendamping yang ada dari proses sebelumnya tidak tersentuh.

Ringkasan terminal melaporkan jumlah peristiwa garis waktu dan catatan tanpa batas waktu. Stempel waktu tidak valid bukan nol dan jumlah zona waktu yang belum terselesaikan ditampilkan secara terpisah. Dengan `--sort`, ia juga melaporkan jumlah duplikat terkonsolidasi. Simpan hasil penguraian asli untuk rekaman yang tidak menghasilkan peristiwa garis waktu dan untuk konteks artefak lengkap.

Tanggal valid yang menunggu asumsi zona waktu yang dibenarkan dihitung sebagai zona waktu yang belum terselesaikan, bukan stempel waktu yang tidak valid. Tidak ada stempel waktu yang ditemukan, dan catatan tanpa waktu yang dapat dipetakan tidak dimasukkan ke dalam garis waktu sebagai peristiwa zaman sintetis.

Catatan dengan waktu valid di luar rentang yang diminta dihitung sebagai difilter, bukan tanpa batas waktu. Ekspor yang berhasil berarti input parsing yang dipilih berhasil diproses; ini tidak berarti setiap artefak dikumpulkan, setiap bidang memiliki stempel waktu, atau setiap serangan dapat dideteksi.

CSV bisa meratakan profil sistem, mount point pengguna USB dan shell item; JSON/JSONL asli lebih kaya. Data yang hilang dari CSV tidak bisa dipulihkan. Artefak tak didukung atau masukan rusak menghasilkan kesalahan, bukan sukses parsial.

## Performa dan keamanan keluaran { #performance-and-output-safety }

Masukan dialirkan dan diproses paralel. Secara default, setiap pekerja menulis fragmen peristiwa berformat; fragmen disambung sesuai urutan berkas masukan tanpa pengurutan dan tanpa membaca ulang struktur peristiwanya. CSV tetap memiliki satu header meskipun banyak berkas diproses. Urutan keluaran deterministik lintas jumlah pekerja, tetapi tidak menandakan urutan kronologis.

`Input files` menunjukkan ukuran gabungan dari input parsing yang dipilih dan garis waktu Hayabusa. `Results saved` menunjukkan ukuran file timeline yang disimpan. Keduanya menggunakan unit `KiB`/`MiB`/`GiB` yang dapat dibaca manusia seperti `auto-parse`.

Dengan `--sort`, buffer pengurutan meluap ke direktori sementara privat, lalu pass penggabungan terbatas menghasilkan keluaran kronologis. Pengurutan memakai detik dan nanodetik bilangan bulat utuh, termasuk tanggal sebelum 1970. Urutan peristiwa berwaktu sama deterministik lintas jumlah pekerja dan anggaran pengurutan. Bilah progres bertambah untuk setiap peristiwa yang diproses selama penggabungan perantara dan akhir, termasuk pass berulang. Bilah ini tampil di terminal dan disembunyikan jika keluaran dialihkan.

`--memory-limit` hanya berlaku dengan `--sort`: nilainya adalah **anggaran total buffer pengurutan**, bukan batas RSS proses. Rekaman, objek JSON, buffer CSV dan penggabungan memerlukan memori tambahan; anggaran kecil mengurangi pekerja. `--max-record-size` membatasi rekaman, default 16 MiB. Array JSON berformat dianggap satu dokumen; naikkan batas atau gunakan JSONL. Read-ahead CSV juga dihitung.

Disk sementara harus menampung fragmen peristiwa berformat (default) atau rekaman pengurutan yang diperluas beserta pass penggabungan (`--sort`), di samping hasil akhir. Kedua mode tetap memakai staging keluaran sementara dan perlindungan agar hasil dari masukan yang gagal tidak dipublikasikan. Satu rekaman dapat menghasilkan banyak peristiwa. Filter waktu/artefak dan pilih volume cukup longgar.

Ctrl+C membatalkan secara kooperatif, menghapus direktori pengurutan/pementasan pribadi dan keluar dengan kode `130`. Pada Linux dan macOS, hal ini juga berlaku untuk SIGTERM dan SIGHUP. Pembatalan diperiksa saat membaca, menormalkan, menggabungkan dan menulis timeline, sebelum publikasi dimulai. Pengakhiran yang tidak dapat dicegah atau hilangnya daya masih dapat meninggalkan file-file sementara.

Output yang ada memerlukan `-C`. Garis waktu tidak boleh menimpa masukan, manifes, referensi bukti lokal, atau direktori bukti yang dilindungi. Kesalahan input atau kegagalan pengurutan membuat timeline yang ada tidak tersentuh. Garis waktu yang telah selesai ditempatkan di samping tujuannya dan menggantikannya secara atom. Gunakan nama file keluaran baru untuk setiap proses ketika mempertahankan ekspor lengkap sebelumnya penting.

## Rincian lanjutan { #further-details }

- [Impor Timesketch CSV/JSONL](https://timesketch.org/guides/user/import-from-json-csv/)
- [Perluasan waktu Plaso dan CSV lama](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Keluaran dan format Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Bukti dan interpretasi](../overview/evidence-model.md)
- [Format keluaran](../getting-started/output-formats.md)

Input Juntei saat ini menggunakan label yang disingkat seperti `Comp`, `Filename`, `Args`, dan `Attr`. MFT CSV juga menerima `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` dan Header `HasADS`, menjaga generasi nomor urut dan perbedaan DOS/ADS tetap utuh. Label lama yang diekspor tetap diterima, dan `SourceField` mempertahankan ejaan pada masukan sebenarnya. Input Hayabusa mempertahankan bidang `Computer`-nya sendiri. Metadata sektor boot dan deskriptor keamanan tanpa stempel waktu peristiwa tetap berada dalam hasil penguraian asli; itu tidak membuat acara garis waktu.

## Parser banyak tabel terbaru { #newer-multi-table-parsers }

JSON/JSONL asli membawa nama tabel. Dalam CSV mandiri, browser, WebCache dan Outlook memakai skema kolom, sehingga nama CSV atau akhiran sumber tidak mengubah interpretasi. UAL memakai bidang waktu dikenal; penghitung dan teks mirip tanggal tidak ditebak.

Properti Outlook hanya diproyeksikan jika jenisnya waktu atau banyak waktu. Tanggal dalam string tetap string. Pesan tertanam mempertahankan ID/induk; ekstraksi tidak mengganti waktu lampiran tersimpan. Baris tanpa waktu tetap terlacak dalam hasil asli dan laporan pendamping.

Skema CSV/JSONL tetap sama; artefak baru tidak menambah ratusan kolom CSV. Konteks khusus ada di `Details`.
