# Outlook { #outlook }

## Apa itu berkas Outlook { #what-outlooks-files-are }

Outlook untuk Windows menyimpan e-mail, kontak, kalender, dan tugas dalam berkas miliknya sendiri:

- **OST** (`.ost`): salinan offline sebuah kotak surat (mailbox), yang disimpan Outlook untuk akun
  Exchange dan Microsoft 365, dan sejak Outlook 2013 juga untuk akun IMAP. OST milik Outlook 2013 dan
  yang lebih baru memiliki halaman 4 KiB dan blok terkompresi.
- **PST** (`.pst`): store pribadi: akun POP3, arsip (`archive.pst`), hasil ekspor, dan store yang dibuat
  pengguna. PST ANSI adalah milik Outlook 97 hingga 2002; PST Unicode milik Outlook 2003 dan yang lebih
  baru.
- **MSG** (`.msg`): satu item yang disimpan sebagai berkas, saat item itu diseret keluar dari Outlook,
  disimpan dengan **Save As**, atau diekspor oleh program lain. Pesan yang dilampirkan juga merupakan
  berkas MSG.
- **OFT** (`.oft`): templat Outlook, yaitu berkas MSG yang menjadi dasar pembuatan item baru.

Setiap store berisi folder, dan setiap folder berisi item-itemnya: e-mail, permintaan rapat beserta
tanggapannya, kontak dan daftar distribusi, item kalender, tugas, catatan, entri jurnal, dan laporan.
Setiap folder juga menyimpan item tersembunyi, yaitu folder associated information (FAI) miliknya:
tampilan, aturan, formulir, dan pengaturan Outlook (item `IPM.Configuration.*`, seperti daftar
autocomplete sebuah kotak surat Exchange). Lampiran sebuah item dapat berupa berkas, referensi ke
berkas, objek OLE, atau item lain (pesan yang diteruskan sebagai lampiran, pengecualian dari janji temu
berulang), yang memiliki lampirannya sendiri.

<span class="evidence-strength">Kesimpulan terbaik:</span> store atau berkas tersebut memuat item itu,
dengan pengirim, penerima, waktu, dan lampiran tersebut, saat dikumpulkan. Ini tidak menyatakan bahwa
pengguna membaca, menulis, atau mengirimnya, dan juga tidak menyatakan bahwa pengirimnya benar-benar
orang yang disebutkan oleh item tersebut.

