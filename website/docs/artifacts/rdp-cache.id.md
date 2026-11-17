# Cache Bitmap RDP { #rdp-bitmap-cache }

## Apa yang dicatat cache bitmap RDP { #what-the-rdp-bitmap-cache-records }

Saat klien Remote Desktop (`mstsc.exe`) terhubung dengan persistent bitmap caching aktif, yang merupakan
pengaturan default, klien menyimpan potongan-potongan layar jarak jauh ke disk agar sesi berikutnya tidak
perlu menerimanya lagi. Setiap potongan adalah sebuah tile berukuran paling besar 64 × 64 piksel. Bersama-sama,
tile-tile itu merupakan fragmen kecil dari apa yang dilihat pengguna di komputer jarak jauh: jendela, teks,
nama berkas, perintah, dan desktop.

| Klien | Berkas cache | Tile |
|---|---|---|
| Windows 7 dan yang lebih baru | `Cache0000.bin`, `Cache0001.bin`, …, masing-masing hingga 100 MB | 32-bit, berurutan satu demi satu |
| Windows XP, Server 2003, dan yang lebih lama | `bcache2.bmc` (8 bit per piksel), `bcache22.bmc` (16), `bcache23.bmc` (24), dan `bcache24.bmc` (32), masing-masing hingga 10 MB | Satu per slot 64 × 64; sebagian dikompresi |

Windows 10 dan yang lebih baru juga meninggalkan `bcache24.bmc` kosong di samping berkas `.bin`.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna yang profilnya menyimpan cache tersebut
terhubung dengan klien Remote Desktop ke komputer yang menampilkan konten yang di-cache. Cache tidak
menyebutkan komputer mana itu, atau kapan.

## Lokasi dan pengumpulan { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

Nilai `BitmapPersistCacheLocation` di `HKCU\Software\Microsoft\Terminal Server Client` dapat memindahkan
cache ke lokasi lain. Kumpulkan seluruh folder `Cache` dari setiap profil, juga dari volume shadow copy:
salinan lama sebuah berkas cache dapat menyimpan tile yang sudah tidak ada di berkas saat ini.

## Peringatan bagi analis { #analyst-cautions }

- Tile tidak memiliki waktu, dan cache tidak menyebutkan nama komputer asal tile tersebut. Kaitkan tile
  dengan koneksi RDP dari bukti lain: key registry `Terminal Server Client` milik klien
  (`parse-registry remote-access`), `Default.rdp` dan berkas `.rdp` lainnya, Jump Lists milik
  `mstsc.exe`, waktu berkas cache di `$MFT`, dan peristiwa logon di komputer jarak jauh.
- Satu berkas cache mencampur tile dari banyak sesi dan dari beberapa komputer jarak jauh. Klien mengganti
  tile lama saat berkas penuh, sehingga sebuah tile bisa berumur berhari-hari atau berbulan-bulan.
- Tile adalah potongan layar, bukan layar utuh. Tile yang bersebelahan di layar sering kali juga
  bersebelahan di dalam berkas, tetapi tidak selalu. Susun kembali layar dengan
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher), atau secara manual dengan melihat kolase.
- Sebagian besar tile berukuran 64 × 64 piksel. Tile di tepi kanan dan bawah layar lebih sempit atau
  lebih pendek, tergantung resolusinya: misalnya lebar 58 atau 62 piksel, atau tinggi 28 atau 35 piksel.
  Juntei membaca setiap tile sesuai ukurannya sendiri. Alat yang membaca setiap tile selebar 64 piksel
  menampilkan tile yang sempit dalam keadaan miring.
- Sebagian besar tile satu warna (`SingleColor`) adalah latar belakang. Kecualikan tile tersebut untuk
  menemukan konten.
- Di berkas `.bmc`, tile 8-bit memakai warna dari palet sesi, yang tidak disimpan oleh cache. Juntei
  menggunakan palet default Windows, sehingga warnanya bisa salah. Tile yang dikompresi menggunakan
  interleaved RLE milik protokol Remote Desktop (MS-RDPBCGR 3.1.9), yang didekompresi Juntei
  sebagaimana didefinisikan oleh spesifikasi.
- Tile `.bmc` yang lebih kecil dari slot 64 × 64-nya menyisakan bagian akhir tile sebelumnya di
  belakangnya. Juntei menyimpan baris-baris sisa itu sebagai gambar terpisah (`RemnantImage`). Baris
  tersebut milik tile yang lebih lama, bukan milik tile di slot itu.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

Dengan `-d`, Juntei membaca berkas yang namanya berupa `Cache` diikuti angka dan `.bin`, serta `bcache`
diikuti angka dan `.bmc`. Berkas `Cache????.bin` yang tidak diawali signature `RDP8bmp` adalah milik
program lain: berkas itu dihitung dalam ringkasan dan dilewati. Dengan `-f`, Juntei membaca berkas
tersebut apa pun namanya.

### Gambar { #images }

