# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## Apa itu ShimCache { #what-shimcache-is }

Subsistem Application Compatibility Windows menyimpan path executable dan metadata dalam cache saat mengevaluasi
perilaku kompatibilitas/shim. Cache ini tetap bertahan meskipun berkas diganti namanya, dihapus, atau terputus bersama
volume lepas pasang. Tata letak dan field yang tersedia berbeda-beda dari Windows XP hingga Windows 11.

ShimCache bernilai untuk menemukan path historis dan, bila formatnya memiliki flag eksekusi
yang andal, sebagai dukungan tambahan atas eksekusi. Waktu yang di-cache biasanya adalah **waktu modifikasi terakhir berkas**,
bukan waktu Windows melihat atau menjalankannya.

<span class="evidence-strength">Kesimpulan terbaik:</span> Windows mengamati path tersebut untuk pemrosesan
kompatibilitas. Sebuah entri saja merupakan bukti keberadaan/pengamatan; gunakan `Executed` hanya bila didukung.

## Lokasi dan pengumpulan { #location-and-collection }

Byte-nya berada di hive registry `SYSTEM`, umumnya di bawah:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Kumpulkan `SYSTEM`, `SYSTEM.LOG1`, dan `SYSTEM.LOG2`. Juntei membaca semua control set dan mengidentifikasi
`Select\Current` bila memungkinkan.

!!! warning "Entri live terbaru mungkin tidak ada di dalam hive"
    Windows dapat menahan pembaruan ShimCache di memori dan menuliskannya (flush) saat shutdown/reboot. Hive yang disalin
    dari sistem yang sedang berjalan mungkin tidak berisi pengamatan terbaru. Waktu last-write registry
    cache membantu menjelaskan kapan cache disimpan, tetapi bukan waktu eksekusi per entri.

## Peringatan bagi analis { #analyst-cautions }

- Entri cache dengan sendirinya **bukan bukti eksekusi**.
- `Executed` dapat bernilai `Yes`, `No`, atau `NA`; semantik yang tersedia bergantung pada format/versi.
- `LastModifiedTimeUTC` adalah milik berkas yang dirujuk, bukan waktu pengamatan atau eksekusi.
- Urutan berguna tetapi bukan timeline presisi yang universal; duplikat muncul di berbagai control set/slack.
- Entri `FromSlack` dipulihkan dari luar cache aktif dan memerlukan kehati-hatian tambahan.
- `SYSTEM` yang dirty tanpa log yang dapat digunakan mungkin tidak memuat catatan terkini. Baca ringkasan pemulihan.
- Temuan menunjukkan pola penamaan/path yang mencurigakan, bukan putusan malware.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `ControlSet` | Nomor control set yang menyimpan cache. Bandingkan `IsCurrent` sebelum memilih salah satunya. |
| `CacheEntryPosition` | Posisi/urutan di dalam cache yang didekode. Berguna untuk urutan relatif, bukan waktu. |
| `Path` | Path executable/paket yang di-cache. Pivot hunting utama. |
| `LastModifiedTimeUTC` | Waktu modifikasi berkas yang disimpan di dalam entri—bukan waktu eksekusi/pengamatan. |
| `Executed` | `Yes`, `No`, atau `NA` sesuai flag format yang didukung. Gunakan hanya dengan konteks versi. |
| `Duplicate` | Entri yang sama sudah terlihat di control set atau slack sebelumnya; membantu mencegah penghitungan ganda. |
| `SourceFile` | Hive/nilai mentah masukan. Penting saat bekerja dengan banyak host. |
| `FromSlack` | Entri yang dipulihkan dari slack nilai cache, bukan dari daftar aktif. Tingkat keyakinan lebih rendah dan mungkin lebih lama. |
| `CacheFormat` | Tata letak Windows/cache yang didekode. Menentukan ketersediaan dan semantik field. |
| `CacheLastWrittenUTC` | Waktu penyimpanan kunci registry/cache. Ini adalah konteks bersama, bukan waktu eksekusi setiap entri. |
| `InsertFlags`, `ShimFlags` | Flag kompatibilitas mentah; `Executed` adalah hasil interpretasi Juntei bila didukung. |
| `DataSize` | Ukuran data entri ini, berguna untuk validasi struktural. |
| `Machine` | Arsitektur yang dicatat oleh format yang lebih baru (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Metadata cache khusus Windows XP bila ada. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Identitas aplikasi terpaket pada entri yang didukung. Publisher ID adalah konteks, bukan bukti tanda tangan. |
| `Findings` | Kode anomali Juntei seperti biner sistem yang dipindahkan atau path staging yang mencurigakan. Hanya petunjuk. |
| `RegistryPath`, `RegistryValueName` | Sumber kunci/nilai hive yang persis. Pertahankan untuk validasi. |
| `CacheValueSize` | Ukuran seluruh nilai registry, bukan `DataSize` entri individual. |
| `IsCurrent` | Apakah control set entri ini sama dengan `SYSTEM\Select\Current`; kosong jika tidak dapat diketahui dengan pasti. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Nomor control set `SYSTEM\Select` mentah untuk konteks dan analisis pemulihan. |

JSON/JSONL juga mempertahankan data entri, tag, sub-catatan aplikasi terpaket, temuan terstruktur,
masalah cache, dan detail pemulihan.

## Strategi investigasi { #investigation-strategy }

1. Utamakan control set saat ini tetapi simpan set lain dan slack sebagai bukti historis.
2. Telusuri path dan temuan; jangan gunakan `LastModifiedTimeUTC` sebagai waktu eksekusi.
3. Bila `Executed == Yes`, korroborasi dengan Prefetch, SRUM, log peristiwa, atau jejak runtime lainnya.
4. Cocokkan path dengan `$MFT`/USN untuk menentukan riwayat pembuatan, penghapusan, dan metadata.
5. Laporkan apakah log transaksi diputar ulang dan apakah cache live mungkin belum di-flush.

## Detail lebih lanjut { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Spesifikasi format registry Windows oleh Maxim Suhanov](https://github.com/msuhanov/regf)
