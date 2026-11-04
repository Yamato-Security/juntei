# Deskriptor keamanan (`$Secure:$SDS`) { #security-descriptors-securesds }

## Apa itu `$SDS` { #what-sds-is }

Setiap berkas dan direktori pada NTFS memiliki **security descriptor**: pemiliknya, grup primernya,
**DACL** (siapa yang boleh melakukan apa terhadapnya), dan **SACL** (akses apa yang diaudit, serta label integritasnya).
NTFS menyimpan setiap descriptor yang berbeda satu kali, di stream `$SDS` milik berkas `$Secure` (entri MFT 9),
dan memberinya **security ID**. `$STANDARD_INFORMATION` sebuah berkas hanya mencatat security ID tersebut: kolom
`SecurityID` dari [`parse-ntfs mft`](mft.md). `$SDS` mengubahnya menjadi pemilik dan ACL.

`$SDS` tersusun dalam blok 256 KiB, masing-masing diikuti oleh salinannya (mirror). Setiap entri memiliki hash
descriptor-nya, yang digunakan NTFS untuk menemukan descriptor yang sudah ada (`$SDH`), serta
security ID (diindeks oleh `$SII`).

<span class="evidence-strength">Kesimpulan terbaik:</span> berkas-berkas yang `SecurityID`-nya menunjuk sebuah
descriptor memiliki pemilik dan ACL tersebut pada saat catatan `$MFT`-nya terakhir ditulis. Pemilik sebuah berkas
biasanya adalah akun yang membuatnya, tetapi administrator, installer, atau program
yang mengambil alih kepemilikan dapat mengubahnya.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Kumpulkan dengan alat forensik yang mampu membaca data mentah (raw); hasil pengumpulan biasanya menamainya `$Secure_$SDS`, karena `:`
tidak boleh ada dalam nama berkas. Kumpulkan `$MFT` dari volume yang sama bersamanya.

## Peringatan bagi analis { #analyst-cautions }

- Pemilik adalah sebuah SID. Hanya well-known SID (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) dan akun bawaan sebuah mesin atau domain (`-500`
  `Administrator`, …) yang diberi nama. Resolusikan sisanya dengan `SAM` milik host
  ([akun SAM](sam.md)), daftar profilnya ([Informasi sistem](system-info.md)), atau domain.
- Sebuah descriptor digunakan bersama oleh setiap berkas dengan pemilik dan ACL yang sama, dan biasanya tetap berada di `$SDS`
  setelah berkas-berkas tersebut dihapus. Descriptor tanpa berkas di `$MFT` tidak dengan sendirinya merupakan
  berkas yang terhapus.
- ACE bukan riwayat akses. DACL menyatakan siapa yang *boleh* mengakses sebuah berkas; log peristiwa dan artefak
  lain menyatakan siapa yang benar-benar melakukannya. Entri audit SACL mendeskripsikan konfigurasi, bukan apa yang dicatat di log.
- ACE dicantumkan sesuai urutannya di ACL. Juntei tidak menghitung akses efektif: entri deny,
  pewarisan, keanggotaan grup, hak istimewa, dan hak implisit pemilik semuanya berpengaruh.
- `DACL` dan `SACL` menuliskan setiap ACE seperti yang dilakukan `icacls`. `SDDL` merepresentasikan izin yang didukung, dengan mask dalam heksadesimal dan akun sebagai SID. Ini bukan salinan byte demi byte dari descriptor biner; teks kondisional yang tidak dapat direpresentasikan membuatnya kosong dan didiagnosis.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

Ringkasan menghitung descriptor dan mencantumkan pemiliknya, beserta berapa banyak descriptor yang dimiliki masing-masing,
descriptor dengan NULL DACL (tanpa pembatasan sama sekali), dan kerusakan apa pun.

## Field keluaran CSV { #csv-output-fields }

