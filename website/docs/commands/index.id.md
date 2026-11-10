# Daftar Perintah { #command-list }

Parser-parser individual dikelompokkan berdasarkan tipe sumbernya. Jalankan `juntei --help`,
`juntei <SOURCE> --help`, atau `juntei <SOURCE> <ARTIFACT> --help` untuk menjelajahinya:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Untuk hasil pengumpulan bukti Windows dari banyak komputer, gunakan
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Perintah ini memanggil perintah-perintah berkelompok yang sama
dan mengatur hasil serta log berdasarkan komputer dan kategori investigasi.

Gunakan [`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md) untuk linimasa lintas artefak hasil parsing dengan arti waktu berlabel. Keluaran tidak diurutkan secara default; tambahkan `--sort` untuk urutan kronologis dan konsolidasi duplikat. `auto-parse -j` tetap menyimpan rekaman asli.

## [Image disk](mount.md) { #disk-images }

| Perintah | Input | Panduan terperinci |
|---|---|---|
| `mount` | Image disk VHDX / E01 / Ex01 | [Memasang Image Disk](mount.md) |

## [Perintah ESE](user-activity.md) { #ese-commands }

| Perintah | Artefak input | Panduan terperinci |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Pencatatan Akses Pengguna (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer dan Edge lama](../artifacts/webcache.md) |

## [Perintah berkas](execution.md) { #file-commands }

| Perintah | Artefak input | Panduan terperinci |
|---|---|---|
| `parse-file defender-quarantine` | Pohon atau entri karantina Defender | [Karantina Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [Pintasan LNK](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [Perintah NTFS](filesystem.md) { #ntfs-commands }

| Perintah | Artefak input | Panduan terperinci |
|---|---|---|
| `parse-ntfs boot` | `$Boot`, atau image volume NTFS | [Sektor boot ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), atau image disk | [Indeks direktori ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | `$LogFile` NTFS | [Log transaksi NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | `$MFT` NTFS | [Master File Table](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Deskriptor keamanan ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [Jurnal USN](../artifacts/usn-journal.md) |

## [Perintah registry](security-registry.md) { #registry-commands }

| Perintah | Artefak input | Panduan terperinci |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Jaringan](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [Rahasia LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (dan `BCD-Template`) | [Boot Configuration Data (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Kompatibilitas aplikasi](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Kebijakan audit](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps, dan item yang disematkan di taskbar](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Riwayat aplikasi](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Daftar MRU (Most-Recently-Used)](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (dengan `SAM`, `SOFTWARE` untuk nama pengguna) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` pada Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Program terinstal](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Dialog Buka dan Simpan Sebagai](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Autostart Registry](../artifacts/persistence.md) |
| `parse-registry read` | Hive registry utama apa pun | [Hive Registry](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (dengan `SOFTWARE` untuk SID pengguna) | [Akses jarak jauh (sisi pengguna)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Akun lokal SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Pengaturan keamanan](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Layanan Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` atau nilai AppCompatCache yang diekspor | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` dari satu sistem | [Informasi sistem](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Cache Penjadwal Tugas](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, dengan `SOFTWARE` dan `NTUSER.DAT` | [Perangkat USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [Perintah SQLite](sqlite.md) { #sqlite-commands }

| Perintah | Artefak input | Panduan terperinci |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Browser web](../artifacts/browser.md) |

## Tipe sumber dan perubahan format { #source-types-and-format-changes }

Kelompok-kelompok tersebut mendeskripsikan format yang saat ini didukung Juntei. `parse-registry amcache` membaca
`Amcache.hve`; hal ini tidak menyiratkan dukungan untuk database inventaris Amcache berbasis SQLite. Crate parser
artefak tetap terpisah sehingga dukungan untuk format penyimpanan lain dapat ditambahkan secara independen.

Nama-nama parser datar yang lama telah digantikan oleh path sumber-dan-artefak di atas. Tetap letakkan
opsi parser yang ada setelah nama artefak. `auto-parse` dan subperintah registry yang ada
tetap mempertahankan namanya.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` melakukan hunting di beberapa format sumber, sehingga merupakan perintah analisis
tingkat atas. Perintah ini melengkapi inventaris konfigurasi lengkap dari `parse-registry persistence` dengan
131 pemeriksaan, klasifikasi allow/block, metadata payload, cakupan, dan korelasi timestamp.

## Penggunaan tingkat atas { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Halaman-halaman sumber mereproduksi penggunaan perintah dan memberikan contoh singkat. Panduan artefak menjelaskan
bukti, field keluaran, keterbatasan, dan strategi investigasi.
