# Riwayat Deteksi Microsoft Defender { #microsoft-defender-detection-history }

## Apa yang dicatat riwayat { #what-the-history-records }

Microsoft Defender menulis satu berkas riwayat deteksi untuk setiap deteksi. Setiap berkas mencatat:

- pengenal, nama, tingkat keparahan, dan kategori ancaman;
- apa yang dilakukan Defender dan apakah tindakan itu berhasil;
- apakah ancaman sedang berjalan;
- pengguna dan proses yang terlibat, serta apa yang mendeteksinya;
- entri karantina, jika ada;
- setiap resource tempat ancaman ditemukan: berkas, anggota arsip, unduhan, proses, nilai registry,
  layanan, baris perintah, dan perilaku.

Defender juga menulis berkas ini untuk ancaman yang tidak pernah dikarantinanya: ancaman yang hanya dideteksinya,
ancaman yang diizinkan, dan ancaman yang gagal dihapusnya. Riwayat perlindungan Windows Security dan kelas WMI
`MSFT_MpThreatDetection` (`Get-MpThreatDetection`) menampilkan catatan ini. Ketika Orange
Cyberdefense mengubah nilai dalam sebuah berkas, WMI melaporkan nilai yang telah diubah.

<span class="evidence-strength">Kesimpulan terbaik:</span> Defender mencatat deteksi ancaman yang disebutkan
pada resource yang tercantum, dengan status dan waktu yang tercatat. Nama ancaman adalah putusan
produk. Catatan deteksi saja tidak menunjukkan bahwa ancaman tersebut berjalan, atau bahwa ancaman itu
dihapus sepenuhnya.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Setiap berkas dinamai sesuai `DetectionID`-nya. Kumpulkan seluruh folder `DetectionHistory`. Masukan
direktori dan `auto-parse` membaca setiap berkas di dalam folder `DetectionHistory`, pada kedalaman berapa pun,
yang awalnya tampak seperti berkas riwayat deteksi. Gunakan `-f` untuk salinan yang disimpan di luar folder semacam
itu. Defender menghapus item dari riwayat pemindaiannya setelah sejumlah hari yang dapat dikonfigurasi
(`ScanPurgeItemsAfterDelay` di `Set-MpPreference`), sehingga deteksi yang lebih lama mungkin sudah hilang.

Karantina adalah artefak terpisah: lihat [Karantina Defender](defender-quarantine.md). `QuarantineID` suatu
deteksi memakai ejaan yang sama dengan `EntryID` pada keluaran karantina (huruf besar, tanpa kurung
kurawal), sehingga keduanya dapat digabungkan.

## Peringatan bagi analis { #analyst-cautions }

- Deteksi, upaya remediasi, dan penghapusan yang berhasil adalah keadaan yang berbeda. Baca
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) bersama
  `ThreatStatusErrorCode`, yang bernilai 0 bila perubahan status terakhir berhasil.
