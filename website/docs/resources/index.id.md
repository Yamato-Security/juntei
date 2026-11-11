# Artikel dan referensi { #articles-and-references }

Halaman ini mengumpulkan titik awal berkualitas tinggi bagi pembaca yang ingin melangkah lebih jauh dari panduan
artefak. Bagian **Detail lebih lanjut** pada setiap artefak berisi tautan tingkat rendah yang paling spesifik.

## Orientasi DFIR untuk pemula { #beginner-dfir-orientation }

- [Poster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — peta visual artefak Windows yang umum beserta lokasinya.
- [Poster SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — pivot perburuan praktis dan pengingat untuk investigasi Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — sampel artefak publik untuk pembelajaran dan validasi parser.
- [Referensi artefak Velociraptor](https://docs.velociraptor.app/artifact_references/) — deskripsi berorientasi pengumpulan dan implementasi VQL untuk banyak artefak Windows.
- [Dokumentasi Plaso](https://plaso.readthedocs.io/) — konsep timeline dan parser untuk analisis lintas artefak.

## Artefak sistem berkas { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) dan [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — deskriptor keamanan, ACE, dan ekspresi kondisional.
- [Dokumentasi format libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Parser mft oleh Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — parser NTFS dengan carving catatan INDX dan index slack.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — melakukan carving slack `$I30` untuk mendapatkan nama berkas.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — riset dan penguraian log transaksi NTFS secara mendetail.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — mengorelasikan riwayat `$LogFile`, `$MFT`, dan USN.

## Artefak eksekusi dan inventaris { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — kajian format/pustaka Prefetch.
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## Artefak aktivitas pengguna { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registry dan persistensi { #registry-and-persistence }

- [Microsoft: Tipe nilai registry](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Spesifikasi format registry Windows oleh Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) dan [libregf](https://github.com/libyal/libregf) — referensi penguraian dan pemulihan hive tingkat rendah.
- [Microsoft: Pohon registry Services](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Skema Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender dan penanganan malware { #defender-and-malware-handling }

- [Microsoft: Memulihkan berkas yang dikarantina di Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Plugin Defender dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — implementasi riset/ekstraksi tersendiri.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — sumber daya penanganan dan analisis.

## Proyek parser Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Materi eksternal dapat berubah"
    Verifikasi klaim terhadap versi Windows yang terdampak dan data yang Anda kumpulkan. Artikel yang berguna dapat
    mendeskripsikan generasi sistem operasi, versi parser, atau eksperimen tertentu, bukan aturan
    yang berlaku universal.
