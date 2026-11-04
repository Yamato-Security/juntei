# Konfigurasi boot (BCD) { #boot-configuration-data-bcd }

## Apa itu BCD store { #what-the-bcd-store-is }

Windows Vista dan versi yang lebih baru menyimpan konfigurasi boot-nya di sebuah hive registry, yaitu BCD store:
`\Boot\BCD` pada sistem BIOS, `\EFI\Microsoft\Boot\BCD` pada partisi sistem EFI di sistem
UEFI. `bcdedit` dan installer Windows menyuntingnya; boot manager membacanya pada setiap boot.

Store ini menyimpan objek-objek, yang masing-masing dinamai dengan sebuah GUID:

- **Windows boot manager** (`{bootmgr}`), dengan entri default, urutan tampilan, dan
  batas waktu menu; pada sistem UEFI juga **firmware boot manager** (`{fwbootmgr}`) dengan
  urutan boot firmware;
- satu **Windows boot loader** untuk setiap instalasi Windows (termasuk lingkungan pemulihan), sebuah
  **resume application** (hibernasi), dan **memory tester**;
- objek **inherited settings** (`{globalsettings}`, `{bootloadersettings}`, …) yang pengaturannya
  diwarisi oleh objek lain;
- objek **device** (opsi ramdisk dari lingkungan pemulihan).

Setiap objek memiliki elemen: deskripsi, device dan path, serta pengaturan seperti
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks`, dan `safeboot`.

Dua jenis serangan mengubahnya:

- **Ransomware** menonaktifkan pemulihan sehingga sistem tidak dapat diperbaiki:
  `bcdedit /set {default} recoveryenabled No` dan
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Beberapa keluarga ransomware menetapkan `safeboot` agar
  Windows dimulai ulang dalam safe mode, tempat produk keamanan tidak berjalan.
- **Rootkit dan driver yang tidak ditandatangani** dapat menggunakan `testsigning on` atau `nointegritychecks on` untuk melemahkan pemeriksaan saat boot. Pengaturan ini saja tidak membuktikan bahwa sebuah rootkit telah berjalan.

<span class="evidence-strength">Kesimpulan terbaik:</span> pada saat pengumpulan, entri boot ini
memiliki pengaturan ini. Waktu penulisan terakhir key milik elemen tersebut adalah waktu terakhir pengaturan itu ditulis.

## Lokasi { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Tipe sebuah elemen terdiri dari 8 digit heksadesimal: kelasnya (`1` setiap aplikasi, `2` satu jenis
aplikasi, `3` device), formatnya (`1` device, `2` string, `3` objek, `4` daftar objek, `5`
integer, `6` boolean, `7` daftar integer), dan nomornya. Perintah ini menamai elemen sesuai
dokumentasi Microsoft (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Alat pengumpulan sering kali tidak menyalin store ini: partisi sistem EFI tidak di-mount, dan `\Boot`
tersembunyi. Kumpulkan secara eksplisit, atau mount partisinya. `-d` juga membaca `BCD-Template`
(`HiveType` menunjukkan yang mana); template adalah titik awal Windows, bukan store yang digunakan untuk boot.

## Peringatan bagi analis { #analyst-cautions }

- **Pengaturan diwariskan.** Sebuah boot loader mewarisi dari objek-objek di
  `BcdLibraryObjectList_InheritedObjects` miliknya (biasanya `{bootloadersettings}`, yang mewarisi
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot`, dan
  `BootStatusPolicy` adalah nilai tersimpan yang telah di-resolve (milik sendiri, atau jika tidak ada, warisan), dan `Inherited` menunjukkan mana yang
  berasal dari objek lain. Pewarisan diikuti paling banyak delapan tingkat, dengan siklus yang dibatasi. Objek yang direferensikan bisa saja tidak ada di store yang parsial. Field kosong tidak menetapkan default Windows yang efektif. Firmware dan Secure Boot juga dapat mencegah opsi yang tersimpan berlaku.
- **Tidak diatur bukan berarti nonaktif.** `RecoveryEnabled` yang kosong berarti elemennya tidak ada; default
  Windows yang berlaku.
- **Waktu bersifat per elemen.** Waktu key setiap elemen (`key_last_written` di `Elements`) adalah waktu
  elemen tersebut terakhir ditulis. Windows menulis ulang beberapa elemen saat pembaruan dan upgrade; bandingkan dengan waktu
  elemen-elemen lainnya.
- **Device hanya didekode sebagian.** Partisi GPT ditampilkan sebagai GUID partisi dan disknya;
  ramdisk sebagai objek opsinya. Device lain hanya menyimpan byte-nya (`data_raw`).
- **Elemen yang tidak terdokumentasi** (`0x15000065`, kelas `0x4…`) tidak memiliki nama dan menyimpan byte-nya.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Ringkasan mencantumkan entri default setiap store serta setiap pengaturan test signing, pemeriksaan integritas, pemulihan,
boot status policy (setiap `Ignore…`), dan safe mode, beserta waktu pengaturan itu ditetapkan atau objek
asal pewarisannya.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive, dan `BCD` atau `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | GUID objek, tipenya (`0x10200003`), dan jenis objeknya (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | Deskripsi, path aplikasi, dan device-nya (partisi dan disk GPT, atau objek opsi ramdisk). |
| `IsDefault`, `DisplayOrder` | Apakah Windows boot manager mem-boot-nya secara default, dan posisinya di menu boot. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, sebagaimana di-resolve dari store. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) dan `bootstatuspolicy` (`IgnoreAllFailures`, …), beserta nomornya. |
| `Inherited` | Mana di antara pengaturan ini yang berasal dari objek lain, dan dari objek mana. |
| `Elements` | Setiap elemen: tipe, nama, nilai, byte (untuk device, tipe yang tidak terdokumentasi, dan data yang tidak dapat didekode), dan waktu key-nya. |
| `KeyLastWrittenUTC` | Waktu penulisan terakhir key milik objek. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Objek tanpa tipe, elemen yang datanya tidak sesuai dengan formatnya. |

## Strategi investigasi { #investigation-strategy }

1. Periksa entri default dan setiap boot loader untuk `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, `BootStatusPolicy` bernilai `Ignore…`, dan `SafeBoot`.
2. Gunakan waktu elemen untuk menentukan tanggal perubahan, dan korelasikan dengan `bcdedit.exe` di Prefetch,
   Amcache, peristiwa pembuatan proses (4688, Sysmon 1), dan log peristiwa System.
3. Cari entri boot atau device yang seharusnya tidak ada (path loader lain, ramdisk), serta
   default atau urutan tampilan yang berubah.
4. Bandingkan store dengan `BCD-Template` dan dengan sistem lain yang memiliki build yang sama.

## Detail lebih lanjut { #further-details }

- [Microsoft: Referensi opsi boot BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (tipe objek)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
