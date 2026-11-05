# Format keluaran { #output-formats }

## Ringkasan terminal { #terminal-summaries }

Tanpa `-o` atau `-t` yang terstruktur, perintah umumnya menampilkan jumlah, rentang waktu, kerusakan, dan temuan
bernilai tinggi. Ringkasan membantu Anda memahami cakupan sebelum membuat keluaran yang besar.

## CSV { #csv }

CSV adalah format penyimpanan default. Format ini praktis untuk alat spreadsheet, impor SQL, dan penampil
timeline. Beberapa parser membuat beberapa tabel karena artefaknya berisi tipe record yang
berbeda (Amcache, Jump Lists, dan SRUM).

!!! warning "Perlakukan kolom bukti sebagai teks"
    Path atau command line dapat diawali dengan `=`, `+`, `-`, atau `@`. Perangkat lunak spreadsheet dapat menginterpretasikannya
    sebagai formula. Impor semua field sebagai teks atau gunakan penampil CSV yang aman untuk forensik. Juntei mempertahankan
    string bukti alih-alih mengubahnya.

## Nama field { #field-names }

Header kolom CSV dan nama field JSON/JSONL menggunakan PascalCase, termasuk Amcache dan
manifest `auto-parse`. Akronim tetap dalam huruf kapital: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`. Kata yang mengikuti akronim dipisahkan dengan tanda hubung: `IP-Address`, `MFT-Entry`,
`USN-Journal`. Dengan `jq`, beri tanda kutip pada field yang mengandung tanda hubung: `.["IP-Address"]`.

CSV MFT default menggunakan judul yang lebih pendek dan mudah dibaca seperti `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)`, dan `Created (FN)`. Lihat [referensi field MFT](../artifacts/mft.md)
untuk skema CSV lengkapnya. Pasangan timestamp mengikuti urutan MACB: DataModified, Accessed,
MetadataChanged, Created, dengan SI sebelum FN. Beri tanda kutip pada judul yang mengandung spasi atau tanda baca
di alat CSV.

Nilai bukti mempertahankan ejaan aslinya. Key dictionary yang berasal dari sumber, seperti nama nilai
registry di bawah `Details`/`DataRaw` dan nama `Zone.Identifier` di bawah `ZoneIdentifier`, juga
tetap tidak berubah. Dictionary `ValueState` dan `Raw` milik SRUM menggunakan label yang sama dengan kolom
datanya. Tabrakan normalisasi dilaporkan alih-alih menghilangkan field secara diam-diam.

Hal ini mengubah skema ekspor sebelumnya. Buat ulang hasil yang ada dan perbarui query yang tersimpan
(misalnya, `.record_type` menjadi `.RecordType`, dan `full_path` Amcache menjadi `FullPath`).
Laporan teks yang dapat dibaca manusia serta format input/konfigurasi mempertahankan konvensi yang sudah ada.

## Nama field yang disingkat { #abbreviated-field-names }

Label berikut berlaku untuk ekspor CSV, JSON, dan JSONL, termasuk penguraian otomatis dan
manifest. Perbarui filter `jq`, `xsv`, dan spreadsheet yang tersimpan saat membuat ulang hasil yang lebih lama.
Hanya label keluaran yang berubah; nilai bukti dan nama nilai registry mentah mempertahankan ejaan aslinya.

| Label sebelumnya | Label saat ini |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

Output CSV MFT memakai nama kolom khusus: `File`, `SN`, `ParentEntryNum`, `ParentSN`, `USN`, `LogfileSN`, `SID`, `Type`, dan `RefCnt`. Nama ini juga berlaku untuk CSV MFT dari `auto-parse`; lihat [kolom output MFT](../artifacts/mft.md#csv-output-fields).

Waktu JSON native untuk informasi sistem adalah `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`). `HasADS` adalah indikator Boolean; `Dir` di Prefetch berisi daftar
direktori yang tercatat. `User` mempertahankan tipe yang spesifik untuk artefaknya: string untuk nama akun,
atau daftar pengguna terkait dalam keluaran USB. Field ini tidak menyiratkan identitas yang mengeksekusi.
Beri tanda kutip pada tanda baca dalam query: `."SI<FN"` dan `.["ZeroU-Secs"]`.

## JSON { #json }

JSON menyimpan hasil lengkap sebagai satu dokumen. Format ini mempertahankan array, struktur bersarang, null,
integer besar, dan pembedaan yang tidak selalu dapat direpresentasikan oleh CSV datar.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL menulis satu objek JSON independen per baris. Format ini cocok untuk streaming dan bekerja secara alami dengan `jq`:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Ketika keluaran terstruktur diarahkan ke standard output, Juntei mengirim progres dan pesan ke standard
error sehingga pipe hanya menerima data.

`auto-parse -j timeline.jsonl` membuat dataset JSONL tambahan untuk koleksi multi-host
yang lengkap sambil mempertahankan hasil CSV yang dikategorikan. Tambahkan `-t json` atau `-t jsonl` untuk mengubah
format hasil individual. Setiap record memiliki field `Comp` dan `Artifact` sehingga sumbernya tetap jelas
setelah ingesti ke SIEM.

## UTC dan presisi { #utc-and-precision }

Juntei memformat waktu yang diurai dalam UTC menggunakan RFC 3339 jika formatnya memungkinkan. Beberapa artefak memiliki
presisi 100 nanodetik; artefak lain menyimpan detik, waktu FAT, interval agregasi, atau tidak menyimpan waktu sama sekali.
Jumlah digit bukanlah akurasi: selalu pahami field sumber yang dijelaskan di halaman artefaknya.

UTC menggunakan bentuk log peristiwa Windows `2026-10-20T08:46:38.2001668Z`, dengan mempertahankan presisi
pecahan yang tersedia. `auto-parse -Z "+09:00"` menampilkan waktu yang diurai dengan offset tetap
yang eksplisit di semua format hasilnya dan ekspor `-j` opsionalnya. Default-nya adalah UTC, terlepas dari
komputer yang menjalankan Juntei. Nilai mentah dan jam dinding tanpa zona tetap sebagaimana tercatat; lihat
[zona waktu keluaran](../commands/auto-parse.md#output-time-zone).

## Kerusakan dan nilai yang hilang { #damage-and-missing-values }

Field kosong, `null`, `NA`, `Problems`, dan field kondisi khusus parser memiliki makna. Field tersebut dapat berarti
artefak tidak menyimpan nilai, record rusak, format tidak mendukungnya, atau
parser tidak dapat me-resolve-nya. Jangan mengganti semua nilai kosong dengan nol.