## Lokasi dan pengumpulan { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST bisa berada di mana saja: pengguna memindahkan arsip ke folder, drive, dan share berkas lain.
Outlook menyimpan path store setiap profil di registry pengguna. Lampiran yang dibuka pengguna dari
Outlook disalin ke sebuah folder di `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\`
(`Temporary Internet Files\Content.Outlook` sebelum Windows 8.1), dan bisa tetap tertinggal di sana.

Outlook terus membuka store-nya selama berjalan, dan sebuah OST bisa berukuran puluhan gigabyte.
Kumpulkan berkas-berkas itu saat Outlook ditutup, atau dari image disk atau volume shadow copy. Juntei
membaca setiap store sebagai stream: Juntei tidak pernah memuat seluruh store ke dalam memori.

## Peringatan bagi analis { #analyst-cautions }

- Juntei membaca secara toleran: Juntei melewati bagian yang rusak, membaca apa yang dapat dibaca, dan
  melaporkan sisanya sebagai masalah. Masalah-masalah itu ditampilkan (10 yang pertama untuk setiap
  berkas), dihitung dalam ringkasan dan tabel Berkas, serta dicantumkan di field `Problems` milik item
  dan lampiran yang terkait. Item yang sama sekali tidak dapat dibaca tetap memiliki baris di tabel Item,
  dengan `ItemID`-nya dan alasannya.
- Waktu dalam UTC, hingga 100 nanodetik. `SubmitTime` adalah kapan klien pengirim mengirimkan pesan
  tersebut, menurut jam komputer itu; `DeliveryTime` adalah kapan pesan itu tiba di kotak surat ini.
  `CreationTime` adalah kapan salinan item ini dibuat: dikirim ke atau dibuat di kotak surat,
  disinkronkan ke OST, disalin atau diimpor ke PST, atau disimpan sebagai berkas MSG. Itu bukan waktu
  pesan ditulis. `LastModifiedTime` adalah kapan item terakhir diubah di store.
- `StartTime` dan `DueTime` sebuah tugas adalah tanggal dalam zona waktu pengguna, yang disimpan Outlook
  sebagai tengah malam. Juntei menuliskannya tanpa offset (`2017-02-19T00:00:00`), dan `-Z` membiarkannya
  apa adanya.
- Outlook menuliskan tengah malam 1 Januari 4501 untuk **None** pada beberapa tanggal, seperti tanggal
  jatuh tempo tugas atau waktu pengingat. Juntei menuliskannya sebagaimana tersimpan:
  `4501-01-01T00:00:00Z`, tanpa `Z` pada `StartTime` dan `DueTime` sebuah tugas.
- `Read` adalah flag sudah-dibaca pada saat store dikumpulkan: seseorang, atau sebuah aturan, menandai
  item itu sebagai sudah dibaca. Flag ini tidak menyatakan siapa maupun kapan.
- Nama dan alamat pengirim serta penerima adalah apa yang dicatat oleh program pengirim dan server
  e-mail, dan dapat dipalsukan. Alamat `EX` adalah distinguished name Exchange
  (`/o=…/ou=…/cn=Recipients/cn=…`); alamat SMTP ada di `SenderSMTP-Address` dan `SMTP-Address` jika item
  juga menyimpannya. Untuk memeriksa asal sebuah pesan, baca header Internet-nya: `0x007D001F` atau
  `0x007D001E` (`PidTagTransportMessageHeaders`) di tabel Properti.
- Item yang dihapus di Outlook berada di **Deleted Items** sampai folder itu dikosongkan, dan dibaca
  seperti item lainnya. Item yang sudah tidak ada di tabel folder mana pun, yang dapat tetap disimpan
  sebuah store sampai ruangnya dipakai ulang, belum dibaca.
- `ItemID` adalah node id pesan di PST atau OST-nya: item yang sama di salinan lain dari store tersebut
  bisa memiliki ID lain. ID item tertanam adalah ID induknya dan indeks lampirannya (`2097348/0`,
  `0/0/1`); ID item sebuah berkas `.msg` atau `.oft` adalah `0`.
- `User` hanya berasal dari path (folder setelah `Users`), dan hilang jika berkas disalin ke tempat lain.
  Sebuah PST bisa milik pengguna mana saja.
- Teks yang bukan Unicode dibaca dengan code page item tersebut (code page miliknya sendiri, milik
  locale-nya, atau code page Internet-nya). `--code-page` memberikan code page untuk item yang tidak
  menyatakan code page-nya: code page ANSI sistem yang menulisnya, `932` untuk sistem berbahasa Jepang.
- Kata sandi PST tidak mengenkripsinya: Juntei membaca PST yang dilindungi kata sandi seperti PST
  lainnya. `Encryption` sebuah store (`Permute`, "compressible encryption" milik Outlook, atau `Cyclic`)
  hanya menyamarkan blok-bloknya. Pesan yang dienkripsi dengan S/MIME menyimpan isi dan lampirannya
  dalam sebuah lampiran terenkripsi (`smime.p7m`), yang tidak didekripsi.
- Lampiran sering kali berupa malware. `--extract-attachments` menulis datanya ke berkas yang dinamai
  dengan SHA-256-nya, tanpa izin eksekusi; jangan pernah menjalankannya, dan simpan di sistem analisis
  yang terisolasi.

### Belum dibaca { #not-read-yet }

Buku alamat offline (OAB), stream autocomplete (`Stream_Autocomplete_*.dat`) dan berkas `.nk2`,
pengaturan kirim/terima (`.srs`), cache pengaturan Outlook (RoamCache, Autodiscover), database SQLite
milik Outlook, dan pesan yang sudah tidak ada di tabel folder mana pun.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` mencari berkas yang berakhiran `.pst`, `.ost`, `.msg`, dan `.oft`, tanpa membedakan huruf besar
dan kecil. Bagaimanapun sebuah berkas diberikan, byte pertamanya menentukan jenisnya: `!BDN` untuk PST
atau OST, signature compound file untuk MSG atau OFT. Selain itu dilaporkan sebagai berkas yang tidak
dapat diurai. Status keluar adalah 1 jika tidak ada berkas yang dapat dibaca.

