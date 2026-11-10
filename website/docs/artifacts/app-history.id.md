# Riwayat aplikasi { #application-history }

## Jejak yang ditinggalkan alat { #what-tools-leave-behind }

Banyak alat pihak ketiga menyimpan pengaturan dan riwayatnya di `NTUSER.DAT` milik pengguna. Beberapa di antaranya
termasuk alat yang digunakan penyerang, atau yang digunakan untuk menyiapkan data sebelum mengambilnya: alat Sysinternals, pengarsip,
klien sinkronisasi cloud, dan alat akses jarak jauh. Juntei membaca hal-hal berikut:

| Aplikasi | Key di bawah `HKCU\Software` | Yang ditunjukkan |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Setiap alat yang lisensinya telah diterima pengguna (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Arsip yang dibuka; nama arsip dan folder ekstraksi yang diketik di dialognya |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Arsip yang dibuat; folder ekstraksi; folder yang dijelajahi dan menjadi tujuan penyalinan di file manager-nya |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Arsip di menu File, folder ekstraksi, arsip terbaru |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Alamat e-mail, folder lokal, dan nama setiap akun; library yang disinkronkan (`ScopeIdToMountPointPathCache`) dan folder (`Tenants\<tenant>`) |
| Adobe Acrobat dan Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Berkas PDF dan folder yang terakhir dibuka, beserta nama berkas, ukuran, dan tanggal |
| TeamViewer | `TeamViewer` (dan `Version<N>` milik versi lama) | Setiap nilai: nama akun dan nama pengguna, folder awal transfer berkas (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Setiap nilai: apa yang diatur untuk dibersihkan, `UpdateKey`, … |

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna ini menjalankan alat tersebut (atau menerima
lisensinya), atau alat tersebut membuka, membuat, atau mengekstrak arsip dan folder ini, atau akun cloud ini
disiapkan untuk pengguna tersebut. Sebagian besar riwayat tidak memiliki waktu per entri: hanya entri terbaru yang mendapatkan
waktu last-write dari key daftarnya.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Kumpulkan `NTUSER.DAT` setiap pengguna beserta log transaksinya, serta `SOFTWARE` milik sistem untuk
SID pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Penerimaan EULA Sysinternals.** `FirstUsedUTC` adalah perkiraan penggunaan pertama dari waktu last-write
  key ketika satu-satunya nilainya adalah `EulaAccepted` yang dapat dibaca dan bernilai `1`. EULA yang ditolak atau
  rusak tidak mendapatkan perkiraan ini. Pengaturan registry juga dapat dikonfigurasi sebelumnya,
  disalin, atau diedit: dukung eksekusi dan waktunya dengan artefak lain.
  Alat yang menyimpan pengaturan di sana (Process Explorer, Autoruns, …) menulis ulang key tersebut di kemudian hari:
  `FirstUsedUTC` kemudian kosong dan `KeyLastWrittenUTC` hanya menunjukkan perubahan pengaturan terakhir. Alat yang
  dijalankan dengan `-accepteula`, atau sebagai pengguna lain (sisi jarak jauh PsExec berjalan sebagai SYSTEM), menulis
  key milik pengguna tersebut.
- **Posisi, bukan waktu.** Entri daftar berurutan (`MruPosition` 0 adalah yang terbaru), tetapi
  hanya last write dari key daftar yang memberi tanggal pada entri terbaru. Entri yang lebih lama digunakan lebih awal, pada
  waktu yang tidak diketahui.
- **Nama yang diketik.** `ArcName` milik WinRAR adalah nama arsip yang diketik saat membuat arsip: sering kali nama
  berkas tanpa folder.
- **Path Adobe** disimpan sebagaimana ditulis oleh Adobe (`/C/Users/alice/a.pdf` adalah
  `C:\Users\alice\a.pdf`). `sDate`-nya (di `Details`) adalah string tanggal PDF, disimpan sebagaimana tertulis.
- **`UpdateKey` milik CCleaner** adalah waktu lokal tanpa zona waktu; disimpan sebagaimana tertulis.
- **TeamViewer.** Log koneksi berupa berkas (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  bukan nilai registry: kumpulkan juga berkas-berkas tersebut.
- **Ketiadaan.** Pengguna dan alat dapat menghapus riwayat ini (CCleaner sendiri dapat melakukannya).

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Ringkasan menampilkan perkiraan penggunaan pertama Sysinternals yang terbaru, arsip dan folder yang terakhir digunakan
oleh setiap daftar, serta akun OneDrive.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Host dan `NTUSER.DAT`. |
| `User`, `UserSID` | Nama folder profil, dan SID-nya dari `ProfileList` milik sistem. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Alat Sysinternals, daftar (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), akun OneDrive atau key folder yang disinkronkannya, atau nama nilai. |
| `MRU-Position` | Posisi entri dalam daftarnya, 0 adalah yang terbaru. |
| `Path` | Arsip, folder, atau dokumen. |
| `Value` | `EulaAccepted`, alamat e-mail akun OneDrive atau id library yang disinkronkan, nilai TeamViewer atau CCleaner. |
| `FirstUsedUTC` | Perkiraan penggunaan pertama dari penerimaan EULA (lihat peringatan di atas). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … milik akun OneDrive; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` milik dokumen Adobe. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Asal record, dan kapan key tersebut terakhir berubah. |
| `DataRaw` | Byte nilai (hex), bila tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Data yang tidak dapat didekode atau dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Cari alat Sysinternals yang tidak dijalankan oleh pengguna biasa (PsExec, ProcDump, PsLoggedOn,
   SDelete) beserta timestamp EULA-nya.
2. Tinjau nama arsip dan folder ekstraksi di sekitar waktu insiden: arsip yang dibuat di
   `ProgramData`, `Temp`, atau folder pengguna sebelum data keluar dari sistem.
3. Periksa akun OneDrive yang bukan milik organisasi, serta nama akun TeamViewer dan folder transfer
   berkasnya.
4. Korelasikan dengan Prefetch, Amcache, BAM, dan UserAssist (alat yang berjalan), `$MFT` dan USN
   Journal (berkas arsip), berkas LNK dan jump list (dokumen yang dibuka), serta log peristiwa.

## Detail lebih lanjut { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