`--extract` membuat satu folder untuk setiap berkas cache, `<DIR>/<user>/<cache file>/`. Jika dua berkas
cache akan memakai folder yang sama, misalnya sebuah berkas cache dan salinannya dari volume shadow copy,
folder kedua diberi nomor (`Cache0001.bin_2`). Folder tersebut berisi:

- `<cache file>_<index>.bmp`: setiap tile, sesuai ukurannya sendiri. Namanya sama dengan yang digunakan
  bmc-tools, sehingga RdpCacheStitcher dapat membuka folder tersebut.
- `<cache file>_old_<index>.bmp`: hanya untuk `.bmc`, bagian akhir tile sebelumnya yang tersisa setelah
  tile yang lebih kecil.
- `<cache file>_collage.png`: setiap tile dalam urutan berkas, 64 per baris (`--collage-width`),
  masing-masing dalam sel 64 × 64 mulai dari pojok kiri atas sel. Tile dengan indeks *i* berada di baris
  *i* / 64 dan kolom *i* % 64, dihitung dari 0. Warna magenta mengisi bagian sel yang kosong: di samping
  tile yang sempit, di bawah tile yang pendek, dan di tempat tile tidak dapat dibaca.

`--collage-only` hanya menyimpan kolase. Gambar yang sudah ada tidak diganti tanpa `-C`, dan `Problems`
milik tile-nya mencatat hal itu. Berkas cache itu sendiri tidak pernah diganti, bahkan dengan `-C`.

## Field keluaran CSV { #csv-output-fields }

| Field | Arti |
|---|---|
| `SourceFile` | Berkas cache. |
| `User` | Pengguna yang folder profilnya berisi berkas cache tersebut, berdasarkan path-nya. |
| `TileIndex` | Nomor tile di dalam berkas, dihitung dari 0, seperti pada nama gambarnya. |
| `Width`, `Height` | Ukuran tile dalam piksel. |
| `SingleColor` | `true` bila setiap piksel tile memiliki warna yang sama. |
| `TileImage` | Dengan `--extract`: berkas BMP milik tile. |
| `Collage` | Dengan `--extract`: kolase berkas cache. |
| `RemnantImage` | `.bmc` dengan `--extract`: bagian akhir tile sebelumnya yang tersisa di slot. |
| `Key` | Key 64-bit milik tile dalam protokol Remote Desktop, dalam heksadesimal. |
| `Format`, `FormatVersion` | `bin` beserta versi setelah signature-nya (6 pada Windows 10 dan yang lebih baru), atau `bmc`. |
| `BitsPerPixel` | 32 untuk `.bin`; 8, 16, 24, atau 32 untuk `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: apakah tile dikompresi, dan flag pada header-nya. |
| `Offset` | Posisi awal header tile di dalam berkas. |
| `Problems` | Kerusakan: berkas yang berakhir di tengah tile, header tile yang tidak mungkin dimiliki tile mana pun, data yang tidak sesuai dengan ukuran tile, data terkompresi yang tidak dapat didekompresi, gambar yang tidak dapat disimpan. |

JSON dan JSONL memiliki field yang sama, dengan angka dan boolean yang bertipe, `null` untuk nilai yang
tidak dimiliki tile, dan `Problems` sebagai daftar. Kerusakan di luar tile mana pun, seperti data setelah
tile terakhir, dilaporkan dalam pesan eksekusi.

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis `LateralMovement/RdpCache.csv` (`RdpCache.json` atau `RdpCache.jsonl` dengan
`-t json` atau `-t jsonl`), beserta `LateralMovement/RdpCache-logs.txt`, dan menyimpan kolase setiap
berkas cache di folder `LateralMovement/RdpCache/`. Tambahkan `--extract-rdp-tiles` untuk juga menyimpan
setiap tile, yang bisa mencapai puluhan ribu berkas untuk satu pengguna. Dari image disk, berkas cache
disalin. Timeline gabungan `-j` menyertakan setiap baris dengan pengenal `Artifact` `rdp-cache`.
Tile tidak memiliki waktu, sehingga [`super-timeline`](../commands/super-timeline.md) tidak membuat
peristiwa dari tile tersebut.

## Strategi investigasi { #investigation-strategy }

1. Buka kolase dan cari teks: nama komputer dan nama pengguna, path berkas, perintah, jendela konsol,
   bilah alamat browser, dan kotak dialog.
2. Susun kembali layar dengan RdpCacheStitcher dari folder tile yang disimpan dengan `--extract`.
3. Temukan komputer yang dihubungi pengguna dari registry klien (`parse-registry remote-access`),
   berkas `.rdp`, dan Jump Lists milik `mstsc.exe`. Di komputer-komputer tersebut, cari logon RDP yang
   sesuai dan record Pencatatan Akses Pengguna (UAL).
4. Gunakan waktu `$MFT` milik berkas cache, dan milik salinannya di volume shadow copy, untuk
   mempersempit kapan tile ditulis.

## Detail lebih lanjut { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: cache bitmap RDP (dalam bahasa Prancis)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