| Field | Interpretasi |
|---|---|
| `SourcePath` | Berkas masukan. |
| `SecurityID` | Security ID: gabungkan dengan `SecurityID` dari [`parse-ntfs mft`](mft.md) untuk volume yang sama. |
| `Hash` | Hash yang dicatat bersama descriptor. |
| `EntryOffset` | Lokasi entri di dalam stream. |
| `FromMirror` | `True` untuk salinan yang dibaca dari blok mirror: salinan yang berbeda dari entrinya (dicantumkan setelahnya), atau salinan dari entri yang rusak. |
| `Size` | Ukuran entri, termasuk header 20 byte-nya. |
| `Owner`, `OwnerName` | SID pemilik, dan namanya jika merupakan well-known SID. |
| `Group`, `GroupName` | SID grup primer, dan namanya jika merupakan well-known SID. Windows tidak menggunakannya untuk pemeriksaan akses. |
| `Control` | Control flag descriptor: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (pewarisan diblokir), `SE_DACL_AUTO_INHERITED`, … digabungkan dengan `|`. |
| `DACL-ACE-Count`, `DACL` | Entri-entri DACL, dipisahkan oleh `; `. Kosong jika tidak ada DACL; `NULL DACL: no restriction` jika descriptor memberikan segalanya kepada semua orang. |
| `SACL-ACE-Count`, `SACL` | Entri-entri SACL: entri audit, mandatory integrity label, process trust label, dan resource attribute. |
| `SDDL` | Seluruh descriptor dalam SDDL. Kosong hanya jika ada bagian di dalamnya yang tidak didekode. |
| `Problems` | Hash yang bukan milik descriptor, bagian yang tidak dapat didekode, ACE yang tipenya tidak dikenal. |

Setiap entri `DACL` dan `SACL` ditulis sebagaimana `icacls` menuliskannya: akun, tipe jika
bukan entri allow, flag pewarisan, dan hak.

| Bagian | Makna |
|---|---|
| Akun | Nama well-known, atau SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | Tipe ACE, bila bukan entri allow. Mandatory label tidak menampilkan tipe. |
| `(I)` | Diwarisi dari direktori induk. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Diwarisi oleh berkas (object inherit), oleh direktori (container inherit), hanya oleh keduanya dan bukan objek ini (inherit only), dan hanya satu tingkat ke bawah (no propagate). |
| `(SA)`, `(FA)` | Entri audit untuk akses yang berhasil atau gagal. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Full control, modify, read and execute, read, write. |
| `(RD,WD,AD,…)` | Hak spesifik untuk kasus lainnya: read data, write data, append data, read dan write extended attributes (`REA`, `WEA`), execute (`X`), delete child (`DC`), read dan write attributes (`RA`, `WA`), delete (`DE`), read control (`RC`), write DAC (`WDAC`), write owner (`WO`), synchronize (`S`), serta hak generik `GA`, `GR`, `GW`, `GE`. `N` berarti tidak ada akses. |
| `(NW)`, `(NR)`, `(NX)` | Kebijakan mandatory label: no write up, no read up, no execute up. |
| `if (…)` | Kondisi callback ACE, seperti `(WIN://SYSAPPID Contains "…")`: entri hanya berlaku untuk proses yang memenuhi kondisi tersebut. Windows menggunakannya untuk memberikan akses kepada paket app container. |

Process trust label dan resource attribute menampilkan mask-nya dalam heksadesimal, dan resource
attribute juga menampilkan klaimnya (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL memiliki field dan nama yang sama, dengan angka sebagai angka, `null` untuk nilai kosong, dan
`Problems` sebagai list.

## Strategi investigasi { #investigation-strategy }

1. Uraikan `$MFT` dari volume yang sama dan gabungkan `SecurityID`-nya dengan keluaran ini: setiap berkas mendapatkan
   pemilik dan ACL-nya.
2. Perhatikan pemilik yang berupa akun pengguna (`S-1-5-21-…`): berkas mana yang dimiliki akun mencurigakan,
   dan akun mana yang memiliki berkas di direktori sistem.
3. Cari ACL yang permisif pada direktori tempat program dijalankan atau dimuat: `Everyone`,
   `Authenticated Users`, atau `User` dengan `(F)`, `(M)`, atau `(W)`, serta NULL DACL.
4. Descriptor yang hanya digunakan oleh satu berkas, atau oleh sekumpulan berkas yang tidak biasa, dapat menunjukkan berkas yang
   dibuat atau diubah di luar instalasi normal.
5. Korroborasi perubahan kepemilikan dan izin dengan entri log peristiwa Security (4670, 4663 dengan
   SACL) dan catatan `SECURITY_CHANGE` di [Jurnal USN](usn-journal.md).

## Detail lebih lanjut { #further-details }

- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (security descriptor, ACE, ekspresi kondisional)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: dokumentasi format NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `secure`-nya)

Dengan keluaran terstruktur di stdout (`-t` tanpa `-o`), peringatan pemulihan tingkat stream dikirim ke stderr, sehingga pipe tetap mempertahankan CSV/JSON/JSONL yang dapat dibaca tanpa menyembunyikan wilayah yang rusak atau dipulihkan.
