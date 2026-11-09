# Memasang Image Disk { #mounting-disk-images }

`juntei mount` menyediakan berkas di dalam image disk (image VHDX atau E01) melalui direktori hanya-baca selama perintah berjalan. Alat seperti Hayabusa dan Juntei dapat memeriksanya tanpa ekstraksi atau hak root. Fitur ini menggunakan FUSE dan berjalan di Linux. Pembaca VHDX dan pustaka mount dipelihara di [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser), sedangkan pembaca E01 di [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Image E01 (Expert Witness Format milik EnCase, juga `.Ex01`) di-mount dengan cara yang sama. Image terpisah (split) diberikan melalui segmen pertamanya, dan Juntei menemukan segmen lainnya (`host1.E02`, `host1.E03`, …) di sebelahnya:

```bash
juntei mount host1.E01 /mnt/host1
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
- Image dari satu volume, tanpa tabel partisi, adalah volume itu sendiri.
- Nama dicantumkan sebagaimana NTFS menyimpannya dan dicari secara persis. Nama DOS 8.3 dan alternate
  data stream tidak ditampilkan.
- Berkas terkompresi didekompresi (hingga 1 GiB per berkas). Berkas yang dienkripsi dengan EFS tidak dapat dibaca.

## Bukti tidak diubah { #the-evidence-is-not-changed }

Image dibuka hanya-baca dan mount menolak penulisan. Log tertunda dari VHDX yang tidak ditutup dengan benar diterapkan di memori, tanpa pernah ditulis kembali. Beberapa writer DiscUtils membiarkan instruksi lompat dan signature `55 AA` pada boot sector NTFS bernilai nol; Juntei membaca volume tersebut apa adanya.

Segmen-segmen image E01 juga dibuka hanya-baca. Setiap chunk diperiksa terhadap checksum-nya saat dibaca: chunk yang rusak tidak dapat dibaca, sedangkan bagian image lainnya tetap dapat dibaca. Sektor yang tidak dapat dibaca saat akuisisi, serta akuisisi yang tidak selesai, dilaporkan sebagai peringatan. Berkas bukti logis (`.L01`, `.Lx01`) berisi berkas, bukan disk, sehingga tidak di-mount.

## Penggunaan { #usage }

```text
Version: 1.1.0 (Dev Build)
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

## Mengurai setiap image dengan auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` mengurai sistem-sistem Windows di dalam setiap image disk VHDX di direktori bukti
(atau satu image yang diberikan dengan `-d`), dan `auto-parse --e01` mengurai sistem di dalam setiap image E01,
tanpa mount terpisah. Kedua opsi dapat diberikan bersamaan:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- Setiap image di-mount dalam mode baca saja di sebuah direktori sementara baru, dalam folder yang dinamai sesuai
  image tersebut (`host1-triage`), dan sistem-sistem yang ditemukan di sana diurai seperti pada hasil pengumpulan
  di disk. Mount image-image tersebut dilepas di akhir eksekusi, dan juga saat Ctrl+C ditekan.
- Terminal mencantumkan image-image tersebut beserta image dari setiap sistem:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- `--e01` mengambil segmen pertama dari setiap image E01 (`*.E01`, `*.Ex01`); segmen lainnya dibaca
  bersamanya.
- Di manifest, `EvidenceDirectory` adalah direktori yang diberikan dan `Image` milik setiap sistem menyebutkan
  image-nya. `EvidenceRoot` dan perintah parser menggunakan folder mount (`./host1-triage/C`).
- Jika image berisi berkas dari Volume Shadow Copies dalam folder seperti `VSS32`, setiap folder ditemukan dan diurai sebagai sistem lain (`HOST1_2`, …).
- Image yang tidak dapat di-mount dilaporkan; image lainnya tetap diurai, dan eksekusi berakhir dengan
  exit status 1.

Jika sebuah volume NTFS tidak dapat dibuka, Juntei melaporkan masalahnya dan mempertahankan volume yang dapat dibaca. Untuk citra yang hanya terbaca sebagian, `auto-parse --vhdx` dan `--e01` mengembalikan kode keluar 1; `mount` juga mengembalikan 1 setelah pelepasan.

## Persyaratan { #requirements }

- Linux: FUSE 3 (`fusermount3`, paket `fuse3`) dan akses ke `/dev/fuse`, yang biasanya dimiliki oleh pengguna
  biasa. Root tidak diperlukan.
- Windows dan macOS tidak didukung. Di Windows, sebagai gantinya, pasang image VHDX dalam mode baca saja di PowerShell
  yang dijalankan dengan hak administrator (elevated), lalu berikan huruf drive-nya ke alat-alat tersebut. Windows tidak dapat
  memasang image E01: mount image tersebut dalam mode baca saja dengan alat yang mampu melakukannya, lalu berikan drive itu ke alat-alat tersebut.

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
