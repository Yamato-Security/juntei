# UserAssist { #userassist }

## Apa itu UserAssist { #what-userassist-is }

Windows Explorer menghitung program dan pintasan yang dijalankan pengguna dari shell Explorer:
Start menu, taskbar, desktop, dan klik ganda di folder. Explorer menyimpan hitungan tersebut di
`NTUSER.DAT` milik pengguna. Setiap item memiliki:

- jumlah eksekusi;
- pada Windows 7 dan yang lebih baru, berapa kali dan berapa lama (dalam milidetik) item tersebut mendapatkan fokus;
- kapan item tersebut terakhir dijalankan.

Nama nilai dienkode dengan ROT13. Nama program biasanya diawali dengan GUID known folder
(`{1AC14E77-…}` adalah `System32`).

UserAssist menunjukkan apa yang dijalankan secara interaktif oleh pengguna tertentu, hal yang tidak diatribusikan ke pengguna
oleh sebagian besar bukti eksekusi lainnya.

<span class="evidence-strength">Kesimpulan terbaik:</span> pengguna menjalankan program atau pintasan ini
melalui Explorer, sebanyak ini, terakhir kali pada waktu ini. Program yang dijalankan dengan cara lain (baris
perintah, layanan, tugas terjadwal, program lain) tidak dihitung.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Isi |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Program (`executable`), Windows 7 dan yang lebih baru |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Pintasan (`shortcut`), Windows 7 dan yang lebih baru |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Program dan pintasan (`active_desktop`), Windows 2000 hingga Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Toolbar internet (`internet_toolbar`), Windows 2000 hingga Vista |

GUID lain dilaporkan sebagai `other`. Kumpulkan `NTUSER.DAT` beserta log transaksinya untuk setiap
profil, serta hive `SOFTWARE` milik sistem, yang memberikan SID pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Profil baru diisi item awal.** Pada Windows 7 dan yang lebih baru, profil baru dimulai dengan item default
  (Getting Started, Calculator, Paint, Sticky Notes, …). Jumlah eksekusinya menghitung mundur dari 14,
  dan semuanya berbagi satu waktu eksekusi terakhir, sekitar saat profil dibuat. Item-item ini bukan aktivitas
  pengguna.
- **Versi 3 menghitung dari 5** (Windows 2000 hingga Vista). Hitungan yang tersimpan dimulai dari 5, sehingga 5
  dikurangkan darinya. Hitungan tersimpan di bawah 5 dilaporkan di `Problems`.
- **Path menggunakan path default known folder** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Folder yang dialihkan ke tempat lain tetap menampilkan path default-nya. App ID
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) dan item Control Panel
  (`::{…}`) tidak memiliki path.
- **Pelacakan dapat dinonaktifkan** dengan `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1, atau `UserAssist\Settings` `NoLog` = 1. Ringkasan
  mencantumkan hive tempat pelacakan dinonaktifkan. UserAssist yang kosong atau tidak ada pada profil yang aktif dapat menandakan
  anti-forensik.
- **`NoEncrypt` = 1** membuat Windows menulis nama tanpa ROT13. Nama-nama seperti itu terdekode secara salah, sehingga
  setiap item dari hive tersebut menyatakannya di `Problems` (dan di stderr ketika record dikirim ke stdout).
- **Waktu eksekusi terakhir dan hitungan disimpan per item,** tetapi waktu last-write key
  (`KeyLastWrittenUTC`) hanyalah waktu pembaruan terbaru pada salah satu item.
- **Jumlah eksekusi dapat bernilai 0** untuk item yang hanya mendapatkan fokus. Hitungan dan waktu fokus hanya ada di
  versi 5 (Windows 7 dan yang lebih baru).

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Ringkasan mencantumkan item yang terakhir dijalankan (terbaru terlebih dahulu) beserta penggunanya, serta hive tempat
UserAssist dinonaktifkan.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nama folder profil, SID pengguna dari `ProfileList` milik sistem (dengan hive `SOFTWARE` milik sistem), dan hive. |
| `GUID`, `Kind` | Key `{GUID}` dan isinya (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Nama nilai sebagaimana tersimpan, dan setelah didekode. |
| `Path` | Nama yang didekode sebagai path, dengan GUID known folder diganti dengan path default-nya; kosong untuk app ID dan shell item. |
| `RunCount` | Berapa kali item dijalankan dari Explorer (versi 3: hitungan tersimpan dikurangi 5). |
| `FocusCount`, `FocusTimeMs` | Berapa kali item mendapatkan fokus, dan berapa lama dalam milidetik (versi 5). |
| `LastRunUTC` | Kapan item terakhir dijalankan. Kosong bila Windows menyimpan 0, atau bila waktu yang tersimpan melewati tahun 9999 (dalam hal itu `Problems` mencantumkannya dan `DataRaw` menyimpan byte-nya). |
| `Format`, `SessionID` | Tata letak data (5: 72 byte, 3: 16 byte) dan nomor sesi tempat data terakhir ditulis. |
| `KeyLastWrittenUTC` | Last write dari key `Count`: pembaruan terbaru pada salah satu itemnya. |
| `DataRaw` | Byte nilai (hex), bila tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Data dengan ukuran yang tidak diketahui, ukuran yang tidak sesuai dengan `Version` key, hitungan versi 3 di bawah 5, eksekusi terakhir melewati tahun 9999, `NoEncrypt`. |

## Strategi investigasi { #investigation-strategy }

1. Urutkan berdasarkan `LastRunUTC` dan sisihkan item awal bawaan: semuanya berbagi satu waktu, sekitar saat
   pembuatan profil.
2. Cari program di lokasi yang dapat ditulis pengguna (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), drive lepas pasang, dan path jaringan, serta alat administrasi dan akses
   jarak jauh.
3. Korelasikan dengan Prefetch (eksekusi beserta waktu eksekusi), Amcache dan ShimCache (keberadaan),
   Jump Lists dan berkas LNK (apa yang dibuka dengannya), serta `$MFT` untuk berkas program.
4. Periksa hive tempat pelacakan dinonaktifkan, dan profil yang UserAssist-nya kosong, untuk mendeteksi pembersihan.
5. Gunakan `parse-registry read --recover-deleted` untuk nilai UserAssist yang dihapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Key User Assist](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
