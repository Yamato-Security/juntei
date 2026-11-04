# Daftar MRU (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Apa itu daftar-daftar ini { #what-the-lists-are }

Windows dan program-programnya menyimpan daftar singkat tentang apa yang terakhir diketik atau dibuka pengguna, di dalam
`NTUSER.DAT` milik pengguna:

- **RunMRU:** perintah yang diketik ke dialog Run (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** path yang diketik ke bilah alamat Explorer.
- **WordWheelQuery:** istilah yang dicari di kotak pencarian Explorer.
- **TypedURLs:** alamat yang diketik ke bilah alamat Internet Explorer, dan sejak Windows 8
  `TypedURLsTime`, kapan masing-masing diketik.
- **Paint dan WordPad:** berkas-berkas yang terakhir dibuka oleh masing-masing program (`Recent File List`).
- **ACMru (Windows XP):** istilah Search Assistant, per jenis pencarian.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna mengetik perintah, path,
istilah pencarian, atau alamat ini, atau program tersebut membuka berkas ini. Hanya entri terbaru dari setiap daftar
yang memiliki tanggal, yaitu waktu last-write kuncinya, kecuali alamat yang diketik, yang diberi tanggal oleh `TypedURLsTime`.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

Subkunci `ACMru` adalah jenis-jenis pencarian: `5603` nama berkas dan folder, `5604` kata atau frasa di dalam
berkas, `5647` komputer dan orang, `5001` Internet.

Kumpulkan `NTUSER.DAT` beserta log transaksinya untuk setiap profil.

## Peringatan bagi analis { #analyst-cautions }

- **Satu waktu per daftar.** Waktu last-write sebuah kunci berubah saat daftarnya berubah, sehingga waktu tersebut hanya menandai
  entri terbaru (`IsMostRecent`, `LastUsedUTC`). Entri lainnya berurutan, tanpa tanggal.
  `TypedURLsTime` memberi tanggal pada setiap alamat yang diketik (`TypedTimeUTC`).
- **Diketik, bukan dijalankan.** RunMRU mencatat apa yang diketik ke dialog Run, baik berhasil dijalankan
  maupun tidak. Windows menyimpan setiap perintah dengan akhiran `\1`; Juntei menghapusnya. Korelasikan
  dengan Prefetch, BAM, UserAssist, dan peristiwa pembuatan proses.
- **Daftarnya pendek.** Masing-masing hanya menyimpan entri terbarunya, dan pengaturan privasi serta alat
  pembersih menghapusnya. Gunakan `parse-registry read --recover-deleted` untuk nilai yang terhapus.
- **Istilah adalah apa yang diketik.** WordWheelQuery dan ACMru menyimpan teks yang diketik, bukan apa yang ditemukan.
- **Urutan berasal dari daftar.** `MRUList` milik RunMRU dan `MRUListEx` milik WordWheelQuery memberikan
  urutannya; untuk yang lain, nomor pada nama nilai yang menentukannya (`url1`, `File1`, dan `000` adalah yang terbaru).

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Ringkasan mencantumkan entri per daftar dan entri yang memiliki tanggal terakhir (alamat yang diketik, serta entri terbaru
dari setiap daftar), diurutkan dari yang terbaru.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, pemilik profil (dari path atau folder shell), SID dari `ProfileList` sistem, dan hive. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad`, atau `ACMru`. |
| `Subkey` | Subkunci `ACMru`: jenis pencarian (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | Posisi di dalam daftar (0 adalah yang terbaru); kosong bila daftar tidak menyebutkan nilai tersebut. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | Perintah (tanpa `\1`-nya), path, istilah pencarian, alamat, atau berkas. |
| `TypedTimeUTC` | Kapan alamat diketik, dari `TypedURLsTime`. |
| `LastUsedUTC` | Waktu last-write kunci, hanya untuk entri terbaru. |
| `KeyPath`, `KeyLastWrittenUTC` | Kunci dan waktu last-write-nya: waktu entri terbarunya, bukan waktu setiap baris. |
| `DataRaw` | Byte nilai (hex) bila tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai yang disebutkan daftar tetapi tidak ada di kunci, nilai yang tidak disebutkan daftar, nilai dengan nama yang salah, tipe yang salah, istilah yang terpotong, waktu yang bukan FILETIME. |

## Strategi investigasi { #investigation-strategy }

1. Baca RunMRU untuk menemukan perintah, skrip, network share, dan alat administrasi yang diketik oleh pengguna.
2. Baca TypedPaths dan WordWheelQuery untuk menemukan folder dan berkas yang dicari pengguna:
   kredensial, hasil ekspor, arsip, lokasi lepas pasang dan jaringan.
3. Tempatkan alamat yang diketik pada timeline dengan `TypedTimeUTC`.
4. Korelasikan dengan Prefetch, BAM, UserAssist, ShellBags, berkas LNK, jump list, dan riwayat browser.

## Detail lebih lanjut { #further-details }

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
