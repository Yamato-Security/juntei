# Perangkat USB { #usb-devices }

## Apa itu riwayat USB { #what-the-usb-history-is }

Windows mencatat setiap perangkat yang diinstalnya. Untuk penyimpanan USB, perangkat USB lain, dan perangkat
portabel (ponsel, kamera, pemutar media), hive `SYSTEM` menyimpan satu key per perangkat dengan nama,
nomor seri, dan waktu ketika Windows menginstalnya, terakhir melihatnya tersambung, dan terakhir melihatnya dilepas.
`MountedDevices` mencatat GUID volume dan huruf drive yang diberikan kepada volume-volumenya. Hive
`SOFTWARE` menyimpan nama yang diberikan Explorer kepada perangkat portabel dan volume, serta catatan
nomor seri volume milik ReadyBoost. `NTUSER.DAT` setiap pengguna mencatat volume yang pernah
di-mount oleh pengguna tersebut.

`parse-registry usb` menggabungkannya menjadi satu record per perangkat dan volume: perangkat apa itu, kapan
perangkat itu diinstal dan dihubungkan, huruf drive dan volume apa yang dimilikinya, dan pengguna mana yang pernah me-mount
volume tersebut.

<span class="evidence-strength">Kesimpulan terbaik:</span> perangkat dengan vendor, produk, dan
nomor seri ini pernah dihubungkan ke sistem ini, pertama dan terakhir kali pada waktu-waktu ini; volumenya memiliki
huruf drive ini; pengguna-pengguna ini pernah me-mount-nya. Record registry tidak menunjukkan berkas mana yang disalin.

## Lokasi { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Properti perangkat | Field | Makna (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: kapan driver-nya terakhir diinstal |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: kapan perangkat pertama kali diinstal |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: kapan perangkat terakhir dihubungkan (Windows 8 dan yang lebih baru) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: kapan perangkat terakhir dilepas (Windows 8 dan yang lebih baru) |

Windows 8 dan yang lebih baru menyimpan setiap waktu sebagai nilai default dari key `0064` … `0067`. Windows 7
menyimpannya di `00000064\00000000\Data`. Juntei membaca keduanya.

Banyak referensi DFIR melabeli `0064` sebagai instalasi pertama dan `0065` sebagai instalasi. Juntei mengikuti
definisi `devpkey.h` milik Microsoft di atas, dan menamai kolom sesuai dengan apa yang dicatat oleh setiap waktu.

Berikan hive `SYSTEM` ke `-f` atau `-d`. Hive `SOFTWARE` milik sistem dan hive `NTUSER.DAT`
milik penggunanya ditemukan di sebelahnya, sebagaimana hasil pengumpulan menyimpannya (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), atau di direktori yang sama dan satu tingkat di bawahnya bila hive-hive tersebut
diekspor bersama-sama. Kumpulkan semuanya beserta log transaksinya.

## Cara record digabungkan { #how-the-records-are-joined }

- **Perangkat:** satu record per key perangkat: `usbstor`, `usb`, dan `wpdbusenum` (`DeviceClass`).
  Perangkat penyimpanan USB mengambil vendor ID dan product ID-nya dari perangkat USB dengan
  container ID yang sama (atau jika tidak ada, nomor seri yang sama).
- **Volume:** volume milik perangkat penyimpanan USB adalah nilai `\??\Volume{GUID}` di
  `MountedDevices` yang datanya berupa path yang menyebutkan perangkat tersebut. Perangkat dengan dua volume memiliki dua
  record. Huruf drive-nya adalah nilai `\DosDevices\X:` dengan data yang sama.
- **Nama dan nomor seri:** nama volume berasal dari `Windows Portable Devices` dan dari
  perangkat portabel yang menyebutkan perangkat yang sama. Label volume dan nomor seri berasal dari
  key `EMDMgmt` yang menyebutkannya.
- **Pengguna:** pengguna yang `MountPoints2`-nya memiliki GUID volume tersebut, dengan waktu last-write key tersebut.
- **Volume tanpa key perangkat** (`volume`): volume `MountedDevices` yang tidak disebutkan oleh key perangkat mana pun (milik
  disk internal, atau milik perangkat yang dilepas yang key `Enum`-nya sudah hilang), huruf drive yang datanya tidak sama dengan
  nilai GUID volume mana pun, dan volume yang hanya dimiliki oleh `MountPoints2` pengguna.
- **Perangkat yang hanya disebutkan oleh `SOFTWARE`** (`portable_device`, `readyboost`).

`SourceKeys` mencantumkan setiap key dan nilai yang menjadi sumber penggabungan sebuah record, beserta hive-nya.

## Peringatan bagi analis { #analyst-cautions }

- **Huruf drive digunakan kembali.** `MountedDevices` menyimpan perangkat terakhir yang diberi setiap huruf. Sebuah
  huruf hanya ditampilkan pada perangkat yang datanya masih merupakan data huruf tersebut.
- **Nomor seri dapat dibuat-buat.** Ketika karakter kedua dari sebuah instance adalah `&`, Windows
  membuatnya sendiri karena perangkat tidak melaporkan nomor seri; dalam hal itu `Serial` kosong. Instance
  seperti itu dapat berbeda di komputer lain.
- **Instance penyimpanan USB diakhiri dengan logical unit** (`&0`). `Serial` tidak menyertakannya; `Instance`
  menyimpannya.
