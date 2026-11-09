# Ucapan terima kasih { #acknowledgements }

Juntei ada karena komunitas DFIR dan open source telah memublikasikan parser, riset format,
keluaran referensi, spesifikasi, dan bukti uji. Proyek-proyek di bawah ini menjadi acuan bagi implementasi,
validasi, penamaan field, atau keluaran yang diharapkan analis. Tercantum di sini tidak berarti kode mereka
disematkan di dalam Juntei; periksa lisensi masing-masing proyek.

## Fondasi parser yang dikelola oleh Yamato Security { #parser-foundations-maintained-by-yamato-security }

| Area Juntei | Parser yang dikelola | Upstream/fondasi |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| Jurnal USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Indeks direktori (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`, modul `i30`) | Riset index slack yang disebutkan di bawah |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `boot` dan `secure`) | MS-DTYP dari Microsoft dan riset NTFS yang disebutkan di bawah |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) dan [ese](https://github.com/Sootmark/ese) |
| Browser web / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite), dan [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Karya compound file dan Jump List dari Sootmark shell |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Image disk VHDX (`mount`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) dan [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), disajikan melalui [fuser](https://github.com/cberner/fuser) | MS-VHDX dari Microsoft |
| Image disk E01 (`mount`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) ([ewf-image](https://github.com/ebrig/ewf-image) karya Elijah Brigham), di-mount dengan ys-vhdx-parser | Dokumentasi format dan alat [libewf](https://github.com/libyal/libewf) karya Joachim Metz, yang digunakan untuk memvalidasi pembaca |
| Registry | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Riset format dan pemulihan registry yang disebutkan di bawah |

## Alat referensi dan validasi independen { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), karya Hexastrike,
  menyediakan katalog pemeriksaan persistensi, allowlist bawaan, snapshot LOLBAS, dan perilaku
  referensi untuk perintah Rust native `persistence-hunter`. Lisensi MIT dan pemberitahuan hak ciptanya
  dipertahankan di `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`. Proyek asli
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) menginspirasi proyek Python tersebut;
  lihat [ucapan terima kasih persistensi](persistence-hunter/acknowledgements.md) untuk silsilah lengkapnya.

- **Spesifikasi regf** dan **yarp** karya Maxim Suhanov menjadi acuan bagi struktur registry, pemutaran ulang
  log transaksi, dan pemulihan record yang terhapus; **dfir_ntfs** karyanya menjadi referensi untuk record INDX dan
  carving index slack.
- **INDXRipper** karya Harel Segev menjadi referensi tambahan untuk carving nama berkas dari index slack.
- **dfir_ntfs**, **LogFileParser** karya Joakim Schicht, dan **ntfs-linker** dari Stroz Friedberg menjadi acuan bagi
  validasi dan interpretasi `$LogFile`; **go-ntfs** dari Velocidex menyediakan bukti uji NTFS publik.
- Proyek Joachim Metz / libyal **libregf**, **libfsntfs**, **libscca**, **libfwsi**, dan
  **libesedb** menyediakan riset format yang terperinci dan referensi parsing independen.
- **dissect.esedb**, **dissect.ntfs**, dan **dissect.target** dari Fox-IT menyediakan interpretasi independen atas SRUM, UAL,
  `$Secure:$SDS`, SAM, dan Defender yang digunakan selama validasi.
- `fsstat` dari **The Sleuth Kit** menjadi referensi untuk sektor boot NTFS pada volume yang dibuat dengan
  `mkntfs`.
- Beberapa alat karya Eric Zimmerman menetapkan nama field dan ekspektasi pemeriksa yang digunakan untuk perbandingan.
- **regipy**, **python-registry**, **Notatin**, dan **artemis** menyediakan implementasi registry,
  pengujian, atau contoh interpretasi.
- **LnkParse3**, **lnk_parser** dan **jumplist_parser** karya AbdulRhmanAlfaifi, serta JLECmd menyediakan
  perilaku LNK/Jump List independen dan kasus uji publik.
- **prefetch-forensic** dari Security Ronin dan proyek **forensicnomicon** menjadi acuan bagi observasi
  Prefetch/Amcache/ShimCache bernilai tinggi di Juntei.
- **defender-dump** dan dissect.target menyediakan implementasi terpisah untuk memahami struktur karantina
  Defender dan perilaku ekstraksi yang aman.
- **Plaso**, **Chainsaw**, dan **Velociraptor** menyediakan perbandingan parser, bukti uji, atau
  perilaku referensi yang berorientasi pengumpulan untuk berbagai artefak.
- **Hindsight** dari Obsidian Forensics dan definisi **SQLiteHunter** dari Velocidex menjadi referensi untuk
  riwayat, unduhan, cookie, dan riwayat formulir browser.

## Spesifikasi dan riset { #specifications-and-research }

Kami berterima kasih kepada Microsoft karena telah memublikasikan Open Specifications untuk Shell Links, Compound Binary Files,
protokol NTFS/SAM, struktur registry/layanan/tugas, dan dokumentasi API. Kami juga berterima kasih kepada para
penulis riset forensik AppCompatCache, Amcache, SRUM, Prefetch, registry, dan NTFS yang membuat
artefak-artefak ini dapat dipahami oleh para praktisi.

## Bukti uji { #test-evidence }

Bukti publik dari
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) karya Andrew Rathbun, NIST CFReDS, Plaso,
Regipy, Artemis, repositori uji parser, dan koleksi lain dengan lisensi yang sesuai memungkinkan pengujian regresi
terhadap data Windows nyata. Juntei menyimpan catatan atribusi/lisensi di samping berkas uji
yang disertakan.

## Kontributor { #contributors }

Terima kasih kepada setiap kontributor parser Juntei dan Yamato Security, peninjau, pelapor issue, dan
pengguna yang menyumbangkan kasus tepi. Kontributor saat ini tercantum oleh GitHub di
[halaman kontributor Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## Lisensi { #license }

Juntei dirilis di bawah
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) dan hak ciptanya dimiliki oleh Shirofune Security Co., Ltd. Kode sumbernya dipublikasikan di bawah
AGPLv3 sebelum versi 1.0.0. Lisensi crate serta kode dan data pihak ketiga lainnya yang digunakan untuk membangunnya
tercantum di
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