`--extract-attachments` menulis data setiap lampiran satu kali, sebagai `<SHA-256>` dengan ekstensi
lampiran tersebut jika ekstensinya terdiri dari 1 hingga 10 huruf dan angka ASCII (`0c7d…1311.jpg`);
tidak ada nama dari item yang dipakai dalam path. Direktori harus kosong kecuali `-C` diberikan; dengan
`-C`, berkas bernama sama yang berisi hal lain akan diganti.

## Tabel keluaran { #output-tables }

CSV membuat delapan berkas, sembilan dengan `--properties`, masing-masing hanya berisi header-nya jika
tidak ada baris. Baris JSON dan JSONL membawa `Table` (`Files`, `Folders`, `Items`, `Recipients`,
`Attachments`, `Contacts`, `Calendar`, `Tasks`, atau `Properties`) setelah `SourceFile`. Baris-baris
sebuah item mengikuti item tersebut, dan baris item tertanamnya mengikuti baris-baris itu; baris tabel
Berkas milik sebuah berkas muncul setelah semua baris lainnya dari berkas itu.

| Berkas CSV | Isi |
|---|---|
| `Outlook-Files.csv` | Satu baris per berkas yang dibaca: store atau berkas tersebut dan isinya. |
| `Outlook-Folders.csv` | Folder dari PST dan OST. |
| `Outlook-Items.csv` | Setiap item: pesan di folder, item tersembunyinya, item dari berkas `.msg` dan `.oft`, serta item yang tertanam di lampirannya. |
| `Outlook-Recipients.csv` | Penerima item. |
| `Outlook-Attachments.csv` | Lampiran item. |
| `Outlook-Contacts.csv` | Field kontak dan daftar distribusi. |
| `Outlook-Calendar.csv` | Field item kalender dan pesan rapat. |
| `Outlook-Tasks.csv` | Field tugas dan permintaan tugas. |
| `Outlook-Properties.csv` | Dengan `--properties`: setiap properti store, folder, item, penerima, dan lampiran, sebagaimana tersimpan. |

## Field { #fields }

Setiap tabel diawali dengan field berikut:

| Field | Interpretasi |
|---|---|
| `SourceFile` | Berkas tempat baris tersebut dibaca. |
| `User` | Pengguna Windows yang profilnya memuat path tersebut (folder setelah `Users`). |

### Berkas (Files) { #files }