- **Waktu berbeda menurut versi Windows.** Windows 7 hanya memiliki waktu instalasi. `LastArrivalUTC` dan
  `LastRemovalUTC` hanyalah koneksi terakhir, bukan setiap koneksi. Log peristiwa
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) dan
  `setupapi.dev.log` memberikan lebih banyak informasi.
- **Control set:** secara default, control set yang terakhir digunakan Windows yang dibaca. `--all-control-sets`
  membaca setiap control set; control set yang lebih lama masih dapat mencantumkan perangkat yang sudah tidak ada di control set saat ini.
- **Waktu last-write sebuah key bukanlah waktu koneksi.** Windows menulis ulang key perangkat karena banyak
  alasan.
- **Perangkat yang dihapus meninggalkan jejak.** Record volume dengan vendor dan produk tetapi tanpa key perangkat
  adalah perangkat yang key `Enum`-nya telah dihapus (oleh proses uninstall atau alat pembersih) tetapi volumenya
  masih dimiliki oleh `MountedDevices`.
- **Nomor seri ReadyBoost** adalah nomor seri volume (`VolumeSerials`, sebagaimana ditampilkan `dir`),
  yang juga dicatat oleh berkas LNK dan Jump Lists.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Ringkasan mencantumkan perangkat penyimpanan USB yang terakhir tersambung (terbaru terlebih dahulu), beserta huruf
drive-nya, dan berapa banyak volume milik perangkat penyimpanan USB yang tidak memiliki key perangkat.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host` | Nama komputer, dari `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum`, atau, tanpa key perangkat, `volume`, `portable_device`, atau `readyboost`. |
| `ControlSet` | Control set dari key perangkat. |
| `Vendor`, `Product`, `Revision` | Penyimpanan USB: dari nama perangkat (`Disk&Ven_…&Prod_…&Rev_…`), juga untuk perangkat yang disebutkan oleh volume atau key `SOFTWARE`. |
| `Vid`, `PID` | Vendor ID dan product ID USB. |
| `Serial`, `Instance` | Nomor seri, dan nama key instance sebagaimana tertulis. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Nilai-nilai key perangkat, sebagaimana tertulis (`DeviceDesc` sering kali berupa `@<inf>,%…%;<text>`). `FriendlyName` milik perangkat portabel biasanya adalah label atau drive volumenya. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Waktu-waktu properti perangkat (lihat di atas). |
| `DriveLetters` | Huruf drive yang data `MountedDevices`-nya adalah data volume tersebut. |
| `VolumeGUID`, `VolumeData` | GUID volume, dan apa yang dinyatakan oleh data `MountedDevices`-nya tentang lokasinya: path perangkat, disk signature MBR dan offset partisi, atau partisi GPT. |
| `VolumeNames`, `VolumeSerials` | Nama yang diberikan Explorer kepada perangkat atau volumenya, label ReadyBoost, dan nomor seri volume (`1234-ABCD`). |
| `User` | Pengguna yang `MountPoints2`-nya memiliki volume tersebut, beserta last write key tersebut (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Tempat record ditemukan (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`), dan setiap key dan nilai yang menjadi sumber penggabungannya. |
| `KeyPath`, `KeyLastWrittenUTC` | Key instance perangkat dan last write-nya. |
| `SourcePath`, `SoftwarePath` | Hive `SYSTEM`, dan hive `SOFTWARE` bila hive tersebut memberikan sesuatu. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry dari hive-hive yang menjadi sumber penggabungan record. |
| `Problems` | Key yang tidak dapat dibaca, dan properti perangkat yang bukan FILETIME (beserta byte-nya). |

## Strategi investigasi { #investigation-strategy }

1. Urutkan perangkat penyimpanan USB berdasarkan `LastArrivalUTC` (atau jika tidak ada, `DriverLastInstalledUTC`), dan sisihkan
   pembaca kartu dan hub milik komputer itu sendiri.
2. Catat vendor, produk, nomor seri, GUID volume, huruf drive, dan nomor seri volume
   setiap perangkat, lalu cari di berkas LNK, Jump Lists, ShellBags, dan `$MFT` (path pada huruf drive
   atau volume tersebut).
3. Periksa `User` untuk mengetahui siapa yang pernah me-mount volume tersebut, dan kapan (last write key `MountPoints2`).
4. Cari record `volume` yang menyebutkan perangkat penyimpanan USB: perangkat yang key-nya telah dihapus.
5. Korelasikan dengan log peristiwa untuk setiap koneksi, bukan hanya yang terakhir.
6. Gunakan `parse-registry read --recover-deleted` untuk key perangkat yang dihapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: Properti perangkat (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Mendukung permintaan mount manager](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (sebagaimana terdapat di header Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): nomor-nomor properti perangkat
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md), dan
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Penggabungan pengguna menggunakan identitas sumber setiap hive, sehingga hive yang diekspor dengan nama profil yang sama tetap terpisah. `SourceKeys` mempertahankan path hive pengguna yang berkontribusi. Penemuan pengguna pendamping melewati symbolic link, konsisten dengan penemuan hive secara rekursif. Korelasi VID/PID memerlukan container ID yang cocok dan dapat digunakan; tanpanya, pencocokan nomor seri harus menghasilkan ID yang tidak ambigu. Kandidat yang saling bertentangan tetap tidak diketahui dengan sebuah problem.
