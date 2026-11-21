# Riwayat PowerShell { #powershell-history }

## Apa yang dicatat riwayat { #what-the-history-records }

PSReadLine menyimpan baris perintah yang dimasukkan dalam host PowerShell interaktif. Berkas berisi teks sesuai urutan penyimpanan, tanpa stempel waktu per perintah.

## Lokasi dan pengumpulan { #locations-and-collection }

Lokasi bawaan di Windows untuk Windows PowerShell 5.1 dan PowerShell 7+ adalah:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Host lain memakai nama seperti `Visual Studio Code Host_history.txt` dalam direktori tersebut. PowerShell di Linux dan macOS biasanya memakai `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. `HistorySavePath` milik PSReadLine dapat mengalihkan berkas; jalur bawaan ini merupakan asumsi pencarian, bukan daftar lokasi yang lengkap.

Masukan direktori dan `auto-parse` memilih `<Host>_history.txt` secara rekursif yang berada langsung dalam folder `PSReadLine`, tanpa membedakan huruf besar/kecil ASCII. Keduanya tidak memilih sembarang berkas `*_history.txt` di tempat lain. Gunakan masukan `-f` secara eksplisit untuk berkas yang diganti namanya atau salinan di luar direktori `PSReadLine`. Jika memungkinkan, pertahankan struktur profil asli untuk atribusi pengguna.

## Peringatan bagi analis { #analyst-cautions }

- Tidak ada baris yang memiliki waktu. `FileLastWriteTime` adalah waktu modifikasi berkas. Secara bawaan, PSReadLine menyimpan riwayat setelah setiap perintah (`HistorySaveStyle` `SaveIncrementally`), sehingga waktu ini biasanya dekat dengan saat baris terakhir ditambahkan, tetapi tidak mengatakan apa pun tentang baris-baris sebelumnya.
- Baris yang tersimpan tidak membuktikan bahwa perintah dijalankan atau selesai dengan sukses. Cocokkan eksekusi dan waktunya dengan log peristiwa PowerShell, telemetri proses, dan artefak eksekusi lainnya.
- Riwayat dapat diedit, dihapus, atau dimatikan (`HistorySaveStyle` `SaveNothing`). PSReadLine juga tidak menyimpan baris yang tampaknya berisi rahasia, seperti baris dengan `password`, `token`, atau `apikey`, dan Windows PowerShell ISE tidak memakai PSReadLine. Skrip dan perintah yang dijalankan dengan `-Command` atau `-EncodedCommand` tidak diketik, sehingga tidak tersimpan.
- Perintah yang terdiri dari beberapa baris disimpan dengan backtick di akhir setiap barisnya kecuali baris terakhir. Juntei mempertahankan baris-baris tersebut sebagai rekaman terpisah yang dihubungkan oleh `EntryStartLine`. Baris yang diakhiri backtick yang diketik pengguna terlihat sama, jadi baca baris-baris satu entri secara bersamaan.

## Mengurai dengan Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV adalah format berkas bawaan. Dengan `-t csv`, `-t json`, atau `-t jsonl` tanpa `-o`, rekaman ditulis ke stdout. Tanpa kedua opsi tersebut, hanya ringkasan yang ditampilkan. JSON berisi objek terpisah dengan format yang mudah dibaca; JSONL berisi satu objek per baris. Tambahkan `-C` untuk menimpa keluaran yang sudah ada; alias berkas sumber dilindungi dari penggantian.

## Encoding dan penanganan baris { #encoding-and-line-handling }

- Penanda urutan byte UTF-8, UTF-16LE, atau UTF-16BE menentukan encoding dan dihapus dari awal berkas. Tanpa BOM, masukan selalu diperlakukan sebagai UTF-8. UTF-16 tanpa BOM tidak ditebak; karakter NUL dipertahankan dan dilaporkan sebagai kemungkinan kerusakan encoding.
- Urutan UTF-8 yang tidak valid dan surrogate UTF-16 tanpa pasangan menjadi U+FFFD, dengan kerusakan dilaporkan dalam `Problems`. Byte UTF-16 sisa di akhir dan kesalahan baca juga dilaporkan; baris yang dapat dibaca tetap disimpan. Periksa `Problems` dan log parser sebelum mengandalkan kelengkapannya.
- Berkas diproses secara streaming. Maksimal 1,048,576 byte masukan dipertahankan per baris fisik, tidak termasuk BOM dan pemisah LF (CR sebelum LF dihitung dalam batas). Baris yang lebih panjang dipotong pada batas karakter utuh; byte sisanya tetap dibaca dan `Problems` melaporkan jumlah byte yang dihilangkan. Batas berlaku untuk byte masukan yang dikodekan, bukan jumlah karakter.
- Akhir baris LF dan CRLF dihapus, begitu pula CR terakhir pada baris tanpa terminator. Spasi dan tab lain dipertahankan. Baris kosong tidak menghasilkan rekaman, tetapi tetap dihitung dalam nomor baris dan mengakhiri rantai kelanjutan; baris yang hanya berisi whitespace tetap menjadi rekaman. Kesalahan baca dapat menghasilkan baris diagnostik kosong untuk menandai titik berhentinya pembacaan.
- Baris tetap terpisah, dengan backtick dan indentasi dipertahankan. `EntryStartLine` menunjuk ke baris pertama dalam rantai berurutan yang baris-baris sebelumnya berakhir dengan backtick. Sintaks shell tidak ditafsirkan dan perintah tidak direkonstruksi. Pemotongan dapat memengaruhi atribusi kelanjutan; periksa `Problems` sebelum mengelompokkan baris tersebut.

## Field keluaran CSV { #csv-output-fields }

| Kolom | Interpretasi |
|---|---|
| `SourceFile` | Jalur relatif terhadap direktori masukan, memakai `/`; untuk `-f`, hanya nama berkas. Jalur sumber absolut tidak diekspor. |
| `User` | Nama profil dari jalur. Folder di atas `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (atau `.local/share/powershell/PSReadLine`) menunjukkan nama pengguna, juga ketika masukan adalah profil tersebut atau folder `PSReadLine` itu sendiri. Masukan direktori juga mengenali struktur `Users`/`Documents and Settings` di bawah akar masukan. Nilai yang tidak diketahui kosong dalam CSV dan `null` dalam JSON. |
| `ShellHost` | Awalan nama berkas sebelum `_history.txt`, seperti `ConsoleHost`; tidak diketahui untuk berkas yang diganti namanya tanpa akhiran tersebut. |
| `LineNumber` | Nomor baris fisik mulai dari 1, termasuk baris kosong yang dilewati. |
| `EntryStartLine` | Baris fisik awal rantai kelanjutan, atau nomor baris ini sendiri. |
| `Command` | Baris fisik yang didekode tanpa akhir baris, dengan batas yang didokumentasikan. |
| `FileLastWriteTime` | Waktu modifikasi berkas sumber, diulang pada setiap baris. **Metadata berkas, bukan waktu eksekusi perintah.** Penguraian image memakai metadata sumber asli, bukan waktu staging. |
| `FileSize` | Ukuran berkas sumber dalam byte. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE`, atau `UTF-16BE`. |
| `Problems` | Diagnostik kerusakan dan pemotongan baris; dipisahkan titik koma dalam CSV, berupa array dalam JSON/JSONL. Diagnostik tingkat berkas juga tercatat dalam log. |

JSON dan JSONL memiliki kolom yang sama, dengan angka yang bertipe, `null` untuk nilai yang tidak diketahui, dan `Problems` sebagai daftar. Setiap format mempertahankan teks perintah sebagaimana tersimpan.

`FileLastWriteTime` secara bawaan memakai UTC dalam format ISO 8601, misalnya `2026-10-20T08:46:38.5Z`. Hanya `auto-parse` yang menerima `-Z, --timezone` untuk mengubah offset yang ditampilkan. Perintah mandiri `parse-file powershell-history` tidak memiliki opsi zona waktu.

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis `ShellHistory/PowerShellHistory.csv` (`PowerShellHistory.json` atau `PowerShellHistory.jsonl` dengan `-t json` atau `-t jsonl`), beserta `ShellHistory/PowerShellHistory-logs.txt`. Dari image VHDX dan E01, berkas riwayat disalin, dan `FileLastWriteTime` adalah waktu yang tercatat di dalam image. Timeline gabungan `-j` menyertakan setiap baris dengan pengenal `Artifact` `powershell-history`. Baris-baris riwayat tidak memiliki waktu, sehingga [`super-timeline`](../commands/super-timeline.md) tidak membuat peristiwa darinya dan menghitungnya sebagai rekaman tanpa waktu.

## Strategi investigasi { #investigation-strategy }

1. Baca riwayat setiap profil, termasuk `Windows\System32\config\systemprofile` dan profil-profil di bawah `Windows\ServiceProfiles`, yang menyimpan perintah yang diketik di shell yang berjalan sebagai akun-akun tersebut.
2. Baca entri multibaris secara bersamaan, dan cari unduhan, perintah yang dienkode, perubahan pengaturan keamanan, perintah akun dan kredensial, serta koneksi jarak jauh.
3. Cocokkan dengan log peristiwa PowerShell (pencatatan script block, peristiwa 4104), peristiwa pembuatan proses, dan berkas Prefetch milik `powershell.exe` dan `pwsh.exe`.
4. Bandingkan `FileLastWriteTime` dengan waktu `$MFT` dan USN Journal milik berkas riwayat, serta dengan logon, untuk mempersempit kapan baris-baris terakhir ditambahkan.

## Detail lebih lanjut { #further-details }

- [Microsoft — about_PSReadLine: command history](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath and HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: script block logging](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — how the history file is saved and read](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