| Field | Interpretasi |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, atau `CFB` untuk compound file yang dibaca sebagai item dan class ID-nya bukan milik pesan maupun templat (banyak program selain Outlook membiarkannya kosong). |
| `Format` | Tata letak store: `ANSI` (Outlook 97 hingga 2002), `Unicode` (Outlook 2003 dan yang lebih baru), atau `Unicode 4K` (OST milik Outlook 2013 dan yang lebih baru); versi compound file untuk berkas MSG atau OFT (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | `wVer` dan `wVerClient` dari header store. |
| `Encryption` | Cara blok store dienkode: `None`, `Permute`, atau `Cyclic`. |
| `DisplayName` | Nama store (`Personal Folders`, sebuah alamat e-mail). |
| `RecordKey` | Identitas store (heksadesimal), yang dimuat dalam entry id item-itemnya. |
| `Folders` | Folder milik store. |
| `Items` | Pesan di folder-foldernya, atau 1 untuk berkas MSG atau OFT. |
| `HiddenItems` | Item tersembunyi (folder associated information) di folder-foldernya. |
| `EmbeddedItems` | Item yang tertanam di lampiran, di semua tingkat. |
| `Attachments` | Lampiran dari semua item tersebut. |
| `Problems` | Masalah yang ditemukan di berkas. |

### Folder (Folders) { #folders }

| Field | Interpretasi |
|---|---|
| `FolderID`, `ParentID` | Node id folder, dan node id folder induknya (kosong untuk folder root). |
| `FolderPath` | Nama-nama folder dari root ke bawah: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | Nama folder. |
| `ContainerClass` | Isi folder: `IPF.Note` (e-mail), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Pesan, dan pesan yang belum dibaca, yang menurut folder itu dimilikinya. |
| `Messages`, `HiddenItems`, `Subfolders` | Baris dari tabel isi, tabel isi terkait (associated contents), dan tabel hierarkinya: yang dibaca oleh Juntei. Selisih dengan `ContentCount` patut diperiksa. |
| `CreationTime`, `LastModifiedTime` | Kapan folder dibuat dan terakhir diubah (UTC), jika store menyimpannya. |
| `Problem` | Alasan folder tidak dapat dibuka; subfoldernya kemudian tidak dibaca. |

### Item (Items) { #items }

| Field | Interpretasi |
|---|---|
| `FolderPath` | Folder item tersebut; kosong di berkas MSG atau OFT. |
| `ItemID` | ID item: node id-nya di PST atau OST, `0` di berkas MSG atau OFT, ID induknya dan indeks lampiran untuk item tertanam (`2097348/0`). |
| `Hidden` | Apakah item tersebut merupakan folder associated information, yaitu item tersembunyi. |
| `EmbeddedIn` | ID item tempat item ini dilampirkan; kosong untuk item lainnya. |
| `ItemType` | Jenis item menurut kelas pesannya (message class): `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration`, atau `Other`. |
| `MessageClass` | Kelas pesan sebagaimana tersimpan (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Kapan klien pengirim mengirimkan pesan tersebut (UTC, menurut jam pengirim). |
| `DeliveryTime` | Kapan pesan itu dikirimkan ke kotak surat ini (UTC). |
| `CreationTime`, `LastModifiedTime` | Kapan salinan ini dibuat di store atau berkas, dan terakhir diubah (UTC). |
| `Subject` | Subjek, beserta awalannya (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Siapa yang mengirimnya: nama, tipe alamat (`SMTP`, `EX`), dan alamat dengan tipe tersebut. |
| `SenderSMTP-Address` | Alamat SMTP pengirim, jika disimpan di samping alamat bertipe lain. |
| `SentRepresentingName`, `SentRepresentingAddress` | Atas nama siapa pesan itu dikirim (delegasi mengirim atas nama manajer; kotak surat bersama), dengan alamat SMTP-nya jika tersimpan, atau alamat dengan tipenya sendiri jika tidak. |
| `To`, `Cc`, `Bcc` | Nama-nama penerima sebagaimana ditampilkan item (lihat tabel Penerima untuk alamatnya). `Bcc` hanya ada di salinan milik pengirim. |
| `InternetMessageID`, `InReplyTo` | `Message-ID` pesan tersebut, dan milik pesan yang dibalasnya. |
| `ConversationTopic` | Subjek tanpa awalannya, yang sama untuk semua pesan dalam satu percakapan. |
| `Importance`, `Sensitivity` | `Low`, `Normal`, atau `High`; `Normal`, `Personal`, `Private`, atau `Confidential`. |
| `Read` | Apakah item ditandai sudah dibaca. |
| `Flags` | Flag pesan (`PidTagMessageFlags`, heksadesimal): `0x1` sudah dibaca, `0x2` tidak diubah, `0x4` sudah dikirimkan (submitted), `0x8` belum terkirim (draf), `0x10` memiliki lampiran, `0x20` dari saya, `0x40` tersembunyi, `0x80` dikirim ulang, `0x400` pernah dibaca, `0x2000` dari Internet. |
| `Size` | Ukuran item di store, termasuk lampirannya. |
| `Recipients`, `Attachments` | Berapa banyak penerima dan lampiran yang dimilikinya. |
| `Categories` | Kategorinya, dipisahkan dengan `;`. |
| `BodyPreview` | Awal isi teks biasanya (plain text), dengan spasi yang diringkas, paling banyak 256 karakter; kosong jika item hanya memiliki isi HTML atau RTF. |
| `Problems` | Apa yang tidak dapat dibaca di item tersebut (paling banyak 10, lalu berapa banyak lagi). |

### Penerima (Recipients) { #recipients }

| Field | Interpretasi |
|---|---|
| `ItemID` | ID item. |
| `Index` | Baris penerima di tabel penerima milik item, mulai dari 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator`, atau nilai sebagaimana tersimpan (heksadesimal). |
| `Name`, `AddressType`, `Address` | Nama, tipe alamat, dan alamat penerima. |
| `SMTP-Address` | Alamat SMTP, jika disimpan di samping alamat bertipe lain. |

### Lampiran (Attachments) { #attachments }

| Field | Interpretasi |
|---|---|
| `ItemID`, `Index` | ID item, dan baris lampiran di tabel lampirannya, mulai dari 0. |
| `Method` | `ByValue` (sebuah berkas), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (path ke sebuah berkas), `EmbeddedMessage` (sebuah item), `OLE` (sebuah objek OLE), `ByWebReference`, atau angkanya. |
| `Filename` | Nama lampiran: nama berkas panjangnya, nama pendeknya, atau nama tampilannya. |
| `Extension` | Ekstensinya sebagaimana tersimpan (`.pdf`), atau jika tidak ada, ekstensi dari namanya. |
| `MimeType`, `ContentID` | Tipe MIME-nya, dan content id yang digunakan isi HTML untuk merujuknya. |
| `Inline` | Apakah isi pesan menampilkannya (gambar dalam isi HTML) alih-alih mencantumkannya. |
| `Hidden` | Apakah Outlook menyembunyikannya dari daftar lampiran. |
| `Size` | Ukurannya sebagaimana diberikan store, termasuk properti-propertinya. |
| `DataSize`, `SHA256` | Ukuran dan SHA-256 datanya: berkasnya, atau compound file sebuah objek OLE di PST atau OST. Kosong untuk item tertanam dan lampiran berupa referensi. |
| `Pathname` | Path lampiran berupa referensi. |
| `CreationTime`, `LastModifiedTime` | Waktu-waktu lampiran (UTC): sering kali sama dengan waktu berkas asal pembuatannya. |
| `EmbeddedSubject` | Subjek item tertanam, yang baris-barisnya sendiri memiliki `ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | Dengan `--extract-attachments`, berkas di direktori tersebut tempat datanya ditulis. |
| `Problems` | Apa yang tidak dapat dibaca, atau ditulis, pada lampiran tersebut. |

### Kontak (Contacts) { #contacts }

| Field | Interpretasi |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Seperti di tabel Item: `Contact` atau `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Nama-nama kontak. |
| `FileUnder` | Nama yang digunakan untuk mengarsipkan kontak (file under). |
| `Company`, `Title`, `Department` | Tempat kontak bekerja. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Tiga alamat e-mail kontak, masing-masing dengan nama tampilannya. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Nomor telepon kontak. |
| `InstantMessaging` | Alamat pesan instan kontak. |
| `CreationTime`, `LastModifiedTime` | Seperti di tabel Item. |

### Kalender (Calendar) { #calendar }

| Field | Interpretasi |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Seperti di tabel Item: `Appointment` atau `MeetingMessage`. |
| `StartTime`, `EndTime` | Kapan janji temu dimulai dan berakhir (UTC); untuk janji temu berulang, waktu kemunculan pertamanya. |
| `DurationMinutes` | Durasinya dalam menit. |
| `Location` | Lokasinya. |
| `AllDay`, `Recurring` | Apakah merupakan acara sepanjang hari, dan apakah berulang (pengecualiannya adalah item tertanam). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice`, atau `WorkingElsewhere`. |
| `ResponseStatus` | Tanggapan pengguna: `None`, `Organized`, `Tentative`, `Accepted`, `Declined`, atau `NotResponded`. |
| `Organizer` | Nama pengirim item. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Para peserta, sebagaimana dicantumkan item. |
| `GlobalObjectID` | Identitas rapat lintas kotak surat (heksadesimal): rapat yang sama di kalender penyelenggara dan para peserta. |
| `ReminderSet`, `ReminderTime` | Apakah pengingat diatur, dan untuk waktu kapan (UTC). |
| `CreationTime`, `LastModifiedTime` | Seperti di tabel Item. |

### Tugas (Tasks) { #tasks }

| Field | Interpretasi |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Seperti di tabel Item: `Task` atau `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting`, atau `Deferred`. |
| `PercentComplete` | Seberapa banyak yang sudah selesai, dari 0 hingga 100. |
| `StartTime`, `DueTime` | Kapan tugas dimulai dan jatuh tempo: tanggal dalam zona waktu pengguna, tanpa offset. |
| `CompletedTime` | Kapan tugas diselesaikan (UTC). |
| `Complete` | Apakah tugas sudah selesai. |
| `Owner`, `Assigner` | Siapa pemiliknya, dan siapa yang menugaskannya. |
| `ReminderSet`, `ReminderTime` | Seperti di tabel Kalender. |
| `CreationTime`, `LastModifiedTime` | Seperti di tabel Item. |

### Properti (Properties) { #properties }

| Field | Interpretasi |
|---|---|
| `ItemID` | ID item; `store` untuk properti milik store sendiri; `folder:<FolderID>` untuk properti sebuah folder. |
| `Scope` | Pemilik properti: `Store`, `Folder`, `Item`, `Recipient`, atau `Attachment`. |
| `Index` | Indeks penerima atau lampiran; kosong untuk yang lain. |
| `PropertyTag` | ID dan tipe properti, dalam bentuk `0x0037001F` (`PidTagSubject`, sebuah string). |
| `PropertyName` | Untuk named property (ID mulai dari `0x8000`), property set dan namanya: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, atau GUID set tersebut jika bukan set yang umum dikenal. |
| `Type` | Tipe properti: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | Nilainya: teks untuk string, ISO 8601 UTC untuk waktu (yang tidak dikonversi oleh `-Z`), heksadesimal untuk nilai biner (4.096 byte pertama, lalu `…(N bytes)`), nilai-nilai properti bernilai ganda sebagai daftar (dipisahkan `, ` dalam CSV), `(object)` untuk item tertanam atau data lampiran, dan byte dalam heksadesimal disertai catatan untuk nilai yang rusak atau bertipe yang tidak didekode. |

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis tabel-tabel ke kategori baru `Communication` sebagai `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv`, dan `Outlook-Tasks.csv` (satu berkas `Outlook.json`
atau `Outlook.jsonl` dengan `-t json` atau `-t jsonl`), beserta `Communication/Outlook-logs.txt`.
Perintah ini meneruskan `--code-page`-nya ke parser, dan tidak menambahkan tabel Properti maupun
mengekstrak lampiran: jalankan `parse-file outlook` untuk keperluan itu. Timeline gabungan `-j`
menyertakan setiap baris, dengan `Comp` dan pengenal `Artifact` `outlook`. `-Z` mengonversi setiap
field yang namanya berakhiran `Time`, kecuali `StartTime` dan `DueTime` sebuah tugas.

## Detail lebih lanjut { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
