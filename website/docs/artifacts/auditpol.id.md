# Kebijakan audit { #audit-policy }

## Apa itu kebijakan audit { #what-it-is }

Kebijakan audit menentukan peristiwa keamanan mana yang dicatat Windows ke log peristiwa Security. Kebijakan ini diatur
per subkategori pada Windows Vista dan yang lebih baru (59 pada Windows 10 dan 11), serta per kategori pada
Windows XP dan Server 2003. Masing-masing mencatat keberhasilan, kegagalan, keduanya, atau tidak sama sekali.

LSA menyimpan kebijakan yang berlaku di hive `SECURITY`, baik yang berasal dari `auditpol`, kebijakan keamanan
lokal, maupun Group Policy.

<span class="evidence-strength">Kesimpulan terbaik:</span> sistem diatur untuk mencatat peristiwa ini (atau
tidak) ketika hive terakhir ditulis. Bila sebuah subkategori tidak diaudit, ketiadaan peristiwa tidak mengatakan
apa pun tentang apakah aktivitas tersebut terjadi.

## Lokasi { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista dan yang lebih baru menyimpan header 12 byte, lalu satu pengaturan per subkategori, kategori demi
kategori, lalu jumlah subkategori yang dimiliki setiap kategori. Versi-versi hanya berbeda dalam jumlah tersebut:
setiap versi menambahkan subkategori di akhir sebuah kategori (56 pada hive Windows 8.x dan Server 2012 R2
yang dibaca, 59 pada Windows 10, 11, dan Server 2022). Windows XP dan Server 2003 menyimpan apakah audit
aktif (satu byte, lalu padding yang tidak dibersihkan Windows) dan sembilan pengaturan kategori. Tata letak itu
diperiksa pada image Windows XP publik dari CFReDS Hacking Case milik NIST, yang tidak mengaudit apa pun;
nilai pengaturan kategorinya mengikuti dokumentasi Microsoft.

Kumpulkan `SECURITY` beserta log transaksinya, serta hive `SYSTEM` milik sistem, yang menyebutkan nama
host.

## Peringatan bagi analis { #analyst-cautions }

- **Data yang tidak diketahui tetap tidak diketahui.** Revisi yang tidak didukung dipertahankan dengan diagnostik. Flag
  pengaturan yang tidak diketahui membuat `Success` dan `Failure` kosong dan dikecualikan dari hitungan pengaturan yang diaudit.
  Tabel hitungan yang rusak tidak dapat mengubah word yang dicadangkan menjadi pengaturan audit lain.
  Hasil tangkapan dapat berisi slot tambahan di luar katalog nama/GUID. Pengaturan dan
  `SubcategoryId` numeriknya dipertahankan dengan nama/GUID kosong; slot yang tidak terpetakan bukanlah
  bukti kerusakan dengan sendirinya.


- **Urutannya adalah milik Windows sendiri.** Di dalam sebuah kategori, pengaturan mengikuti urutan message ID
  subkategori, bukan GUID-nya. Special Logon (`%%12548`, indeks 4 dari Logon/Logoff)
  muncul sebelum IPsec Quick Mode. Alat yang membacanya dalam urutan GUID melaporkan default Windows 10
  sebagai Special Logon "No Auditing" dan IPsec Quick Mode "Success". `SubcategoryId` adalah
  ID yang sama dengan yang diberikan peristiwa 4719 (System audit policy was changed), sehingga keduanya dapat dicocokkan.
- **Kebijakan yang berlaku, bukan sumbernya.** Group Policy dan kebijakan lokal menulis nilai yang sama.
  `audit.csv` milik Group Policy (di SYSVOL dan `C:\Windows\System32\GroupPolicy`) menunjukkan apa yang
  dimaksudkan untuk diterapkan (`Machine\Microsoft\Windows NT\Audit\audit.csv` dari sebuah kebijakan).
- **Perubahan dari waktu ke waktu:** `KeyLastWrittenUTC` menunjukkan kira-kira kapan kebijakan terakhir berubah. Peristiwa 4719
  mencatat setiap perubahan (selama log Security masih menyimpannya). Penyerang dapat menonaktifkan audit,
  lalu mengaktifkannya kembali.
- **Subkategori versus kategori.** Pada Windows Vista dan yang lebih baru, subkategorilah yang
  digunakan Windows (kecuali opsi keamanan `SCENoApplyLegacyAuditPolicy` dinonaktifkan dan kebijakan kategori
  lama menimpanya).
- **Windows XP dan 2003:** dengan audit dinonaktifkan (`AuditingEnabled` False), tidak ada kategori yang diaudit,
  apa pun pengaturannya.
- **Peristiwa:** `Events` mencantumkan peristiwa utama setiap subkategori, berdasarkan dokumentasi Microsoft
  tentang pengaturan advanced audit policy. Inilah yang biasanya dicari dalam investigasi, bukan
  setiap peristiwa yang dicatat subkategori tersebut.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Untuk setiap hive, ringkasan menampilkan tata letaknya, berapa banyak pengaturan yang diaudit, dan kapan kebijakan
terakhir ditulis. Ringkasan mencantumkan apa yang diaudit untuk keberhasilan dan kegagalan, hanya keberhasilan, dan hanya
kegagalan, lalu subkategori yang tidak diaudit, beserta peristiwa utamanya: peristiwa-peristiwa tersebut
dinonaktifkan dalam kebijakan yang ditangkap; hal ini tidak menetapkan kebijakan yang berlaku selama aktivitas sebelumnya.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Host (dari hive `SYSTEM` milik sistem), dan hive `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista dan yang lebih baru) atau `categories` (XP dan 2003); kosong bila nilai tidak dapat didekode. |
| `Category`, `Subcategory` | Sebagaimana dinamai oleh `auditpol`. |
| `SubcategoryGUID`, `SubcategoryID` | GUID subkategori, dan message ID-nya sebagaimana diberikan oleh peristiwa 4719 (`%%12548`). |
| `Setting` | `none`, `success`, `failure`, atau `success_and_failure`. |
| `Success`, `Failure` | Apakah keberhasilan dan kegagalan dicatat. |
| `AuditingEnabled` | Windows XP dan 2003: apakah audit aktif sama sekali. |
| `Events` | Event ID utama subkategori. |
| `KeyLastWrittenUTC` | Last write dari `PolAdtEv`: kira-kira kapan kebijakan terakhir berubah. |
| `DataRaw` | Byte nilai (hex), bila tidak dapat didekode secara utuh. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Tata letak atau ukuran yang tidak diketahui, hitungan yang tidak sesuai, dan nilai di atas 3. |

## Strategi investigasi { #investigation-strategy }

1. Sebelum menyimpulkan bahwa sesuatu tidak terjadi, periksa apakah subkategorinya diaudit:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (tugas terjadwal, 4698), Security System Extension (instalasi layanan, 4697), User Account and
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Bandingkan kebijakan dengan log Security: peristiwa dari subkategori yang saat ini tidak diaudit
   berarti kebijakan telah berubah. Cari peristiwa 4719 dan `KeyLastWrittenUTC` yang baru.
3. Bandingkan dengan `audit.csv` milik Group Policy untuk membedakan kebijakan tingkat domain dari perubahan lokal.
4. Gunakan `parse-registry read --recover-deleted` dan log transaksi untuk versi sebelumnya dari
   `PolAdtEv`.

## Detail lebih lanjut { #further-details }

- [Microsoft: Pengaturan advanced security audit policy](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Konstanta audit (GUID subkategori)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
