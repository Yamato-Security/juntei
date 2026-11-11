# RecentDocs { #recentdocs }

## Apa itu RecentDocs { #what-recentdocs-is }

Windows Explorer menyimpan daftar berkas dan folder yang paling baru digunakan (MRU) di `NTUSER.DAT` milik setiap pengguna.
Daftar root mencakup item terbaru dari berbagai tipe; subkunci mengelompokkan ekstensi seperti `.docx`, ditambah
daftar `Folder`. `MRUListEx` menentukan urutan nilai-nilai bernomor.

RecentDocs berguna untuk mengidentifikasi dokumen yang berinteraksi dengan pengguna, termasuk berkas yang
kemudian dihapus atau dipindahkan. Nilai juga dapat berisi nama pintasan yang digunakan Windows di folder Recent
milik pengguna, yang dapat dikorelasikan dengan bukti LNK.

<span class="evidence-strength">Kesimpulan terbaik:</span> Explorer mencatat item tersebut dalam daftar
RecentDocs milik pengguna itu. Waktu kunci hanya menandai item terbaru dalam daftar spesifik tersebut.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Kumpulkan `NTUSER.DAT` beserta log transaksinya untuk setiap profil yang relevan, ditambah direktori LNK
Recent milik pengguna.

## Peringatan bagi analis { #analyst-cautions }

- `MRUListEx` memberikan urutan, bukan timestamp untuk setiap item.
- Waktu last-write kunci sesuai dengan pembaruan terbaru; hanya posisi MRU 0 yang mendapatkan
  `LastOpenedUTC` yang dapat dipertanggungjawabkan, yang diturunkan dari waktu tersebut.
- Sebuah item dapat muncul di daftar root dan daftar ekstensi. Jangan menghitungnya ganda sebagai dua kali pembukaan.
- Perilaku aplikasi dan Explorer dapat bervariasi; sebagian metode akses mungkin tidak memperbarui RecentDocs.
- Alat pembersih/privasi dan batas ukuran daftar menghapus bukti yang lebih lama.
- Nama berkas dapat dikendalikan penyerang dan bukan bukti tipe konten.
- Inferensi pengguna berasal dari path profil atau data shell-folder; tinjau nama pengguna yang kosong/ambigu.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host` | Host yang diturunkan dari konteks pengumpulan/sistem bila tersedia. |
| `User` | Pemilik profil yang disimpulkan dari path atau path shell-folder; pertahankan ketidakpastian bila kosong. |
| `SourcePath` | `NTUSER.DAT` masukan. Penting saat menggabungkan beberapa pengguna. |
| `Extension` | Subdaftar (`.docx`, `Folder`, dll.); null untuk daftar RecentDocs root. |
| `MRU-Position` | Nol adalah yang terbaru dalam daftar ini. Ini adalah urutan relatif, bukan waktu yang berlalu. |
| `ValueIndex` | Nilai registry bernomor yang dirujuk oleh `MRUListEx`. Hanya pivot struktural. |
| `Filename` | Nama item yang didekode. Korelasikan path lengkap melalui LNK, Jump List, MFT, atau data aplikasi. |
| `LinkName` | Nama pintasan yang dienkode di dalam shell item, sering ditemukan di direktori Recent milik pengguna. |
| `KeyLastWrittenUTC` | Waktu kunci daftar terakhir berubah. Berlaku untuk daftar, bukan untuk setiap baris. |
| `LastOpenedUTC` | Hanya diisi untuk posisi MRU 0, di mana waktu kunci mendukung interpretasi tersebut. |
| `ValueDataRaw` | Byte mentah bila dekode gagal atau bukti perlu divalidasi. |
| `TransactionLogStatus`, `LogsApplied` | Apakah log registry diputar ulang dan log mana saja. |
| `Problems` | Nilai MRU yang hilang, nilai yang tidak tercantum, tipe yang salah, atau kerusakan dekode. |

## Strategi investigasi { #investigation-strategy }

1. Pisahkan daftar root dan daftar ekstensi setiap pengguna; hapus duplikasi item yang sama dengan hati-hati.
2. Gunakan posisi MRU untuk pengurutan dan hanya `LastOpenedUTC` untuk klaim yang bertimestamp.
3. Cocokkan `LinkName`/`Filename` dengan LNK Recent, Jump List, `$MFT`, dan data USN.
4. Tinjau nama umpan (decoy), ekstensi ganda, target jaringan/lepas pasang, dan dokumen di sekitar waktu akses awal.
5. Korelasikan dengan Prefetch/SRUM aplikasi dan log peristiwa bila eksekusi menjadi pertimbangan.

## Detail lebih lanjut { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [Poster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/)
