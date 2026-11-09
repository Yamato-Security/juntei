# Memasang Image Disk { #mounting-disk-images }

`juntei mount` menyediakan berkas di dalam image disk VHDX melalui direktori hanya-baca selama perintah berjalan. Alat seperti Hayabusa dan Juntei dapat memeriksanya tanpa ekstraksi atau hak root. Fitur ini menggunakan FUSE dan berjalan di Linux. Pembaca dan pustaka mount dipelihara di [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
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

Parser memerlukan jalur: Juntei menyalin berkas NTFS yang tersedia ke ruang sementara privat dengan buffer terbatas. **Sediakan ruang untuk semua berkas tersebut**, bukan hanya artefak yang menghasilkan rekaman. Wilayah nol ditulis sebagai lubang jika didukung. Nama, struktur, hard link, dan waktu modifikasi dipertahankan. Waktu sumber LNK dan Jump List berasal dari snapshot metadata NTFS asli, bukan salinan sementara. Penyelesaian normal, termasuk kesalahan parser, menghapus salinan; penghentian paksa dapat meninggalkannya. Sumber tetap hanya-baca; log VHDX tertunda diterapkan hanya di memori.

Penemuan host, CSV/JSON/JSONL, dan `-j` memakai alur biasa. `Image` dalam `manifest.json` menunjukkan image asli; `EvidenceRoot` menunjukkan jalur relatif di dalamnya. Kesalahan baca dan nama tidak aman dicatat dalam `ImageErrors` dan baris gagal `manifest.csv`. Hasil yang terbaca dipertahankan; input tidak lengkap mengembalikan 1. Nama berbahaya atau ambigu tidak diganti. Stream NTFS bernama tidak diekstrak sebagai berkas tambahan; artefak yang sudah dikumpulkan sebagai berkas biasa tetap tersedia.

Perintah [`mount` terpisah](mount.md) menyediakan sistem berkas bagi program lain dan tetap memerlukan Linux FUSE. `auto-parse --vhdx` tidak memakainya.

## Persyaratan mount yang terlihat oleh OS { #os-visible-mount-requirements }

`juntei mount` memerlukan Linux FUSE 3 (`fusermount3`) dan akses ke `/dev/fuse`. Di Windows dan macOS, gunakan analisis native di atas.
