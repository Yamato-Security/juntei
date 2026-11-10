# Instal Juntei { #install-juntei }

## Biner rilis { #release-binaries }

Unduh berkas zip untuk sistem Anda dari
[halaman rilis Juntei](https://github.com/Yamato-Security/juntei/releases). Jika belum ada biner rilis yang
dipublikasikan, kompilasi dari kode sumber seperti dijelaskan di bawah. Simpan arsip atau commit sumber beserta
checksum-nya bersama catatan kasus Anda agar analis lain dapat mereproduksi eksekusinya.

| Sistem | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` atau `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` atau `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Setiap zip hanya berisi biner, yang diberi nama dengan versi dan platformnya (misalnya
`juntei-1.0.0-win-x64.exe`): Juntei tidak memerlukan berkas lain, dan biner Windows tidak memerlukan
Visual C++ Redistributable. Biner Linux musl ditautkan secara statis dan dapat berjalan di distribusi
apa pun; biner gnu memerlukan glibc versi terbaru. Di macOS, izinkan biner yang diunduh di
System Settings > Privacy & Security, atau hapus atribut karantinanya dengan
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

Contoh-contoh menggunakan `juntei` untuk biner tersebut; ganti namanya, atau ketik nama lengkapnya.

## Kompilasi dari kode sumber { #compile-from-source }

Juntei saat ini memerlukan versi Rust yang dideklarasikan di `Cargo.toml` (Rust 1.99 pada saat halaman
ini ditulis).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` menggunakan revisi dependensi persis yang tercatat di `Cargo.lock`.

## Bentuk dasar perintah { #basic-command-shape }

Setiap parser menerima satu berkas (`-f`) atau sebuah direktori hasil pengumpulan (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Tampilkan opsi saat ini alih-alih menyalin perintah lama dari sebuah laporan:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Lanjutkan ke [Investigasi pertama](first-investigation.md) untuk alur kerja yang aman.