- `ExecutionStatus` adalah pandangan Defender pada saat deteksi (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Ini bukan bukti bahwa ancaman berjalan, maupun bukti bahwa ancaman tidak pernah
  berjalan. Cocokkan dengan Prefetch, Amcache, log peristiwa, dan telemetri proses.
- `ThreatName` adalah nama signature pada saat itu. False positive dan reklasifikasi di kemudian hari
  dapat terjadi. Berkas uji seperti EICAR dilaporkan seperti ancaman sungguhan.
- `User` adalah nilai `DomainUser` yang dicatat Defender (`UserSource` adalah `DomainUser`). Nilainya bisa
  kosong, akun layanan, atau pengguna yang berbeda dari pengguna yang membuat berkas tersebut.
- Defender dapat menulis beberapa berkas untuk satu ancaman, dan Juntei menyimpan semuanya. Dua berkas bahkan
  dapat memuat `DetectionID` yang sama, seperti pada berkas yang disalin atau dipulihkan. Kelompokkan
  berdasarkan `DetectionID` dan `SourceFile` sekaligus.
- Nilai yang maknanya tidak diketahui tetap disimpan, tidak ditafsirkan: bagian dan posisinya ada di
  `UnknownValues`.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV adalah format berkas bawaan. Dengan `-t csv`, `-t json`, atau `-t jsonl` tanpa `-o`, catatan ditulis ke
stdout. Tanpa kedua opsi tersebut, hanya ringkasan yang ditampilkan. Tambahkan `-C` untuk menimpa keluaran
yang sudah ada. Keluaran tidak pernah ditulis menimpa salah satu berkas masukan, bahkan dengan `-C`.

Juntei menulis satu catatan per resource dari setiap deteksi. Deteksi tanpa resource mendapat satu catatan
dengan field resource yang kosong. Catatan-catatan dari satu deteksi berbagi `DetectionID` yang sama dan
diberi nomor dengan `ResourceIndex` sesuai urutan penyimpanannya.

Kerusakan tidak menghentikan proses. Berkas yang terpotong tetap mempertahankan nilai yang dibaca sebelum
kerusakan, dan `Problems` menyebutkan apa yang hilang. Berkas di dalam folder `DetectionHistory` yang bukan
berkas riwayat deteksi dilaporkan dan dilewati. Berkas lainnya tetap dibaca.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Record` | `resource`, atau `detection` untuk deteksi tanpa resource. |
| `InitialDetectionTime` | Saat Defender pertama kali mendeteksi ancaman (UTC). Kosong bila berkas memuat 0; nilai yang bukan waktu disimpan di `UnknownValues`. |
| `LastThreatStatusChangeTime` | Saat status ancaman terakhir kali berubah (UTC). |
| `RemediationTime` | Saat Defender meremediasi ancaman (UTC). Ini adalah waktu suatu upaya, bukan bukti penghapusan; periksa `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Nama signature dan pengenal numerik Defender (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe`, atau `Unknown`, beserta angka yang tersimpan. |
| `Category`, `CategoryID` | Kategori ancaman seperti `TROJAN`, `TOOL`, atau `BEHAVIOR`, beserta angka yang tersimpan. |
| `ThreatStatus`, `ThreatStatusID` | Status ancaman menurut Defender. Angka tanpa nama yang terdokumentasi disimpan dengan `ThreatStatus` kosong. |
| `ThreatStatusErrorCode` | Kode kesalahan dari perubahan status terakhir; 0 berarti perubahan itu berhasil. |
| `ExecutionStatus`, `ExecutionStatusID` | Apakah Defender melihat ancaman sedang berjalan saat terdeteksi. |
| `DetectionSource`, `DetectionSourceTypeID` | Apa yang mendeteksinya: `Real-time`, `IOAV` (unduhan dan lampiran), `User`, `System`, `AMSI`, dan lainnya. |
| `User`, `UserSource` | Nilai `DomainUser` yang dicatat Defender, dan dasarnya (`DomainUser`). Kosong bila berkas tidak menyebutkan pengguna. |
| `Process` | Proses yang terlibat (`ProcessName`); bisa berupa `Unknown`. |
| `ResourceIndex`, `ResourceCount` | Posisi resource ini, dan jumlah resource dalam deteksi. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi`, dan lainnya, sesuai penamaan Defender. |
| `Path` | Resource tersebut: path, anggota arsip (`archive.zip->member`), nilai registry, baris perintah, atau proses. `webfile` juga menyebutkan URL-nya dan proses yang mengunduhnya (`pid:…,ProcessStart:…`). Pertahankan ejaan persisnya. |
| `ResourceFlags` | Flag resource dalam heksadesimal; maknanya tidak terdokumentasi. |
| `TrackingStartTime` | `ThreatTrackingStartTime` milik resource, bilangan bulat yang dibaca sebagai FILETIME (UTC), sama seperti cara plaso menentukan waktu deteksi. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Hash dan ukuran dari data pelacakan resource. Nilai ini menggambarkan berkas pada saat Defender memindainya. |
| `TrackingID` | `ThreatTrackingId` milik resource. |
| `OtherTracking` | Setiap nilai pelacakan lainnya, sebagai `Key=Value` sesuai urutan penyimpanan, seperti `ThreatTrackingScanSource` atau `ThreatTrackingBMContextRich` untuk deteksi perilaku. |
| `QuarantineID` | Entri karantina (`Quarantine\Entries\{GUID}`); kosong bila berkas memuat GUID nol. |
| `DetectionID` | GUID deteksi; juga nama berkas. Ketidakcocokan dilaporkan dalam `Problems`. |
| `MagicVersion` | `Magic.Version:1.2` pada header; kosong pada berkas yang ditulis tanpanya, seperti sebagian berkas dari Windows Server 2016. |
| `UnknownValues` | Nilai yang maknanya tidak diketahui, sebagai `position=value` (misalnya `6.13=NT AUTHORITY\SYSTEM`), dengan daftar terhitung setelah status sebagai `2.list`. |
| `SourceFile` | Berkas yang dibaca, relatif terhadap direktori masukan, atau namanya bila memakai `-f`. |
| `Problems` | Kerusakan, nilai yang hilang, tipe yang tidak terduga, atau nama berkas yang tidak cocok. Jangan disembunyikan. |

JSON dan JSONL mempertahankan field yang sama dengan nilai bertipe: angka tetap angka dan nilai yang hilang
menjadi `null`. Sebagai ganti `OtherTracking`, keduanya memiliki `Tracking`, yaitu setiap nilai pelacakan
beserta key Defender, tipe, dan nilainya. `UnknownValues` menjadi daftar posisi beserta tipenya. `CountedList`
memuat daftar terhitung. FILETIME yang tersimpan mempertahankan jumlah tick mentahnya di samping waktu hasil
pembacaannya.

## Perilaku timeline { #timeline-behavior }

`auto-parse` menyimpan catatan sebagai `SecurityProducts/DefenderHistory.csv` (atau `.json`/`.jsonl`) dan
menambahkannya ke timeline gabungan `-j`. `super-timeline` membuat peristiwa dari
`InitialDetectionTime`, `LastThreatStatusChangeTime`, dan `RemediationTime` pada catatan setiap resource,
dengan status, kode kesalahan, status eksekusi, dan sumber deteksi di detail peristiwa.
`TrackingStartTime` disimpan sebagai bilangan bulat, sehingga hanya menjadi peristiwa dengan
`--include-interpreted-times`. Waktu milik berkas itu sendiri tidak pernah digunakan. Catatan tanpa waktu
yang dapat digunakan tetap berada di keluaran hasil penguraian dan dihitung sebagai catatan tanpa waktu.

## Strategi investigasi { #investigation-strategy }

1. Mulailah dari `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime`, dan `ThreatStatus`.
2. Periksa `ThreatStatusErrorCode` dan `Problems` sebelum menyimpulkan bahwa suatu ancaman telah dihapus.
3. Gabungkan `QuarantineID` dengan `EntryID` karantina untuk memulihkan byte yang dikarantina.
4. Lakukan pivot hash dan path ke `$MFT`, Jurnal USN, Prefetch, Amcache, riwayat browser, dan log
   peristiwa (log operasional Defender: ID peristiwa 1116, 1117, 1118, dan 1119).
5. Untuk resource `webfile`, gunakan URL dan waktu mulai proses untuk menginvestigasi akses awal, lalu
   validasi keduanya dengan bukti browser, proxy, dan DNS.

## Versi yang diuji { #tested-versions }

Pengujian menggunakan berkas publik yang hasilnya dibaca secara independen oleh alat lain:

- Sampel Windows 10 dari Orange Cyberdefense (Defender 4.18.24090.11), diperiksa terhadap nilai yang
  dilaporkan WMI.
- Dua berkas uji Windows milik plaso, diperiksa terhadap parser `windefender_history` milik plaso.
- 14 berkas dari VM simulator APT di DFIRArtifactMuseum.
- Salinan yang ditulis tanpa versi pada header, seperti sebagian berkas dari Windows Server 2016.
- Berkas yang terpotong, kosong, atau berjenis lain.

## Detail lebih lanjut { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [wddh-parser Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Parser riwayat Defender plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: kelas MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: ID peristiwa Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
