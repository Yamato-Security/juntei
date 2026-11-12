# Riwayat PowerShell { #powershell-history }

## Apa yang dicatat riwayat { #what-the-history-records }

PSReadLine menyimpan baris perintah yang dimasukkan dalam host PowerShell interaktif. Berkas berisi teks sesuai urutan penyimpanan, tanpa stempel waktu per perintah. Riwayat dapat diedit, dihapus, dinonaktifkan, atau difilter dan tidak membuktikan bahwa perintah selesai dengan sukses. Cocokkan eksekusi dan waktunya dengan log peristiwa PowerShell, telemetri proses, dan artefak eksekusi lainnya.

## Lokasi dan pengumpulan { #location-and-collection }

Lokasi bawaan di Windows untuk Windows PowerShell 5.1 dan PowerShell 7+ adalah:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Host lain memakai nama seperti `Visual Studio Code Host_history.txt` dalam direktori tersebut. PowerShell di Linux dan macOS biasanya memakai `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. `HistorySavePath` milik PSReadLine dapat mengalihkan berkas; jalur bawaan ini merupakan asumsi pencarian, bukan daftar lokasi yang lengkap.

Masukan direktori dan `auto-parse` memilih `<Host>_history.txt` secara rekursif yang berada langsung dalam folder `PSReadLine`, tanpa membedakan huruf besar/kecil ASCII. Keduanya tidak memilih sembarang berkas `*_history.txt` di tempat lain. Gunakan masukan `-f` secara eksplisit untuk berkas yang diganti namanya atau salinan di luar direktori `PSReadLine`. Jika memungkinkan, pertahankan struktur profil asli untuk atribusi pengguna.

## Mengurai dengan Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV adalah format berkas bawaan. Dengan `-t csv`, `-t json`, atau `-t jsonl` tanpa `-o`, rekaman ditulis ke stdout. Tanpa kedua opsi tersebut, hanya ringkasan yang ditampilkan. JSON berisi objek terpisah dengan format yang mudah dibaca; JSONL berisi satu objek per baris. Tambahkan `-C` untuk menimpa keluaran yang sudah ada; alias berkas sumber dilindungi dari penggantian.

`auto-parse` menyimpan `ShellHistory/PowerShellHistory.csv` (atau format JSON yang dipilih), termasuk untuk image VHDX/E01. Riwayat tidak menyumbangkan peristiwa ke `auto-parse -j` atau `super-timeline`.

## Encoding dan penanganan baris { #encoding-and-line-handling }

- Penanda urutan byte UTF-8, UTF-16LE, atau UTF-16BE menentukan encoding dan dihapus dari awal berkas. Tanpa BOM, masukan selalu diperlakukan sebagai UTF-8. UTF-16 tanpa BOM tidak ditebak; karakter NUL dipertahankan dan dilaporkan sebagai kemungkinan kerusakan encoding.
- Urutan UTF-8 yang tidak valid dan surrogate UTF-16 tanpa pasangan menjadi U+FFFD, dengan kerusakan dilaporkan dalam `Problems`. Byte UTF-16 sisa di akhir dan kesalahan baca juga dilaporkan; baris yang dapat dibaca tetap disimpan. Periksa `Problems` dan log parser sebelum mengandalkan kelengkapannya.
- Berkas diproses secara streaming. Maksimal 1,048,576 byte masukan dipertahankan per baris fisik, tidak termasuk BOM dan pemisah LF (CR sebelum LF dihitung dalam batas). Baris yang lebih panjang dipotong pada batas karakter utuh; byte sisanya tetap dibaca dan `Problems` melaporkan jumlah byte yang dihilangkan. Batas berlaku untuk byte masukan yang dikodekan, bukan jumlah karakter.
- Akhir baris LF dan CRLF dihapus, begitu pula CR terakhir pada baris tanpa terminator. Spasi dan tab lain dipertahankan. Baris kosong tidak menghasilkan rekaman, tetapi tetap dihitung dalam nomor baris dan mengakhiri rantai kelanjutan; baris yang hanya berisi whitespace tetap menjadi rekaman. Kesalahan baca dapat menghasilkan baris diagnostik kosong untuk menandai titik berhentinya pembacaan.
- PSReadLine menyimpan entri multibaris dengan backtick di akhir setiap baris fisik yang berlanjut. Baris tetap terpisah, dengan backtick dan indentasi dipertahankan. `EntryStartLine` menunjuk ke baris pertama dalam rantai berurutan yang baris-baris sebelumnya berakhir dengan backtick. Sintaks shell tidak ditafsirkan dan perintah tidak direkonstruksi. Pemotongan dapat memengaruhi atribusi kelanjutan; periksa `Problems` sebelum mengelompokkan baris tersebut.

## Kolom keluaran { #output-fields }

| Kolom | Interpretasi |
|---|---|
| `SourceFile` | Jalur relatif terhadap direktori masukan, memakai `/`; untuk `-f`, hanya nama berkas. Jalur sumber absolut tidak diekspor. |
| `User` | Nama profil yang disimpulkan dari jalur yang dipertahankan. Masukan direktori juga mengenali struktur `Users`/`Documents and Settings` di bawah akar masukan; `-f` memerlukan akhiran jalur profil PSReadLine yang lazim. Nilai yang tidak diketahui kosong dalam CSV dan `null` dalam JSON. |
| `ShellHost` | Awalan nama berkas sebelum `_history.txt`, seperti `ConsoleHost`; tidak diketahui untuk berkas yang diganti namanya tanpa akhiran tersebut. |
| `LineNumber` | Nomor baris fisik mulai dari 1, termasuk baris kosong yang dilewati. |
| `EntryStartLine` | Baris fisik awal rantai kelanjutan, atau nomor baris ini sendiri. |
| `Command` | Baris fisik yang didekode tanpa akhir baris, dengan batas yang didokumentasikan dan pelindung CSV. |
| `FileLastWriteTime` | Waktu modifikasi berkas sumber, diulang pada setiap baris. **Metadata berkas, bukan waktu eksekusi perintah.** Penguraian image memakai metadata sumber asli, bukan waktu staging. |
| `FileSize` | Ukuran berkas sumber dalam byte. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE`, atau `UTF-16BE`. |
| `Problems` | Diagnostik kerusakan dan pemotongan baris; dipisahkan titik koma dalam CSV, berupa array dalam JSON/JSONL. Diagnostik tingkat berkas juga tercatat dalam log. |

`FileLastWriteTime` secara bawaan memakai UTC dalam format ISO 8601, misalnya `2026-10-20T08:46:38.5Z`. Hanya `auto-parse` yang menerima `-Z, --timezone` untuk mengubah offset yang ditampilkan. Perintah mandiri `parse-file powershell-history` tidak memiliki opsi zona waktu.

Setiap sel teks CSV diperiksa untuk awalan rumus spreadsheet (`=`, `+`, `-`, `@`, termasuk bentuk lebar penuh dan awalan setelah whitespace atau karakter tak terlihat/kontrol tertentu). Sel yang terpengaruh dan sel yang diawali karakter kontrol diberi apostrof di depan. Ini dapat mengubah teks perintah dan jalur dalam CSV. JSON dan JSONL mempertahankan teks yang didekode tanpa apostrof CSV; penggantian saat decoding dan pemotongan tetap berlaku. Gunakan JSONL untuk membandingkan teks perintah yang tersimpan, dan selalu pertahankan berkas riwayat sumber.
