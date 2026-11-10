# Karantina Microsoft Defender { #microsoft-defender-quarantine }

## Apa isi karantina { #what-the-quarantine-contains }

Karantina Microsoft Defender menyimpan metadata tentang suatu deteksi dan konten terenkripsi dari
resource yang dikarantinanya. Juntei mengaitkan catatan Entries, ResourceData, dan Resources untuk melaporkan
label ancaman, waktu karantina, path logis dan fisik asli, metadata berkas, pemilik,
`Zone.Identifier` asal unduhan, hash, dan masalah integritas.

Artefak ini dapat bertahan lebih lama daripada log peristiwa yang semula mencatat deteksi/remediasi. Artefak ini juga
dapat mempertahankan byte mencurigakan yang persis untuk rekayasa balik di lingkungan terisolasi.

<span class="evidence-strength">Kesimpulan terbaik:</span> Defender membuat catatan karantina untuk
ancaman/resource yang disebutkan pada waktu yang tercatat. Label produk bukan bukti independen adanya malware,
dan karantina saja tidak membuktikan adanya eksekusi sebelumnya.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Kumpulkan seluruh direktori `Quarantine`. Entri tanpa berkas konten/catatan yang cocok tetap merupakan
metadata yang berguna, tetapi tidak dapat menghasilkan byte berkas atau hash yang dihitung.

## Keamanan dan peringatan bagi analis { #safety-and-analyst-cautions }

- Konten yang diekstrak kemungkinan besar berbahaya. Gunakan sistem analisis yang terisolasi dan jangan pernah mengeksekusinya.
- `ThreatName` adalah signature/klasifikasi Defender pada saat itu. False positive dan
  reklasifikasi di kemudian hari dapat terjadi.
- Waktu karantina adalah waktu remediasi/karantina—belum tentu waktu unduhan, pembuatan, atau eksekusi.
- Berkas yang dikarantina dapat terdeteksi sebelum dieksekusi atau setelah berjalan.
- Path asli dan Zone.Identifier bernilai tinggi tetapi bisa tidak ada atau dikendalikan penyerang.
- Catatan yatim (orphan) dapat disebabkan oleh retensi/pembersihan, pengumpulan yang tidak lengkap, atau kerusakan.
- Juntei memverifikasi ukuran, CRC, ID, dan hash; tinjau `Problems` sebelum memercayai byte yang dipulihkan.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Sebagai contoh, `tool.exe` menjadi `tool.exe.quarantined`. Stream tambahan menyertakan tipenya di
dalam nama, seperti `tool.exe.Zone.Identifier.quarantined`. Juntei juga menghapus izin eksekusi.
Ekstensi dan izin tersebut mengurangi risiko eksekusi yang tidak disengaja, tetapi byte yang dipulihkan tetap harus
diperlakukan sebagai malware.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Record` | `resource`, entri tanpa resource, `orphan-content`, atau `orphan-record`. Catatan yatim menunjukkan keterkaitan/pengumpulan yang tidak lengkap. |
| `QuarantineTime` | Waktu karantina/remediasi Defender dalam UTC, bukan waktu eksekusi. |
| `ThreatName`, `ThreatID` | Klasifikasi Defender dan ID numerik. Perlakukan sebagai putusan/konteks dari vendor. |
| `ResourceType` | Kategori resource berupa berkas, kunci registry, layanan, item startup, tugas terjadwal, atau lainnya. |
| `Path`, `PhysicalPath` | Lokasi logis/fisik asli. Pivot korelasi utama; pertahankan ejaan persisnya. |
| `FileSize` | Ukuran yang dicatat oleh metadata resource. Bandingkan dengan `ContentSize` dan MFT. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Metadata yang direkam untuk resource yang dikarantina; terpisah dari waktu karantina dan rentan terhadap timestomping. |
| `Attr` | Atribut berkas yang tercatat. Status hidden/system dapat menjadi konteks yang berguna. |
| `Owner` | SID dari security descriptor yang dipulihkan; pemilik belum tentu pengguna yang mengeksekusi. |
| `ZoneID`, `ReferrerURL`, `HostURL` | Nilai zona unduhan dan URL dari `Zone.Identifier`. Petunjuk asal yang kuat bila ada; ADS bisa tidak ada/dihapus. |
| `ContentSize` | Ukuran sebenarnya dari konten utama yang dipulihkan. Bandingkan dengan `FileSize` yang tercatat. |
| `MD5`, `SHA1`, `SHA256` | Dihitung dari byte yang dipulihkan. Gunakan SHA-256 untuk identifikasi konten modern; simpan semuanya demi interoperabilitas. |
| `Streams` | Jenis/nama/ukuran stream yang dipulihkan; dapat mencakup ADS, data keamanan, atau hive registry. |
| `ResourceID` | Pengenal yang menautkan catatan metadata/konten resource. |
| `EntryID` | GUID entri karantina; juga menautkan ke bukti riwayat/peristiwa Defender. |
| `ScanID` | Pengenal pemindaian Defender untuk mengelompokkan resource/tindakan. |
| `QuarantineFolder` | Folder sumber root/pembeda host. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Berkas sumber persis yang digunakan untuk membangun baris. Penting untuk validasi. |
| `ExtractedFile` | Berkas utama yang dikarantina dan ditulis oleh `--extract`; kosong bila tidak ada ekstraksi utama. Stream tambahan ditulis di sampingnya. `auto-parse -x` memberikannya relatif terhadap direktori keluarannya (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Ketidaksesuaian integritas, konten yang hilang, ketidakcocokan ID, kerusakan, atau konteks yatim. Jangan disembunyikan. |

JSON/JSONL mempertahankan resource terstruktur, daftar stream, entri catatan, dan semua detail
hubungan, alih-alih teks gabungan seperti di CSV.

## Perilaku ekstraksi { #extraction-behavior }

Juntei menggunakan nama berkas asli dengan ekstensi akhir `.quarantined` bila entri karantina
mencatatnya. Juntei menggunakan pengenal konten untuk resource yatim atau resource non-berkas, dan menambahkan
pengenal tersebut untuk mengatasi tabrakan nama berkas. Karakter nama berkas yang tidak valid di Windows diganti. Stream keamanan
dan stream alternatif ditulis di samping berkas utama. Setiap stream keamanan juga mendapatkan laporan teks
yang mudah dibaca. Juntei menghapus izin eksekusi;
konten berbeda yang sudah ada tidak ditimpa tanpa `-C`. Kontrol-kontrol ini mengurangi kecelakaan tetapi tidak
membuat byte tersebut aman.

### Membaca security descriptor yang dipulihkan { #reading-the-recovered-security-descriptor }

Berkas `.security.quarantined` berisi metadata keamanan Windows biner, sehingga tidak dapat dibaca di
editor teks. Berkas ini adalah stream cadangan tersendiri, bukan salinan lain dari executable. Juntei secara otomatis
mendekodenya saat mengekstrak, baik dengan `parse-file defender-quarantine --extract` maupun `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

