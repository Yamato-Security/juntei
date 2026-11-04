# Microsoft Office (berkas terbaru, Trusted Documents, pengaturan makro) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Apa yang dicatat Office { #what-office-records }

Microsoft Office menyimpan, per pengguna di `NTUSER.DAT` serta per versi dan aplikasi:

- **Berkas dan lokasi terbaru:** `File MRU`, dokumen-dokumen yang dibuka oleh sebuah aplikasi, dan `Place MRU`,
  folder-folder tempat dokumen itu dibuka, masing-masing item dengan waktu terakhir kali dibuka. Microsoft 365
  juga menyimpan daftar per akun yang sedang masuk (`User MRU\<account>`).
- **Trusted Documents:** ketika pengguna mengklik *Enable Editing* atau *Enable Content* pada sebuah dokumen,
  Office mengingatnya di `TrustRecords`: path dokumen, kapan dokumen itu dipercaya, dan apakah
  makronya diaktifkan.
- **Pengaturan makro:** `VBAWarnings` (makro mana yang dijalankan), `BlockContentExecutionFromInternet`
  (apakah makro dalam berkas dari Internet diblokir), `AccessVBOM` (apakah program boleh
  mengubah proyek VBA), dan **Trusted Locations**, folder-folder yang makro dalam dokumennya dijalankan tanpa
  konfirmasi. Group Policy dapat menetapkan hal yang sama di bawah `Software\Policies`.

Dokumen berbahaya biasanya membutuhkan pengguna untuk mengaktifkan makronya: trust record dengan makro
yang diaktifkan mengaitkan dokumen, pengguna, dan waktu.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna membuka dokumen ini di
aplikasi ini (dan terakhir kali pada waktu ini); pengguna mengaktifkan pengeditan atau makro dokumen ini pada
waktu ini; Office dikonfigurasi untuk menjalankan makro dengan cara ini.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versi: `16.0` adalah Office 2016 dan yang lebih baru (2019, 2021, 2024, Microsoft 365), `15.0` Office 2013,
`14.0` Office 2010, `12.0` Office 2007.

Sebuah item terlihat seperti `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: `T`
adalah waktu terakhir dibuka (FILETIME dalam heksadesimal), path mengikuti `*`. Sebuah trust record berukuran 24
byte: FILETIME saat dokumen dipercaya, lalu flag-nya di 4 byte terakhir, `0x7FFFFFFF`
jika makronya diaktifkan dan `0x00000001` jika hanya pengeditannya yang diaktifkan.

## Peringatan bagi analis { #analyst-cautions }

- **Waktu.** `LastOpenedUTC` adalah waktu milik setiap item. `TrustedAtUTC` adalah saat pengguna memercayai
  dokumen, yaitu saat *Enable Editing* atau *Enable Content* diklik, yang dapat lebih awal dari
  pembukaan terbaru.
- **Makro diaktifkan, bukan dijalankan.** `MacrosEnabled` menyatakan bahwa pengguna mengaktifkan makro dokumen; ini
  tidak membuktikan bahwa makro dijalankan atau apa yang dilakukannya. Korelasikan dengan pembuatan proses, Prefetch, log
  PowerShell, dan dokumen itu sendiri.
- **Path sebagaimana disimpan Office.** Trust record menamai dokumen dengan path yang dapat menggunakan
  `%USERPROFILE%`, garis miring maju, URL `file:///`, dan URL encoding (`%20`). Item MRU untuk
  dokumen cloud berupa URL.
- **Default dan kebijakan efektif.** Office membuat trusted location default (`Templates`,
  `XLSTART`, `STARTUP`, …). `Date` sebuah lokasi merupakan petunjuk yang berguna, bukan bukti siapa yang membuatnya:
  administrator, skrip, kebijakan, dan pengaturan registry yang disalin juga dapat mengisinya. `VBAWarnings`
  yang tidak ada saja tidak menetapkan kebijakan makro yang efektif. Versi Office, penandaan berkas
  Internet, dan pengaturan manajemen lainnya juga berpengaruh.
  [Microsoft menjelaskan pemblokiran makro Internet](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Dokumen yang dihapus.** Daftar-daftar ini bertahan lebih lama daripada dokumennya; berkas yang sudah tidak ada masih dapat ditampilkan
  di sini. Daftarnya pendek dan dikosongkan oleh *Clear unpinned items* di Office serta alat pembersih.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Ringkasan mencantumkan dokumen yang makronya diaktifkan, pengaturan yang membuat makro berjalan lebih
leluasa (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), trusted location yang ditambahkan pengguna atau ditetapkan kebijakan, serta dokumen
yang terakhir dibuka.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, pemilik profil (dari path atau shell folder), SID dari `ProfileList` sistem, dan hive. |
| `OfficeVersion`, `App` | Key versi (`16.0`) dan aplikasinya (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security`, atau `Trusted Locations`. |
| `Account` | Akun Microsoft 365 dari sebuah daftar `User MRU`. |
| `Policy` | Apakah pengaturan atau lokasi ditetapkan oleh kebijakan. |
| `MRU-Position`, `IsMostRecent` | Posisi item di daftarnya (`Item 1` adalah 0, yang terbaru). |
| `Path` | Dokumen atau folder; path trusted document sebagaimana tersimpan; `Path` dari trusted location. |
| `LastOpenedUTC` | Kapan item terakhir kali dibuka. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Kapan dokumen dipercaya, apakah makronya diaktifkan, dan flag record tersebut. Flag yang tidak dikenal membuat `MacrosEnabled` kosong dan menghasilkan diagnostik. |
| `Setting`, `Value` | Sebuah pengaturan makro dan nilainya beserta artinya (`1 (all macros enabled)`); key trusted location (`Location3`) dan nilai-nilai lainnya (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | Key dan waktu penulisan terakhirnya. |
| `DataRaw` | Byte nilai (hex) jika tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Item tanpa `*` atau dengan waktu yang tidak valid, trust record yang tidak berukuran 24 byte, lokasi tanpa `Path`, tipe yang salah. |

## Strategi investigasi { #investigation-strategy }

1. Mulailah dengan `MacrosEnabled` bernilai true: dokumennya, kapan, dan oleh siapa. Temukan berkasnya (`$MFT`,
   Jurnal USN, `Zone.Identifier`), sumbernya (riwayat browser dan email), dan apa yang dijalankan sesudahnya.
2. Periksa pengaturan makro dan trusted location: semua makro diaktifkan, makro Internet tidak
   diblokir, akses proyek VBA, atau lokasi yang ditambahkan pengguna (di `Downloads`, `Temp`, network share)
   melemahkan pengaturan default.
3. Masukkan `LastOpenedUTC` ke dalam timeline bersama berkas LNK, jump list, dan RecentDocs untuk
   path dokumen, termasuk lokasi removable dan jaringan.

## Detail lebih lanjut { #further-details }

- [Microsoft: Mengaktifkan atau menonaktifkan makro dalam berkas Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Dokumen tepercaya](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Menambahkan, menghapus, atau mengubah lokasi tepercaya](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
