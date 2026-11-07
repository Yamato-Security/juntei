# Memasang Image Disk { #mounting-disk-images }

`juntei mount` membuat berkas-berkas di dalam image disk VHDX dapat dibaca di sebuah direktori, dalam mode baca saja,
selama perintah ini berjalan. Hasil pengumpulan `--vhdx` KAPE dan disk Hyper-V kemudian dapat dipindai oleh alat apa pun,
seperti Hayabusa atau perintah Juntei sendiri, tanpa mengekstrak apa pun, tanpa root, dan tanpa memerlukan sistem operasi
yang mendukung VHDX atau NTFS. Perintah ini menggunakan FUSE dan berjalan di Linux.

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

- Berkas-berkas volume NTFS di titik mount: untuk hasil pengumpulan KAPE, folder `C` yang berisi
  berkas-berkas yang dikumpulkan, folder `LongFileNames` milik KAPE, dan berkas metadata milik volume itu sendiri
  seperti `$MFT`. Image yang berisi beberapa volume NTFS menampilkannya sebagai `Volume1`, `Volume2`, dan seterusnya.
- Setiap berkas bersifat baca saja dan dimiliki oleh pemilik titik mount. Waktu berkas adalah waktu
  `$STANDARD_INFORMATION` NTFS.
- Nama dicantumkan sebagaimana NTFS menyimpannya dan dicari secara persis. Nama DOS 8.3 dan alternate
  data stream tidak ditampilkan.
- Berkas terkompresi didekompresi (hingga 1 GiB per berkas). Berkas yang dienkripsi dengan EFS tidak dapat dibaca.

## Bukti tidak diubah { #the-evidence-is-not-changed }

Image dibuka dalam mode baca saja dan mount menolak setiap penulisan. VHDX yang tidak ditutup dengan bersih dibaca
dengan log tertundanya diterapkan di memori, dan tidak pernah ditulis kembali. KAPE menulis berkas VHDX-nya dengan
DiscUtils, yang membuat sektor boot NTFS tidak memiliki instruksi jump dan signature `55 AA`; Juntei membaca
volume-volume tersebut apa adanya.

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

## Mengurai setiap image dengan auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` mengurai sistem-sistem Windows di dalam setiap image disk VHDX di direktori bukti
(atau satu image yang diberikan dengan `-d`), tanpa mount terpisah:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Setiap image di-mount dalam mode baca saja di sebuah direktori sementara baru, dalam folder yang dinamai sesuai
  image tersebut (`host1-triage`), dan sistem-sistem yang ditemukan di sana diurai seperti pada hasil pengumpulan
  di disk. Mount image-image tersebut dilepas di akhir eksekusi, dan juga saat Ctrl+C ditekan.
- Terminal mencantumkan image-image tersebut beserta image dari setiap sistem:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- Di manifest, `EvidenceDirectory` adalah direktori yang diberikan dan `Image` milik setiap sistem menyebutkan
  image-nya. `EvidenceRoot` dan perintah parser menggunakan folder mount (`./host1-triage/C`).
- KAPE menyimpan berkas yang disalinnya dari Volume Shadow Copies di folder seperti `VSS32`. Masing-masing folder
  tersebut ditemukan dan diurai sebagai sistem tersendiri (`HOST1_2`, …).
- Image yang tidak dapat di-mount dilaporkan; image lainnya tetap diurai, dan eksekusi berakhir dengan
  exit status 1.

## Persyaratan { #requirements }

- Linux: FUSE 3 (`fusermount3`, paket `fuse3`) dan akses ke `/dev/fuse`, yang biasanya dimiliki oleh pengguna
  biasa. Root tidak diperlukan.
- Windows dan macOS tidak didukung. Di Windows, sebagai gantinya, pasang image dalam mode baca saja di PowerShell
  yang dijalankan dengan hak administrator (elevated), lalu berikan huruf drive-nya ke alat-alat tersebut:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