Nama berkas teks mengikuti nama biner yang sebenarnya, termasuk pengenal tabrakan bila ada. Teks yang dihasilkan
adalah UTF-8 biasa dan menggunakan perlindungan penimpaan yang sama seperti berkas hasil ekstraksi lainnya. Teks tersebut mencakup
nama berkas biner, jumlah byte, dan SHA-256 sehingga laporan dapat dicocokkan dengan sumbernya.

| Field laporan | Cara menginterpretasikannya |
|---|---|
| Pemilik dan grup primer | SID yang tercatat di dalam descriptor. Well-known SID diberi nama seperti `BUILTIN\Administrators` atau `NT AUTHORITY\SYSTEM`. SID lain tetap tidak diresolusi; RID yang familier saja tidak cukup untuk menamai akun lokal/domain. Kepemilikan tidak membuktikan siapa yang membuat atau menjalankan berkas. |
| Control flag | Flag numerik asli beserta namanya, termasuk keberadaan ACL, pewarisan, dan proteksi. |
| DACL | Entri kontrol akses (ACE) yang berurutan: tipe allow/deny, SID akun, mask izin, hak, dan flag pewarisan. Ini adalah entri yang tercatat, bukan perhitungan akses efektif untuk token pengguna. |
| SACL | Entri audit atau mandatory-integrity yang tercatat. Flag audit-success/failure mendeskripsikan konfigurasi, bukan riwayat akses. SACL yang tidak ada tidak membuktikan bahwa semua audit dinonaktifkan. |
| Object GUID | GUID opsional untuk ACE khusus objek, dengan hak khusus objek dipertahankan dalam bentuk numerik alih-alih diberi label keliru sebagai hak berkas. |
| SDDL | Representasi Security Descriptor Definition Language dari pemilik, grup, dan entri ACL yang didukung. Periksa control flag secara terpisah. Juntei tidak mengeluarkan string SDDL parsial jika ada komponen yang rusak atau tidak didukung. |
| Status dekode dan masalah | `complete` atau `incomplete`, beserta alasannya. ACE yang tidak didukung mempertahankan hex mentahnya di dalam laporan. Biner asli selalu dipertahankan bila ekstraksi berhasil. Masalah dekode juga muncul di field `Problems` catatan karantina. |

DACL yang tidak ada, NULL DACL, dan DACL kosong dilaporkan secara terpisah. DACL yang tidak ada atau NULL tidak memberikan
pembatasan akses diskresioner; DACL kosong tidak memiliki ACE yang memberikan akses. Pemeriksaan akses
dan hak istimewa Windows lainnya tetap berpengaruh. ACE yang diwarisi dan ACE inherit-only tetap ditandai secara eksplisit.

Sebagai contoh, entri yang memberikan full control yang diwarisi kepada Administrators dapat muncul sebagai:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Dekoder berjalan secara offline di Windows, macOS, dan Linux tanpa Windows API, Python, atau pencarian akun.
Dekoder ini mendekode ACE dasar allow/deny/audit/alarm, ACE objek, dan label mandatory-integrity. ACE conditional,
callback, resource-attribute, dan ACE lain yang tidak didukung diidentifikasi secara eksplisit dan dipertahankan dalam
bentuk mentah; kondisi aksesnya tidak dievaluasi. Descriptor yang rusak tidak menghentikan ekstraksi
stream lainnya.

## Strategi investigasi { #investigation-strategy }

1. Mulailah dari `ThreatName`, `Path`, `QuarantineTime`, `EntryId`, dan hash.
2. Validasi `Problems` dan bandingkan ukuran yang tercatat dengan konten yang dipulihkan.
3. Lakukan pivot SHA-256 ke host lain, EDR, sandbox, dan data threat intelligence.
4. Korelasikan path/waktu asli dengan `$MFT`, Jurnal USN, Prefetch, Amcache, dan log peristiwa.
5. Gunakan field URL/zona untuk menginvestigasi akses awal, tetapi validasi dengan bukti browser/proxy/DNS.

## Detail lebih lanjut { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Plugin Defender dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender Antivirus remediation and quarantine](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE strings](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
