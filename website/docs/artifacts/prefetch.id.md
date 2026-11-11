# Prefetch { #prefetch }

## Apa itu Prefetch { #what-prefetch-is }

Berkas Prefetch Windows mencatat informasi yang digunakan untuk mengoptimalkan startup aplikasi. Berkas `.pf`
khusus program umumnya menyimpan nama dan hash executable, jumlah eksekusi, waktu eksekusi terbaru, informasi
volume, direktori, dan berkas yang dirujuk selama startup.

Untuk versi Windows yang didukung, record Prefetch aplikasi yang valid biasanya merupakan bukti kuat
bahwa executable tersebut berjalan. Prefetch adalah salah satu artefak eksekusi non-log peristiwa yang paling berguna.

<span class="evidence-strength">Kesimpulan terbaik:</span> executable yang disebutkan diluncurkan pada
waktu eksekusi terbaru yang tercatat, dengan tetap memperhatikan perilaku sistem operasi dan validitas parser.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei mendukung versi format 17 hingga 31 (Windows XP hingga Windows 11), termasuk berkas
Windows 8+ yang terkompresi. Kumpulkan seluruh direktori agar hash nama executable dan record di sekitarnya dapat
dibandingkan.

## Peringatan bagi analis { #analyst-cautions }

- Windows hanya menyimpan satu waktu eksekusi historis pada versi lama dan hingga delapan pada versi yang lebih baru.
  `RunCount` dapat melebihi jumlah timestamp yang tersedia.
- Prefetch dapat dinonaktifkan, dibersihkan, dikecualikan, atau tidak tersedia pada beberapa sistem/peran. Ketiadaannya bukan
  bukti bahwa sebuah program tidak berjalan.
- Daftar berkas yang dimuat mendeskripsikan dependensi dan akses saat startup; daftar ini tidak membuktikan bahwa setiap berkas yang tercantum
  berbahaya, sengaja dibuka, atau dibaca sepenuhnya.
- Nama executable di header Prefetch memiliki batas panjang. Juntei membedakan jalur termuat yang
  bernama sama dengan hash jalur SCCA. Jika beberapa kandidat tetap tidak dapat dibedakan, Juntei
  mempertahankan nama header dan menetapkan PathAmbiguous; daftar berkas yang dimuat tetap tersedia.
  Satu-satunya kandidat yang hash-nya tidak cocok tetap dipertahankan, dengan PathValidated bernilai
  false. Proses hosting dapat memasukkan data baris perintah ke dalam hash, sehingga jalur yang
  tidak tervalidasi belum tentu salah.
- Timestamp sistem berkas dari berkas sumber `.pf` terpisah dari waktu eksekusi yang tersimpan di dalamnya.
- `Findings` adalah observasi untuk hunting. Installer dan administrasi yang sah sering menggunakan path staging.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Field CSV standar { #standard-csv-fields }

| Field | Interpretasi |
|---|---|
| `SourceFile` | `.pf` input; pertahankan untuk membedakan host dan nama executable yang duplikat. |
| `ExecutableName` | Nama executable di header. Tidak peka huruf besar/kecil untuk sebagian besar hunting; pertahankan ejaan aslinya. |
| `ExecutablePath` | Path yang dimuat sebagaimana ditentukan Juntei bila memungkinkan. Konfirmasikan dengan `Files` dan artefak lain. |
| `PathValidated` | Jalur terpilih cocok dengan hash jalur SCCA. Ini memvalidasi atribusi jalur, bukan isi atau keterpercayaan berkas. |
| `PathAmbiguous` | Beberapa kandidat jalur belum dapat dibedakan; `ExecutablePath` mempertahankan nama header. Periksa `Files` dan artefak lain. |
| `PrefetchHash` | Hash yang digunakan Windows dalam nama berkas `.pf`; ini bukan hash kriptografis dari konten berkas. |
| `FormatVersion` | Versi struktur Prefetch; menentukan field yang tersedia dan jumlah timestamp. |
| `RunCount` | Penghitung kumulatif yang dicatat Windows. Berguna untuk frekuensi, bukan riwayat yang persis. |
| `LastRun` | Waktu eksekusi tersimpan yang paling baru. |
| `PreviousRun1` … `PreviousRun7` | Waktu eksekusi tersimpan sebelumnya, dari yang terbaru ke yang lebih lama. Kosong bila format/riwayat tidak memilikinya. |
| `VolumeCount` | Jumlah volume sumber yang tercatat. |
| `VolumeDevicePaths` | Path perangkat NT untuk volume yang digunakan selama startup. Korelasikan dengan nomor seri. |
| `VolumeSerialNumbers` | Pengenal volume; berguna untuk menautkan bukti drive lepas pasang atau drive yang telah diformat ulang. |
| `VolumeCreated` | Waktu pembuatan volume yang tercatat, bukan waktu pembuatan executable. |
| `DirectoryCount` | Jumlah string direktori yang tercatat. |
| `FileCount` | Jumlah referensi berkas yang dimuat yang tercatat. |
| `Findings` | Observasi Juntei seperti binary sistem yang dipindahkan atau path eksekusi yang mencurigakan. Tinjau secara manual. |
| `Files` | Berkas yang dimuat/dirujuk beserta referensi NTFS yang tersedia. Titik pivot DLL/path yang sangat baik, bukan bukti niat terhadap berkas. |
| `Dir` | String direktori yang terkait dengan startup. Berguna bila path berkas yang dimuat secara individual tidak lengkap. |

## Field CSV timeline (`-T`) { #timeline-csv-fields-t }

| Field | Interpretasi |
|---|---|
| `RunTime` | Satu waktu eksekusi Prefetch yang tersimpan. |
| `ExecutableName`, `ExecutablePath` | Konteks identitas/path program. |
| `PathValidated` | Jalur terpilih cocok dengan hash jalur SCCA. Ini memvalidasi atribusi jalur, bukan isi atau keterpercayaan berkas. |
| `PathAmbiguous` | Beberapa kandidat jalur belum dapat dibedakan; `ExecutablePath` mempertahankan nama header. Periksa `Files` dan artefak lain. |
| `PrefetchHash` | Memisahkan varian/path yang memiliki nama executable yang sama. |
| `RunCount` | Penghitung dari berkas Prefetch; diulang pada setiap baris timeline. |
| `SourceFile` | `.pf` sumber untuk validasi. |

JSON/JSONL mempertahankan volume, direktori, berkas yang dimuat, dan referensi berkas NTFS sebagai array
terstruktur alih-alih sel CSV yang digabung dengan koma.

## Strategi investigasi { #investigation-strategy }

1. Gunakan `-T` untuk menambahkan eksekusi yang tersimpan ke timeline.
2. Kelompokkan berdasarkan nama executable ditambah hash Prefetch; nama yang sama dapat memiliki path/hash yang berbeda.
3. Tinjau `ExecutablePath`, path staging, temuan binary sistem yang dipindahkan, dan DLL yang dimuat.
4. Cocokkan referensi MFT target dan path berkas dengan record `$MFT`/USN.
5. Korelasikan waktu eksekusi dengan log peristiwa, SRUM, telemetri jaringan, layanan, tugas, dan artefak pengguna.

## Detail lebih lanjut { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
- [PECmd oleh Eric Zimmerman](https://github.com/EricZimmerman/PECmd)
