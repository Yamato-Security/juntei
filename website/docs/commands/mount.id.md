# Memasang Image Disk { #mounting-disk-images }

`juntei mount` menyediakan berkas di dalam image disk VHDX / E01 / Ex01 melalui direktori hanya-baca selama perintah berjalan. Alat seperti Hayabusa dan Juntei dapat memeriksanya tanpa ekstraksi atau hak root. Fitur ini menggunakan FUSE dan berjalan di Linux. Pembaca dan pustaka mount dipelihara di [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Di terminal lain:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Tekan Ctrl+C di terminal pertama untuk melepas mount, atau jalankan `fusermount3 -u /mnt/host1`.

## Apa yang ditampilkan mount { #what-the-mount-shows }

- Berkas volume NTFS tampil di titik mount. Image hasil pengumpulan dapat berisi direktori seperti `C` dan `LongFileNames`, serta metadata volume seperti `$MFT`. Beberapa volume NTFS tampil sebagai `Volume1`, `Volume2`, dan seterusnya.
- Setiap berkas bersifat baca saja dan dimiliki oleh pemilik titik mount. Waktu berkas adalah waktu
  `$STANDARD_INFORMATION` NTFS.
- Nama dicantumkan sebagaimana NTFS menyimpannya dan dicari secara persis. Nama DOS 8.3 dan alternate
  data stream tidak ditampilkan.
- Berkas terkompresi didekompresi (hingga 1 GiB per berkas). Berkas yang dienkripsi dengan EFS tidak dapat dibaca.

## Bukti tidak diubah { #the-evidence-is-not-changed }

Image dibuka hanya-baca dan mount menolak penulisan. Log tertunda dari VHDX yang tidak ditutup dengan benar diterapkan di memori, tanpa pernah ditulis kembali. Beberapa writer DiscUtils membiarkan instruksi lompat dan signature `55 AA` pada boot sector NTFS bernilai nol; Juntei membaca volume tersebut apa adanya.

## Penggunaan { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Analisis image secara native { #parsing-every-image-with-auto-parse }

Dengan `--vhdx`, `auto-parse` membaca VHDX secara native di **Linux, macOS, dan Windows** melalui [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). Tidak perlu mount OS, FUSE, atau hak administrator.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Parser memerlukan jalur: Juntei menyalin berkas yang dibaca parser ke ruang sementara privat dengan buffer terbatas: hive registri beserta log transaksinya, berkas metadata NTFS, basis data, berkas prefetch, pintasan, Jump List, berkas Outlook, berkas indeks Recycle Bin, dan folder karantina Defender. Berkas lain menjadi placeholder kosong dengan waktu modifikasi aslinya, sehingga direktori dan jalur terlihat sama seperti di disk. persistence-hunter membaca isi placeholder dari image hanya saat diperlukan, misalnya berkas eksekusi yang dirujuk entri persistensi. **Sediakan ruang untuk berkas yang disalin**, selain untuk hasil. Wilayah nol ditulis sebagai lubang jika didukung. Nama, struktur, hard link, dan waktu modifikasi dipertahankan. Waktu sumber LNK dan Jump List berasal dari snapshot metadata NTFS asli, bukan salinan sementara. Penyelesaian normal, termasuk kesalahan parser, menghapus salinan; penghentian paksa dapat meninggalkannya. Sumber tetap hanya-baca; log VHDX tertunda diterapkan hanya di memori.

Penemuan host, CSV/JSON/JSONL, dan `-j` memakai alur biasa. `Image` dalam `manifest.json` menunjukkan image asli; `EvidenceRoot` menunjukkan jalur relatif di dalamnya. Kesalahan baca dan nama tidak aman dicatat dalam `ImageErrors` dan baris gagal `manifest.csv`. Hasil yang terbaca dipertahankan; input tidak lengkap mengembalikan 1. Nama berbahaya atau ambigu tidak diganti. Stream NTFS bernama tidak diekstrak sebagai berkas tambahan; artefak yang sudah dikumpulkan sebagai berkas biasa tetap tersedia.

Perintah [`mount` terpisah](mount.md) menyediakan sistem berkas bagi program lain dan tetap memerlukan Linux FUSE. `auto-parse --vhdx` tidak memakainya.

Staging secara default memakai direktori privat pada sistem berkas output. Gunakan `--temp-dir` untuk memilih direktori yang sudah ada di disk lain. Metadata asli disimpan terpisah pada sistem berkas output, sehingga disk staging yang penuh tidak menghalangi parsing berkas yang sudah terbaca. Sediakan ruang untuk salinan dan hasil; jika volume output penuh, hasil tidak dapat disimpan. Ctrl+C menghentikan proses parser, membersihkan ruang privat, dan keluar dengan kode 130. SIGKILL atau listrik padam masih dapat meninggalkan salinan. Subdirektori yang tak dapat dibaca dilewati saat pencarian image; root input yang tak terbaca tetap merupakan kesalahan.

Direktori staging harus berada di luar direktori output. Di Linux dan macOS, SIGTERM dan SIGHUP juga menghentikan proses parser dan membersihkan ruang kerja.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## Image disk E01 dan Ex01 { #e01-and-ex01-disk-images }

`--e01` membaca image fisik EWF1 (`.E01`) dan EWF2 (`.Ex01`) secara native di **Linux, macOS dan Windows**, tanpa FUSE, mount OS, atau hak administrator. Fitur ini menggunakan [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) dan alur parsing NTFS yang sama seperti di atas.

Berikan segmen pertama untuk image terpisah. Simpan semua berkas pendamping, termasuk `.E02`, di sampingnya dengan nama asli. Disk penuh (MBR/GPT) dan volume NTFS tunggal didukung. Gunakan `--vhdx --e01` untuk direktori campuran.

Persyaratan ruang sementara dan metadata sumber tetap sama. Image E01 biasanya berisi seluruh disk, dan semua image di-staging sebelum parsing dimulai, tetapi hanya berkas yang dibaca parser yang disalin, biasanya sebagian kecil dari ruang terpakai. Image dibaca saja. Kekosongan akuisisi dan kerusakan saat membuka atau membaca dilaporkan dalam `ImageErrors` dan `manifest.csv`; hasil yang terbaca dipertahankan dengan kode keluar 1. Hanya data yang diakses yang diperiksa, bukan semua chunk atau hash seluruh media. Berkas logis `.L01` dan `.Lx01` ditolak; tidak ada opsi kata sandi.

Perintah terpisah `mount` tetap memerlukan Linux FUSE; parsing otomatis native tidak.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Persyaratan mount yang terlihat oleh OS { #os-visible-mount-requirements }

`juntei mount` memerlukan Linux FUSE 3 (`fusermount3`) dan akses ke `/dev/fuse`. Di Windows dan macOS, gunakan analisis native di atas.
